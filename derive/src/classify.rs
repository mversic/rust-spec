//! Syntactic type-shape classification, following co3's tail classifier.
//!
//! Unknown names and projections retain trait-based classification. A tuple,
//! like a struct, takes its sizedness from its last element.
use syn::Type;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TypeClass {
    Sized,
    Unsized(UnsizedTail),
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UnsizedTail {
    SliceLike,
    DynTraitLike,
    Unknown,
}

fn peel(ty: &Type) -> &Type {
    match ty {
        Type::Paren(t) => peel(&t.elem),
        Type::Group(t) => peel(&t.elem),
        _ => ty,
    }
}

/// Only inspect field structure; a named wrapper owns the meaning of its type arguments.
pub(crate) fn contains_raw_pointer(ty: &Type) -> bool {
    match peel(ty) {
        Type::Ptr(_) => true,
        Type::Array(array) => contains_raw_pointer(&array.elem),
        Type::Slice(slice) => contains_raw_pointer(&slice.elem),
        Type::Tuple(tuple) => tuple.elems.iter().any(contains_raw_pointer),
        _ => false,
    }
}

fn sized_bound(bound: &syn::TypeParamBound) -> Option<bool> {
    let syn::TypeParamBound::Trait(bound) = bound else {
        return None;
    };
    let path = &bound.path;
    let is_sized = path.is_ident("Sized")
        || (path.segments.len() == 3
            && (path.segments[0].ident == "core" || path.segments[0].ident == "std")
            && path.segments[1].ident == "marker"
            && path.segments[2].ident == "Sized");
    is_sized.then(|| bound.maybe.is_none())
}

pub(crate) fn classify_type(ty: &Type, generics: &syn::Generics) -> TypeClass {
    let ty = peel(ty);
    let param = match ty {
        Type::Path(path) if path.qself.is_none() => path
            .path
            .get_ident()
            .and_then(|ident| generics.type_params().find(|p| p.ident == *ident)),
        _ => None,
    };
    let bounds = param
        .into_iter()
        .flat_map(|p| &p.bounds)
        .chain(
            generics
                .where_clause
                .iter()
                .flat_map(|c| &c.predicates)
                .filter_map(|p| match p {
                    syn::WherePredicate::Type(p) if peel(&p.bounded_ty) == ty => Some(&p.bounds),
                    _ => None,
                })
                .flatten(),
        )
        .filter_map(sized_bound)
        .collect::<Vec<_>>();
    if bounds.contains(&true) {
        return TypeClass::Sized;
    }
    if param.is_some() {
        return if bounds.contains(&false) {
            TypeClass::Unsized(UnsizedTail::Unknown)
        } else {
            TypeClass::Sized
        };
    }
    match ty {
        Type::Slice(_) => TypeClass::Unsized(UnsizedTail::SliceLike),
        Type::TraitObject(_) => TypeClass::Unsized(UnsizedTail::DynTraitLike),
        Type::Reference(_) | Type::Ptr(_) | Type::Array(_) | Type::FnPtr(_) | Type::Never(_) => {
            TypeClass::Sized
        }
        Type::Tuple(t) => t
            .elems
            .last()
            .map_or(TypeClass::Sized, |tail| classify_type(tail, generics)),
        _ => TypeClass::Unknown,
    }
}

/// What the type syntax proves about its size, independent of any RustSpec impl.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NonZeroClass {
    Yes,
    No,
    Unknown,
}

pub(crate) fn classify_nonzero(ty: &Type) -> NonZeroClass {
    match peel(ty) {
        Type::Reference(_) | Type::Ptr(_) | Type::FnPtr(_) => NonZeroClass::Yes,
        Type::Tuple(tuple) => {
            let mut result = NonZeroClass::No;
            for element in &tuple.elems {
                match classify_nonzero(element) {
                    NonZeroClass::Yes => return NonZeroClass::Yes,
                    NonZeroClass::Unknown => result = NonZeroClass::Unknown,
                    NonZeroClass::No => {}
                }
            }
            result
        }
        Type::Array(array) => match &array.len {
            syn::Expr::Lit(expr) => match &expr.lit {
                syn::Lit::Int(length) => match length.base10_parse::<usize>() {
                    Ok(0) => NonZeroClass::No,
                    Ok(_) => classify_nonzero(&array.elem),
                    Err(_) => NonZeroClass::Unknown,
                },
                _ => NonZeroClass::Unknown,
            },
            _ => NonZeroClass::Unknown,
        },
        _ => NonZeroClass::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;
    #[test]
    fn nonzero_size_proofs_distinguish_zero_and_unknown() {
        for ty in [
            parse_quote!(()),
            parse_quote!(((), ())),
            parse_quote!([u8; 0]),
            parse_quote!([*const (); 0]),
            parse_quote!(([u8; 0], ())),
        ] {
            assert_eq!(classify_nonzero(&ty), NonZeroClass::No);
        }
        for ty in [
            parse_quote!(*const u8),
            parse_quote!(&u8),
            parse_quote!(fn()),
            parse_quote!((u8, *mut u8)),
            parse_quote!([*mut u8; 2]),
        ] {
            assert_eq!(classify_nonzero(&ty), NonZeroClass::Yes);
        }
        for ty in [
            parse_quote!(T),
            parse_quote!(((), T)),
            parse_quote!([u8; N]),
            parse_quote!(u8),
            parse_quote!(core::primitive::u8),
        ] {
            assert_eq!(classify_nonzero(&ty), NonZeroClass::Unknown);
        }
    }

    #[test]
    fn tuples_follow_the_final_element() {
        let generics = parse_quote!(<T: ?Sized, S>);
        for ty in [
            parse_quote!(()),
            parse_quote!((S,)),
            parse_quote!((u8, *const T)),
        ] {
            assert_eq!(classify_type(&ty, &generics), TypeClass::Sized);
        }
        for ty in [parse_quote!((T,)), parse_quote!((u8, (T,)))] {
            assert_eq!(
                classify_type(&ty, &generics),
                TypeClass::Unsized(UnsizedTail::Unknown)
            );
        }
        assert_eq!(
            classify_type(&parse_quote!((u8, [S])), &generics),
            TypeClass::Unsized(UnsizedTail::SliceLike)
        );
        assert_eq!(
            classify_type(&parse_quote!((u8, dyn Send)), &generics),
            TypeClass::Unsized(UnsizedTail::DynTraitLike)
        );
        assert_eq!(
            classify_type(&parse_quote!((u8, core::ffi::CStr)), &generics),
            TypeClass::Unknown
        );
        assert_eq!(
            classify_type(&parse_quote!((u8, SomeAlias)), &generics),
            TypeClass::Unknown
        );
    }

    #[test]
    fn type_shapes_follow_tail_and_bounds() {
        let input: syn::DeriveInput = parse_quote!(
            struct Example<T: ?Sized, S, U>
            where
                U: ?Sized,
            {
                tail: T,
            }
        );
        for ty in [
            parse_quote!(S),
            parse_quote!(*mut T),
            parse_quote!(&T),
            parse_quote!([S; 2]),
            parse_quote!(fn()),
        ] {
            assert_eq!(classify_type(&ty, &input.generics), TypeClass::Sized);
        }
        for ty in [parse_quote!(T), parse_quote!(U), parse_quote!((u8, T))] {
            assert_eq!(
                classify_type(&ty, &input.generics),
                TypeClass::Unsized(UnsizedTail::Unknown)
            );
        }
        assert_eq!(
            classify_type(&parse_quote!((u8, [S])), &input.generics),
            TypeClass::Unsized(UnsizedTail::SliceLike)
        );
        assert_eq!(
            classify_type(&parse_quote!(dyn Send), &input.generics),
            TypeClass::Unsized(UnsizedTail::DynTraitLike)
        );
        assert_eq!(
            classify_type(&parse_quote!(core::ffi::CStr), &input.generics),
            TypeClass::Unknown
        );
        assert_eq!(
            classify_type(&parse_quote!(core::marker::PhantomData<T>), &input.generics),
            TypeClass::Unknown
        );
        assert_eq!(
            classify_type(&parse_quote!(SomeAlias), &input.generics),
            TypeClass::Unknown
        );
        assert_eq!(
            classify_type(&parse_quote!(S::Associated), &input.generics),
            TypeClass::Unknown
        );
        let bounded: syn::DeriveInput = parse_quote!(
            struct B<T: ?Sized>
            where
                T: Sized,
            {
                tail: T,
            }
        );
        assert_eq!(
            classify_type(&parse_quote!(T), &bounded.generics),
            TypeClass::Sized
        );
        assert_eq!(
            classify_type(&parse_quote!(T), &input.generics),
            TypeClass::Unsized(UnsizedTail::Unknown)
        );
    }

    #[test]
    fn raw_pointer_fields_are_syntactic() {
        for ty in [
            parse_quote!(*mut u8),
            parse_quote!((*const u8, u8)),
            parse_quote!([*mut u8; 2]),
        ] {
            assert!(contains_raw_pointer(&ty));
        }
        for ty in [
            parse_quote!(&*const u8),
            parse_quote!(Option<*mut u8>),
            parse_quote!(core::marker::PhantomData<*mut u8>),
        ] {
            assert!(!contains_raw_pointer(&ty));
        }
    }
}
