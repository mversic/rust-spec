//! Recursive, axis-specific field analysis. Bounds are emitted only for the
//! expression that survives simplification.
use crate::{AggregateFamily, classify, crate_path, field_has_generic_params};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Generics, Type};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Axis {
    Layout,
    Size,
    Alignment,
    Trap,
    Niche,
    Drop,
    Mutability,
    IndirectTrap,
}
impl Axis {
    fn name(self) -> syn::Ident {
        format_ident!(
            "{}",
            match self {
                Self::Layout => "Layout",
                Self::Size => "Size",
                Self::Alignment => "__Alignment",
                Self::Trap => "Trap",
                Self::Niche => "Niche",
                Self::Drop => "Drop",
                Self::Mutability => "Mutability",
                Self::IndirectTrap => "__IndirectTrap",
            }
        )
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Known {
    Stable,
    Unstable,
    Zero,
    Nonzero,
    Slice,
    DynTrait,
    Robust,
    NonRobust,
    NoNiche,
    NoDrop,
    Exclusive,
    One,
}
impl Known {
    fn tokens(self) -> TokenStream {
        let c = crate_path();
        match self {
            Self::Stable => quote!(#c::Stable),
            Self::Unstable => quote!(#c::Unstable),
            Self::Zero => quote!(#c::size::Sized<#c::Zero>),
            Self::Nonzero => quote!(#c::size::Sized<#c::Gt<#c::Zero>>),
            Self::Slice => quote!(#c::size::MetaSized<#c::size::SliceLike>),
            Self::DynTrait => quote!(#c::size::MetaSized<#c::size::DynTraitLike>),
            Self::Robust => quote!(#c::layout::Robust),
            Self::NonRobust => quote!(#c::layout::NonRobust),
            Self::NoNiche => quote!(#c::niche::WithoutNiche),
            Self::NoDrop => quote!(#c::drop::NoDrop),
            Self::Exclusive => quote!(#c::mutability::Exclusive<#c::mutability::WithoutInterior>),
            Self::One => quote!(#c::One),
        }
    }
}
#[derive(Clone)]
pub(crate) enum Expr {
    Known(Known),
    Projection(Type, Axis),
    Binary(Axis, Box<Expr>, Box<Expr>),
    SizedField(Box<Expr>, Type),
    PointerLayout(Box<Expr>),
    PointerMutability(Box<Expr>),
    Niche(Box<Expr>),
}
impl Expr {
    fn known(k: Known) -> Self {
        Self::Known(k)
    }
    fn is(&self, k: Known) -> bool {
        matches!(self, Self::Known(v) if *v == k)
    }
    pub(crate) fn combine(axis: Axis, left: Self, right: Self) -> Self {
        use Known::*;
        if matches!(axis, Axis::Drop | Axis::Niche) {
            let identity = if axis == Axis::Drop { NoDrop } else { NoNiche };
            if left.is(identity) {
                if let Self::Binary(inner_axis, inner_left, _) = &right {
                    if *inner_axis == axis && inner_left.is(identity) {
                        return right;
                    }
                }
            }
        }
        match axis {
            Axis::Layout => {
                if left.is(Unstable) || right.is(Unstable) {
                    return Self::known(Unstable);
                }
                if left.is(Stable) {
                    return right;
                }
                if right.is(Stable) {
                    return left;
                }
            }
            Axis::Trap | Axis::IndirectTrap => {
                if left.is(NonRobust) || right.is(NonRobust) {
                    return Self::known(NonRobust);
                }
                if left.is(Robust) {
                    return right;
                }
                if right.is(Robust) {
                    return left;
                }
            }
            Axis::Size => {
                if left.is(Slice) || right.is(Slice) {
                    return Self::known(Slice);
                }
                if left.is(Zero) {
                    return right;
                }
                if right.is(Zero) {
                    return left;
                }
                if left.is(Nonzero) && right.is(Nonzero) {
                    return left;
                }
            }
            Axis::Alignment => {
                if left.is(One) {
                    return right;
                }
                if right.is(One) {
                    return left;
                }
            }
            Axis::Drop if left.is(NoDrop) && right.is(NoDrop) => return left,
            Axis::Niche if matches!(&left, Self::Niche(_)) || matches!(&right, Self::Niche(_)) => {
                return Self::Niche(Box::new(Self::known(Unstable)));
            }
            Axis::Niche if left.is(NoNiche) && right.is(NoNiche) => return left,
            Axis::Mutability => {
                if left.is(Exclusive) {
                    return right;
                }
                if right.is(Exclusive) {
                    return left;
                }
            }
            _ => {}
        }
        Self::Binary(axis, Box::new(left), Box::new(right))
    }
    fn has_params(&self, generics: &Generics) -> bool {
        match self {
            Self::Known(_) => false,
            Self::Projection(ty, _) => field_has_generic_params(ty, generics),
            Self::Binary(_, l, r) => l.has_params(generics) || r.has_params(generics),
            Self::SizedField(inner, ty) => {
                inner.has_params(generics) || field_has_generic_params(ty, generics)
            }
            Self::PointerLayout(x) | Self::PointerMutability(x) | Self::Niche(x) => {
                x.has_params(generics)
            }
        }
    }
    pub(crate) fn emit(self, generics: &Generics) -> AggregateFamily {
        let c = crate_path();
        match self {
            Self::Known(k) => AggregateFamily::fixed(k.tokens()),
            Self::Projection(ty, axis) => {
                let name = axis.name();
                let mut family = AggregateFamily::fixed(quote!(<#ty as #c::RustSpec>::#name));
                if field_has_generic_params(&ty, generics) {
                    family.aggregate_bounds.push(quote!(#ty: #c::RustSpec));
                    if axis == Axis::Drop {
                        family
                            .aggregate_bounds
                            .push(quote!(<#ty as #c::RustSpec>::Drop: #c::drop::DropKind));
                    }
                    if axis == Axis::Niche {
                        family.aggregate_bounds.push(quote!(#c::niche::WithoutNiche: ::core::ops::Add<<#ty as #c::RustSpec>::Niche>));
                    }
                }
                family
            }
            Self::Binary(axis, left, right) => {
                let mut left = left.emit(generics);
                let right = right.emit(generics);
                let operator = match axis {
                    Axis::Alignment => quote!(#c::Max),
                    Axis::Drop => quote!(CombineDrop),
                    _ => quote!(::core::ops::Add),
                };
                let l = left.kind;
                let r = right.kind;
                left.aggregate_bounds.extend(right.aggregate_bounds);
                // Concrete operations need no where predicate. Generic operations
                // do, including operations whose operands are nested expressions.
                if !left.aggregate_bounds.is_empty() {
                    left.aggregate_bounds.push(quote!(#l: #operator<#r>));
                }
                left.kind = quote!(<#l as #operator<#r>>::Output);
                left
            }
            Self::SizedField(inner, ty) => {
                let mut family = inner.emit(generics);
                let kind = family.kind;
                if field_has_generic_params(&ty, generics) || !family.aggregate_bounds.is_empty() {
                    family
                        .aggregate_bounds
                        .push(quote!(#kind: AddSizedField<#ty>));
                }
                family.kind = quote!(<#kind as AddSizedField<#ty>>::Output);
                family
            }
            Self::PointerLayout(inner) => {
                emit_unary(*inner, generics, Some(quote!(#c::size::PointerLayout)))
            }
            Self::PointerMutability(inner) => emit_unary(
                *inner,
                generics,
                Some(quote!(#c::mutability::PointerMutability)),
            ),
            Self::Niche(inner) => emit_unary(*inner, generics, None),
        }
    }
}

fn emit_unary(inner: Expr, generics: &Generics, operator: Option<TokenStream>) -> AggregateFamily {
    let c = crate_path();
    let mut family = inner.emit(generics);
    let kind = family.kind;
    if let Some(operator) = operator {
        if !family.aggregate_bounds.is_empty() {
            family.aggregate_bounds.push(quote!(#kind: #operator));
        }
        family.kind = quote!(<#kind as #operator>::Output);
    } else {
        family.kind = quote!(#c::niche::WithNiche<#kind>);
    }
    family
}

pub(crate) fn aggregate(axis: Axis, fields: &[&Type], generics: &Generics) -> Option<Expr> {
    // Earlier fields are sized by Rust's well-formedness rules. Only the tail
    // can prevent a nonzero witness from settling the size of the aggregate.
    if axis == Axis::Size {
        if let Some(tail) = fields.last() {
            if classify::classify_type(tail, generics) == classify::TypeClass::Sized
                && fields
                    .iter()
                    .any(|t| classify::classify_nonzero(t) == classify::NonZeroClass::Yes)
            {
                return Some(Expr::known(Known::Nonzero));
            }
        }
    }
    if axis == Axis::Size {
        let (tail, _) = fields.split_last()?;
        // Once the tail is known to be sized, any field may be the seed.
        // Prefer a concrete one so rustc can discard generic size dependencies.
        let seed = if classify::classify_type(tail, generics) == classify::TypeClass::Sized {
            fields
                .iter()
                .rposition(|ty| !field_has_generic_params(ty, generics))
                .unwrap_or(fields.len() - 1)
        } else {
            fields.len() - 1
        };
        let mut size = field(fields[seed], axis, generics);
        for ty in fields
            .iter()
            .enumerate()
            .rev()
            .filter_map(|(i, ty)| (i != seed).then_some(ty))
        {
            if size.is(Known::Nonzero) || size.is(Known::Slice) || size.is(Known::DynTrait) {
                break;
            }
            size = if size.is(Known::Zero) {
                field(ty, axis, generics)
            } else {
                Expr::SizedField(Box::new(size), (*ty).clone())
            };
        }
        return Some(size);
    }
    let (concrete, generic): (Vec<_>, Vec<_>) = fields
        .iter()
        .map(|ty| field(ty, axis, generics))
        .partition(|expr| !expr.has_params(generics));
    let fold = |items: Vec<Expr>| items.into_iter().reduce(|l, r| Expr::combine(axis, l, r));
    match (fold(concrete), fold(generic)) {
        (Some(l), Some(r)) => Some(Expr::combine(axis, l, r)),
        (Some(x), None) | (None, Some(x)) => Some(x),
        (None, None) => None,
    }
}

pub(crate) fn field(ty: &Type, axis: Axis, generics: &Generics) -> Expr {
    use Axis::*;
    use Known::*;
    let known = Expr::known;
    let projection = || Expr::Projection(ty.clone(), axis);
    match ty {
        Type::Paren(t) => field(&t.elem, axis, generics),
        Type::Group(t) => field(&t.elem, axis, generics),
        Type::Tuple(t) => {
            if t.elems.is_empty() {
                return known(match axis {
                    Layout => Stable,
                    Size => Zero,
                    Alignment => One,
                    Trap | IndirectTrap => Robust,
                    Niche => NoNiche,
                    Drop => NoDrop,
                    Mutability => Exclusive,
                });
            }
            if axis == Layout {
                return known(Unstable);
            }
            let fields: Vec<_> = t.elems.iter().collect();
            let inner = aggregate(axis, &fields, generics).unwrap();
            if axis == Drop {
                Expr::combine(axis, known(NoDrop), inner)
            } else {
                inner
            }
        }
        Type::Array(t) => {
            let length = match &t.len {
                syn::Expr::Lit(l) => match &l.lit {
                    syn::Lit::Int(n) => n.base10_parse::<usize>().ok(),
                    _ => None,
                },
                _ => None,
            };
            if axis == Layout || axis == Alignment {
                return field(&t.elem, axis, generics);
            }
            match length {
                Some(0) => known(match axis {
                    Size => Zero,
                    Trap | IndirectTrap => Robust,
                    Niche => NoNiche,
                    Drop => NoDrop,
                    Mutability => Exclusive,
                    _ => unreachable!(),
                }),
                Some(_) => {
                    let inner = field(&t.elem, axis, generics);
                    match axis {
                        Drop => Expr::combine(axis, known(NoDrop), inner),
                        Niche => Expr::combine(axis, known(NoNiche), inner),
                        _ => inner,
                    }
                }
                None => projection(),
            }
        }
        Type::Slice(t) => match axis {
            Size => known(Slice),
            Niche => known(NoNiche),
            Drop => Expr::combine(Drop, known(NoDrop), field(&t.elem, Drop, generics)),
            _ => field(&t.elem, axis, generics),
        },
        Type::FnPtr(function) => match axis {
            Size => known(Nonzero),
            Alignment => Expr::Projection(syn::parse_quote!(::core::primitive::usize), Alignment),
            Trap => known(NonRobust),
            Niche => Expr::Niche(Box::new(known(Stable))),
            Drop => known(NoDrop),
            Mutability => known(Exclusive),
            IndirectTrap => known(Robust),
            Layout
                if function.abi.as_ref().is_none_or(|abi| {
                    abi.name.as_ref().is_some_and(|name| name.value() == "Rust")
                }) =>
            {
                known(Unstable)
            }
            // Foreign ABI layout depends on the signature. Keep the whole
            // projection so bound lifetimes stay inside the function type.
            Layout => projection(),
        },
        Type::TraitObject(_) if axis == Size => known(DynTrait),
        Type::Reference(_) | Type::Ptr(_) => {
            let (pointee, reference) = match ty {
                Type::Reference(t) => (&*t.elem, true),
                Type::Ptr(t) => (&*t.elem, false),
                _ => unreachable!(),
            };
            match axis {
                Size => known(Nonzero),
                Alignment => {
                    Expr::Projection(syn::parse_quote!(::core::primitive::usize), Alignment)
                }
                Drop => known(NoDrop),
                Trap if reference => known(NonRobust),
                Trap | IndirectTrap => field(pointee, Trap, generics),
                Niche if !reference => known(NoNiche),
                Niche => Expr::Niche(Box::new(pointer_layout(pointee, generics))),
                Layout => Expr::combine(
                    Layout,
                    pointer_layout(pointee, generics),
                    field(pointee, Layout, generics),
                ),
                Mutability => {
                    let inner = field(pointee, axis, generics);
                    if inner.is(Exclusive) {
                        inner
                    } else {
                        Expr::PointerMutability(Box::new(inner))
                    }
                }
            }
        }
        Type::Path(_) | Type::Macro(_) | Type::Verbatim(_) => projection(),
        // These are not ordinary supported field types; leave diagnostics and
        // any future RustSpec implementations to the compiler.
        Type::ImplTrait(_) | Type::Infer(_) | Type::Never(_) => projection(),
        _ => projection(),
    }
}
fn pointer_layout(ty: &Type, generics: &Generics) -> Expr {
    match classify::classify_type(ty, generics) {
        classify::TypeClass::Sized => Expr::known(Known::Stable),
        classify::TypeClass::Unsized(
            classify::UnsizedTail::SliceLike | classify::UnsizedTail::DynTraitLike,
        ) => Expr::known(Known::Unstable),
        _ => Expr::PointerLayout(Box::new(field(ty, Axis::Size, generics))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn nested_syntax_discards_unneeded_axis_dependencies() {
        let generics = parse_quote!(<'a, T, U>);
        for (ty, axis, expected) in [
            (parse_quote!([(&'a T, U); 2]), Axis::Layout, Known::Unstable),
            (parse_quote!([(&'a T, U); 2]), Axis::Trap, Known::NonRobust),
            (parse_quote!(([T; 0], &'a U)), Axis::Drop, Known::NoDrop),
            (parse_quote!([(&'a T, U); 0]), Axis::Size, Known::Zero),
            (parse_quote!([(&'a T, U); 0]), Axis::Trap, Known::Robust),
            (
                parse_quote!([(&'a T, U); 0]),
                Axis::Mutability,
                Known::Exclusive,
            ),
            (parse_quote!((T, (&'a U, ()))), Axis::Size, Known::Nonzero),
            (parse_quote!(((&'a T,), [U])), Axis::Size, Known::Slice),
        ] {
            let family = field(&ty, axis, &generics).emit(&generics);
            assert_eq!(family.kind.to_string(), expected.tokens().to_string());
            assert!(family.aggregate_bounds.is_empty());
        }
    }

    #[test]
    fn aggregate_absorbers_discard_other_fields_and_their_bounds() {
        let generics = parse_quote!(<T, U>);
        let unknown = parse_quote!(T);
        let tuple = parse_quote!([(&U,); 2]);
        for (axis, expected) in [
            (Axis::Layout, Known::Unstable),
            (Axis::Trap, Known::NonRobust),
        ] {
            let family = aggregate(axis, &[&unknown, &tuple], &generics)
                .unwrap()
                .emit(&generics);
            assert_eq!(family.kind.to_string(), expected.tokens().to_string());
            assert!(family.aggregate_bounds.is_empty());
        }
    }

    #[test]
    fn function_pointer_axes_do_not_depend_on_signature_types() {
        let generics = parse_quote!(<T>);
        for ty in [
            parse_quote!(fn(T) -> T),
            parse_quote!(unsafe extern "C" fn(T) -> T),
            parse_quote!(for<'a> fn(&'a T) -> &'a T),
        ] {
            for (axis, expected) in [
                (Axis::Size, Known::Nonzero),
                (Axis::Drop, Known::NoDrop),
                (Axis::Trap, Known::NonRobust),
                (Axis::Mutability, Known::Exclusive),
                (Axis::IndirectTrap, Known::Robust),
            ] {
                let family = field(&ty, axis, &generics).emit(&generics);
                assert_eq!(family.kind.to_string(), expected.tokens().to_string());
                assert!(family.aggregate_bounds.is_empty());
            }
        }
        let rust =
            field(&parse_quote!(extern "Rust" fn(T)), Axis::Layout, &generics).emit(&generics);
        assert_eq!(rust.kind.to_string(), Known::Unstable.tokens().to_string());
        assert!(rust.aggregate_bounds.is_empty());
        let foreign =
            field(&parse_quote!(extern "C" fn(T)), Axis::Layout, &generics).emit(&generics);
        assert!(!foreign.aggregate_bounds.is_empty());
    }

    #[test]
    fn trait_object_tail_determines_size() {
        let generics = parse_quote!(<T>);
        let family = field(
            &parse_quote!((T, dyn ::core::fmt::Debug)),
            Axis::Size,
            &generics,
        )
        .emit(&generics);
        assert_eq!(
            family.kind.to_string(),
            Known::DynTrait.tokens().to_string()
        );
        assert!(family.aggregate_bounds.is_empty());
    }

    #[test]
    fn zero_arrays_keep_alignment_and_layout_but_unknown_lengths_keep_projections() {
        let generics = parse_quote!(<T, const N: usize>);
        for axis in [Axis::Alignment, Axis::Layout] {
            let family = field(&parse_quote!([T; 0]), axis, &generics).emit(&generics);
            assert!(!family.aggregate_bounds.is_empty());
            assert!(family.kind.to_string().contains("T as"));
        }
        for axis in [
            Axis::Size,
            Axis::Trap,
            Axis::Niche,
            Axis::Drop,
            Axis::Mutability,
        ] {
            let family = field(&parse_quote!([T; N]), axis, &generics).emit(&generics);
            assert!(family.kind.to_string().contains("[T ; N] as"));
        }
    }
}

/// Block-scoped public traits can appear in the public implementation's
/// associated types, but their names cannot be reached outside the anonymous
/// constant. Each expansion owns its helpers and introduces no module names.
pub(crate) fn helpers() -> TokenStream {
    let c = crate_path();
    quote! {
        pub trait CombineDrop<Rhs> {
            type Output: #c::drop::DropKind;
        }
        impl<L, R> CombineDrop<R> for L
        where
            L: ::core::ops::Add<R>,
            <L as ::core::ops::Add<R>>::Output: #c::drop::DropKind,
        {
            type Output = <L as ::core::ops::Add<R>>::Output;
        }

        pub trait AddSizedField<T> {
            type Output;
        }
        impl<T> AddSizedField<T> for #c::size::Sized<#c::Gt<#c::Zero>> {
            type Output = Self;
        }
        impl<T: #c::RustSpec> AddSizedField<T> for #c::size::Sized<#c::Zero> {
            type Output = T::Size;
        }
        impl<T, K: #c::size::MetadataKind> AddSizedField<T> for #c::size::MetaSized<K> {
            type Output = Self;
        }
        impl<T> AddSizedField<T> for #c::size::ExternTypeLike {
            type Output = Self;
        }
        impl<T> AddSizedField<T> for #c::size::NulTerminated {
            type Output = Self;
        }
    }
}
