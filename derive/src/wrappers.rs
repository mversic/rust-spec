//! Named constructors are expanded only on recursive paths. Every expansion
//! carries an exact type-identity proof in the caller's generic context.
use proc_macro2::TokenStream;
use quote::quote;
use syn::{GenericArgument, PathArguments, Type};

use crate::{
    axis::{self, Expr, Known},
    families::Axis,
};

#[derive(Clone, Copy)]
pub(crate) enum Wrapper {
    Box,
    Vec,
    VecDeque,
    Pin,
    NonNull,
    Option,
}

impl Wrapper {
    pub(crate) fn recognize(ty: &Type) -> Option<(Self, Vec<Type>)> {
        let Type::Path(path) = ty else {
            return None;
        };
        if path.qself.is_some() {
            return None;
        }
        let last = path.path.segments.last()?;
        let wrapper = match last.ident.to_string().as_str() {
            "Box" => Self::Box,
            "Vec" => Self::Vec,
            "VecDeque" => Self::VecDeque,
            "Pin" => Self::Pin,
            "NonNull" => Self::NonNull,
            "Option" => Self::Option,
            _ => return None,
        };
        let PathArguments::AngleBracketed(args) = &last.arguments else {
            return None;
        };
        let args = args
            .args
            .iter()
            .map(|a| match a {
                GenericArgument::Type(ty) => Some(ty.clone()),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        (args.len() == 1).then_some((wrapper, args))
    }

    pub(crate) fn recursive_use(
        ty: &Type,
        is_self: impl Fn(&Type) -> bool,
    ) -> Option<(Self, Vec<Type>)> {
        fn direct_indirection(ty: &Type, is_self: &impl Fn(&Type) -> bool) -> bool {
            match ty {
                Type::Paren(t) => direct_indirection(&t.elem, is_self),
                Type::Group(t) => direct_indirection(&t.elem, is_self),
                Type::Reference(t) => is_self(&t.elem),
                Type::Ptr(t) => is_self(&t.elem),
                _ => Wrapper::recognize(ty).is_some_and(|(wrapper, args)| {
                    !matches!(wrapper, Wrapper::Option | Wrapper::Pin) && is_self(&args[0])
                }),
            }
        }
        let (wrapper, args) = Self::recognize(ty)?;
        let supported = match wrapper {
            Self::Option => {
                direct_indirection(&args[0], &is_self)
                    || Self::recognize(&args[0]).is_some_and(|(inner, args)| {
                        matches!(inner, Self::Pin) && direct_indirection(&args[0], &is_self)
                    })
            }
            Self::Pin => direct_indirection(&args[0], &is_self),
            _ => is_self(&args[0]),
        };
        supported.then_some((wrapper, args))
    }

    fn canonical(self, args: &[Type]) -> TokenStream {
        let constructor = match self {
            Self::Box => quote!(RecursiveAlloc::boxed::Box),
            Self::Vec => quote!(RecursiveAlloc::vec::Vec),
            Self::VecDeque => quote!(RecursiveAlloc::collections::VecDeque),
            Self::Pin => quote!(::core::pin::Pin),
            Self::NonNull => quote!(::core::ptr::NonNull),
            Self::Option => quote!(::core::option::Option),
        };
        quote!(#constructor<#(#args),*>)
    }

    pub(crate) fn expression(self, args: &[Type], axis: Axis, generics: &syn::Generics) -> Expr {
        use Axis::*;
        use Known::*;
        let known = Expr::Known;
        let child = |a| axis::field(&args[0], a, generics);
        let needs_drop = |expr| Expr::combine(Drop, known(NoDrop), expr);
        let pointer_mutability = || Expr::pointer_mutability(child(Mutability));
        let pointer_layout = || Expr::pointer_layout(child(Size));
        let pointer_alignment =
            || Expr::Projection(syn::parse_quote!(::core::primitive::usize), Alignment);
        match self {
            Self::Box | Self::NonNull => match axis {
                Layout => Expr::combine(Layout, pointer_layout(), child(Layout)),
                Size => known(Nonzero),
                Alignment => pointer_alignment(),
                Trap => known(NonRobust),
                Niche => Expr::Niche(Box::new(pointer_layout())),
                Drop if matches!(self, Self::Box) => {
                    Expr::CustomDrop(Box::new(needs_drop(child(Drop))))
                }
                Drop => known(NoDrop),
                Mutability => pointer_mutability(),
                IndirectTrap => child(Trap),
            },
            Self::Vec | Self::VecDeque => match axis {
                Layout => known(Unstable),
                Size => known(Nonzero),
                Alignment => pointer_alignment(),
                Trap => known(NonRobust),
                Niche => Expr::Niche(Box::new(known(Unstable))),
                Drop => Expr::CustomDrop(Box::new(known(AutoDrop))),
                Mutability => pointer_mutability(),
                IndirectTrap => child(Trap),
            },
            Self::Pin => {
                if axis == Drop {
                    needs_drop(child(Drop))
                } else {
                    child(axis)
                }
            }
            Self::Option => match axis {
                Layout => {
                    Expr::combine(Layout, Expr::nullable(child(Niche), Layout), child(Layout))
                }
                Trap => Expr::combine(
                    Trap,
                    Expr::nullable(child(Niche), Trap),
                    child(IndirectTrap),
                ),
                Niche => Expr::nullable(child(Niche), Niche),
                Size => known(Nonzero),
                Drop => needs_drop(child(Drop)),
                _ => child(axis),
            },
        }
    }
}

#[derive(Default)]
pub(crate) struct Checks {
    entries: Vec<(Type, Wrapper, Vec<Type>)>,
}
impl Checks {
    pub(crate) fn add(&mut self, ty: Type, wrapper: Wrapper, args: Vec<Type>) {
        if !self.entries.iter().any(|e| e.0 == ty) {
            self.entries.push((ty, wrapper, args));
        }
    }
    pub(crate) fn emit(self, name: &syn::Ident, generics: &syn::Generics) -> TokenStream {
        if self.entries.is_empty() {
            return TokenStream::new();
        }
        let alloc = self
            .entries
            .iter()
            .any(|(_, w, _)| matches!(w, Wrapper::Box | Wrapper::Vec | Wrapper::VecDeque))
            .then(|| {
                quote!(
                    extern crate alloc as RecursiveAlloc;
                )
            });
        let assertions = self.entries.iter().map(|(actual, wrapper, args)| {
            let expected = wrapper.canonical(args);
            quote!(same_type::<#actual, #expected>();)
        });
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        quote! {
            const _: () = {
                #alloc
                trait SameType<T: ?::core::marker::Sized> {}
                impl<T: ?::core::marker::Sized> SameType<T> for T {}
                trait CheckRecursiveWrappers { fn check_recursive_wrappers(); }
                impl #impl_generics CheckRecursiveWrappers for #name #ty_generics #where_clause {
                    fn check_recursive_wrappers() {
                        fn same_type<A: ?::core::marker::Sized + SameType<B>, B: ?::core::marker::Sized>() {}
                        #(#assertions)*
                    }
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn recognizes_only_supported_recursive_links_and_containers() {
        let own: Type = parse_quote!(Node<T>);
        let self_type: Type = parse_quote!(Self);
        for ty in [
            parse_quote!(Box<Self>),
            parse_quote!(Vec<Node<T>>),
            parse_quote!(VecDeque<Node<T>>),
            parse_quote!(Pin<Box<Self>>),
            parse_quote!(Pin<&mut Self>),
            parse_quote!(Option<Pin<Box<Self>>>),
            parse_quote!(Option<VecDeque<Self>>),
            parse_quote!(core::ptr::NonNull<Self>),
            parse_quote!(Option<Box<Self>>),
            parse_quote!(Option<Vec<Node<T>>>),
            parse_quote!(Option<NonNull<Self>>),
            parse_quote!(Option<&Self>),
            parse_quote!(Option<*mut Node<T>>),
        ] {
            assert!(Wrapper::recursive_use(&ty, |t| t == &own || t == &self_type).is_some());
        }
        for ty in [
            parse_quote!(Box<T>),
            parse_quote!(VecDeque<T>),
            parse_quote!(Pin<T>),
            parse_quote!(Pin<Pin<Box<Self>>>),
            parse_quote!(Pin<Option<Box<Self>>>),
            parse_quote!(VecDeque<NonNull<Self>>),
            parse_quote!(Box<Node<u8>>),
            parse_quote!(Box<(Self, u32)>),
            parse_quote!(Vec<Option<Self>>),
            parse_quote!(Vec<NonNull<Self>>),
            parse_quote!(Option<Option<Box<Self>>>),
            parse_quote!(Option<Box<T>>),
            parse_quote!(ManuallyDrop<Box<Self>>),
            parse_quote!(MaybeUninit<Box<Self>>),
        ] {
            assert!(Wrapper::recursive_use(&ty, |t| t == &own || t == &self_type).is_none());
        }
    }
}
