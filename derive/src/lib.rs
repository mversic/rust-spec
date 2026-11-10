use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Token, punctuated::Punctuated};

use crate::{
    axis::Expr,
    families::{Axis, Families},
    repr::{EnumRepr, EnumTag, ReprKind, parse_repr},
};

mod axis;
mod classify;
mod families;
mod repr;
mod wrappers;

/// Derives `RustSpec` for a struct, enum, or union.
///
/// Add `#[rust_spec(custom_drop)]` if the type itself implements [`Drop`].
/// Add `#[rust_spec(custom_niche)]` if a struct has a custom niche.
/// Add `#[rust_spec(non_robust)]` if the type has an additional validity invariant.
/// Add `#[rust_spec(opaque)]` to classify the type as `Opacity = Opaque`
///
/// # Safety hazards of private invariants
///
/// The derive infers `RustSpec` axes from the fields. It cannot see invariants enforced through
/// private constructors.
///
/// For example, if an invariant of `#[repr(C)] Wrapper(u8)` is that it may never hold `u8::MAX`,
/// `RustSpec` will infer `Layout = Stable` and `Trap = Robust` which downstream consumers may
/// interpret as a permission to transmute `u8` into `Wrapper` and cause UB.
///
///Likewise, be very careful when the inner fields are raw pointers, `NonNull<T>` or `MaybeUninit<T>`.
/// `UnsafeCell<T>` and `ManuallyDrop<T>` fields, however, don't require special care from the user.
#[proc_macro_derive(RustSpec, attributes(rust_spec))]
pub fn rust_spec(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn crate_path() -> proc_macro2::TokenStream {
    let rust_spec = proc_macro_crate::crate_name("rust-spec");
    match rust_spec {
        Ok(proc_macro_crate::FoundCrate::Itself) => return quote! { rust_spec },
        Ok(proc_macro_crate::FoundCrate::Name(name)) => {
            let name = format_ident!("{}", name);
            return quote! { #name };
        }
        Err(_) => {}
    }

    match proc_macro_crate::crate_name("co3") {
        Ok(proc_macro_crate::FoundCrate::Itself) => quote! { crate::rust_spec },
        Ok(proc_macro_crate::FoundCrate::Name(name)) => {
            let name = format_ident!("{}", name);
            quote! { #name::rust_spec }
        }
        Err(_) => quote! { co3::rust_spec },
    }
}

struct EmittedFamily {
    kind: proc_macro2::TokenStream,
    aggregate_bounds: Vec<proc_macro2::TokenStream>,
    // Surviving type/const parameters, independent of the predicates emitted so far.
    depends_on_generics: bool,
}

impl EmittedFamily {
    fn fixed(kind: proc_macro2::TokenStream) -> Self {
        Self {
            kind,
            aggregate_bounds: Vec::new(),
            depends_on_generics: false,
        }
    }
}

fn field_has_generic_params(ty: &syn::Type, generics: &syn::Generics) -> bool {
    use syn::visit::Visit;

    struct Visitor<'a> {
        type_params: Vec<&'a syn::Ident>,
        const_params: Vec<&'a syn::Ident>,
        found: bool,
    }

    impl<'a> Visitor<'a> {
        fn new(generics: &'a syn::Generics) -> Self {
            Self {
                type_params: generics.type_params().map(|p| &p.ident).collect(),
                const_params: generics.const_params().map(|p| &p.ident).collect(),
                found: false,
            }
        }
    }

    impl syn::visit::Visit<'_> for Visitor<'_> {
        fn visit_expr_path(&mut self, path: &syn::ExprPath) {
            if path.qself.is_none()
                && let Some(first) = path.path.segments.first()
                && self.const_params.contains(&&first.ident)
            {
                self.found = true;
            }
            syn::visit::visit_expr_path(self, path);
        }

        fn visit_type_path(&mut self, type_path: &syn::TypePath) {
            if type_path.qself.is_none()
                && let Some(first_segment) = type_path.path.segments.first()
                && (self.type_params.contains(&&first_segment.ident)
                    || self.const_params.contains(&&first_segment.ident))
            {
                self.found = true;
            }

            syn::visit::visit_type_path(self, type_path);
        }
    }

    let mut visitor = Visitor::new(generics);
    visitor.visit_type(ty);
    visitor.found
}

fn expand(input: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let attrs = parse_rust_spec_attrs(&input.attrs)?;
    let repr = parse_repr(&input.attrs)?;
    let alignment = repr.align;
    let repr = repr.kind.as_ref();
    let name = &input.ident;
    let generics = &input.generics;
    if attrs.opaque && attrs.custom_niche {
        return Err(syn::Error::new_spanned(
            name,
            "`custom_niche` cannot be combined with `opaque`",
        ));
    }
    if attrs.opaque && attrs.non_robust {
        return Err(syn::Error::new_spanned(
            name,
            "`non_robust` cannot be combined with `opaque`",
        ));
    }
    if attrs.opaque && attrs.custom_drop {
        return Err(syn::Error::new_spanned(
            name,
            "`custom_drop` cannot be combined with `opaque`",
        ));
    }
    if attrs.opaque {
        let crate_ = crate_path();
        let mut rust_spec_generics = generics.clone();
        rust_spec_generics
            .make_where_clause()
            .predicates
            .push(syn::parse_quote!(Self: ::core::marker::Sized));
        let (impl_generics, ty_generics, where_clause) = rust_spec_generics.split_for_impl();
        return Ok(quote! {
            unsafe impl #impl_generics #crate_::RustSpec for #name #ty_generics #where_clause {
                type Layout = #crate_::Stable;
                type Size = #crate_::size::Sized<#crate_::Gt<#crate_::Zero>>;
                type __Alignment = #crate_::One;
                type Trap = #crate_::layout::Robust;
                type Niche = #crate_::niche::WithoutNiche;
                type Drop = #crate_::drop::NoDrop;
                type Mutability = #crate_::mutability::Exclusive<#crate_::mutability::WithoutInterior>;
                type Opacity = #crate_::Opaque;
                type __IndirectTrap = #crate_::layout::Robust;
            }
        });
    }
    if attrs.custom_niche && !matches!(&input.data, syn::Data::Struct(_)) {
        return Err(syn::Error::new_spanned(
            name,
            "`custom_niche` is only supported on structs",
        ));
    }
    let mut discriminant_assert = None;
    let rust_spec_impl = match &input.data {
        syn::Data::Struct(data) => {
            gen_struct_impl(repr, alignment, name, generics, &data.fields, attrs)
        }
        syn::Data::Enum(data) => {
            let enum_repr = EnumRepr::analyze(repr, &data.variants)?;
            discriminant_assert = enum_repr
                .discriminant_check
                .as_ref()
                .map(|tag| gen_discriminant_assert(name, &data.variants, tag));
            gen_enum_impl(
                repr,
                alignment,
                name,
                generics,
                &data.variants,
                &enum_repr.tags,
                attrs,
            )
        }
        syn::Data::Union(data) if matches!(repr, Some(ReprKind::Transparent)) => {
            let fields = data
                .fields
                .named
                .iter()
                .map(|field| &field.ty)
                .collect::<Vec<_>>();

            gen_struct_fields_impl(
                Some(&ReprKind::Transparent),
                alignment,
                name,
                generics,
                &fields,
                attrs,
                gen_mutability_family(generics, &fields),
            )
        }
        syn::Data::Union(data) => {
            gen_union_impl(repr, alignment, name, generics, &data.fields, attrs)
        }
    };

    let drop_assert = gen_drop_assert(name, generics, attrs.custom_drop);
    let helpers = axis::helpers();
    let sized_assertions = classify::sized_assertions(generics);
    Ok(quote! {
        #discriminant_assert
        #drop_assert
        #sized_assertions
        const _: () = {
            #helpers
            #rust_spec_impl
        };
    })
}

/// Re-evaluate unresolved discriminants in rustc, including implicit increments.
/// A default-representation discriminant cannot depend on generic parameters.
fn gen_discriminant_assert(
    name: &syn::Ident,
    variants: &Punctuated<syn::Variant, Token![,]>,
    tag: &syn::Type,
) -> proc_macro2::TokenStream {
    // Rewrite paths, not macro input (e.g. `stringify!(Self)` must stay unchanged).
    struct QualifySelf<'a>(&'a syn::Ident);
    impl syn::visit_mut::VisitMut for QualifySelf<'_> {
        // Nested items have their own Self context.
        fn visit_item_mut(&mut self, _: &mut syn::Item) {}

        fn visit_path_mut(&mut self, path: &mut syn::Path) {
            if let Some(first) = path.segments.first_mut()
                && first.ident == "Self"
            {
                first.ident = self.0.clone();
            }
            syn::visit_mut::visit_path_mut(self, path);
        }
    }
    let mut base = quote! { 0 };
    let mut base_index = 0usize;
    let checks = variants.iter().enumerate().map(|(index, variant)| {
        if let Some((_, expr)) = &variant.discriminant {
            let mut expr = expr.clone();
            syn::visit_mut::VisitMut::visit_expr_mut(&mut QualifySelf(name), &mut expr);
            base = quote! { #expr };
            base_index = index;
        }
        let offset = index - base_index;
        // Supply the original expected type without introducing local bindings
        // that could shadow names used by a discriminant expression.
        let value =
            quote! { (::core::convert::identity::<isize>(#base) as i128 + #offset as i128) };
        quote! {
            assert!(
                #value >= <#tag>::MIN as i128 && #value <= <#tag>::MAX as i128,
                concat!("enum discriminant does not fit inferred ", stringify!(#tag),
                    "; add an explicit integer #[repr(...)]"),
            );
        }
    });
    quote! { const _: () = { #(#checks)* }; }
}

fn gen_drop_assert(
    name: &syn::Ident,
    generics: &syn::Generics,
    custom_drop: bool,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    if custom_drop {
        quote! {
            const _: () = {
                trait AssertHasDrop { fn check(); }
                impl #impl_generics AssertHasDrop for #name #ty_generics #where_clause {
                    fn check() {
                        fn require_drop<T: ::core::ops::Drop>() {}
                        let _ = require_drop::<#name #ty_generics>;
                    }
                }
            };
        }
    } else {
        quote! {
            const _: () = {
                trait AssertNoDrop { fn check(); }
                impl #impl_generics AssertNoDrop for #name #ty_generics #where_clause {
                    fn check() {
                        trait DropImplIsNotSupported<A> { fn check() {} }
                        impl<T: ?::core::marker::Sized> DropImplIsNotSupported<()> for T {}
                        struct DropDetected;
                        impl<T: ?::core::marker::Sized + ::core::ops::Drop> DropImplIsNotSupported<DropDetected> for T {}
                        let _ = <#name #ty_generics as DropImplIsNotSupported<_>>::check;
                    }
                }
            };
        }
    }
}

#[derive(Clone, Copy, Default)]
struct RustSpecAttrs {
    custom_niche: bool,
    non_robust: bool,
    custom_drop: bool,
    opaque: bool,
}

fn parse_rust_spec_attrs(attrs: &[syn::Attribute]) -> syn::Result<RustSpecAttrs> {
    let mut result = RustSpecAttrs::default();

    for attr in attrs
        .iter()
        .filter(|attr| attr.path().is_ident("rust_spec"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("opaque") {
                if result.opaque {
                    return Err(meta.error("duplicate `opaque` within attribute"));
                }
                result.opaque = true;
                return Ok(());
            }
            if meta.path.is_ident("custom_niche") {
                if result.custom_niche {
                    return Err(meta.error("duplicate `custom_niche` within attribute"));
                }
                result.custom_niche = true;
                return Ok(());
            }
            if meta.path.is_ident("non_robust") {
                if result.non_robust {
                    return Err(meta.error("duplicate `non_robust` within attribute"));
                }
                result.non_robust = true;
                return Ok(());
            }
            if meta.path.is_ident("custom_drop") {
                if result.custom_drop {
                    return Err(meta.error("duplicate `custom_drop` within attribute"));
                }
                result.custom_drop = true;
                return Ok(());
            }
            Err(meta.error("unknown rust_spec attribute"))
        })?;
    }

    Ok(result)
}

fn field_types(fields: &syn::Fields) -> Vec<&syn::Type> {
    fields.iter().map(|field| &field.ty).collect()
}

fn variant_field_types(variants: &Punctuated<syn::Variant, Token![,]>) -> Vec<&syn::Type> {
    variants
        .iter()
        .flat_map(|variant| field_types(&variant.fields))
        .collect()
}

fn gen_struct_impl(
    repr: Option<&ReprKind>,
    repr_alignment: Option<usize>,
    name: &syn::Ident,
    generics: &syn::Generics,
    fields: &syn::Fields,
    attrs: RustSpecAttrs,
) -> proc_macro2::TokenStream {
    let fields = field_types(fields);

    let mutability = gen_mutability_family(generics, &fields);
    gen_struct_fields_impl(
        repr,
        repr_alignment,
        name,
        generics,
        &fields,
        attrs,
        mutability,
    )
}

fn gen_struct_fields_impl(
    repr: Option<&ReprKind>,
    alignment: Option<usize>,
    name: &syn::Ident,
    generics: &syn::Generics,
    fields: &[&syn::Type],
    attrs: RustSpecAttrs,
    mutability: Expr,
) -> proc_macro2::TokenStream {
    let layout = if repr.is_some() {
        gen_stable_layout_family(generics, fields)
    } else {
        gen_unstable_layout_family()
    };
    let size = gen_size_family(generics, fields);
    let alignment = apply_repr_alignment(gen_alignment_family(generics, fields, None), alignment);
    let trap = if attrs.custom_niche || attrs.non_robust {
        Expr::Known(axis::Known::NonRobust)
    } else {
        gen_trap_family(generics, fields, false)
    };
    let niche = if attrs.custom_niche {
        Expr::Niche(Box::new(Expr::Known(axis::Known::Unstable)))
    } else if let Some(ReprKind::Transparent) = repr {
        gen_transparent_niche_family(generics, fields)
    } else {
        gen_niche_family(generics, fields)
    };

    let drop = gen_drop_family(generics, fields, attrs.custom_drop);
    let indirect_trap = gen_indirect_trap_family(generics, fields);

    let spec = Families {
        layout,
        size,
        alignment,
        trap,
        niche,
        drop,
        mutability,
        indirect_trap,
    };

    gen_type_spec_impl(name, generics, spec, quote! {})
}

fn gen_enum_impl(
    repr: Option<&ReprKind>,
    repr_alignment: Option<usize>,
    name: &syn::Ident,
    generics: &syn::Generics,
    variants: &Punctuated<syn::Variant, Token![,]>,
    tags: &[EnumTag],
    attrs: RustSpecAttrs,
) -> proc_macro2::TokenStream {
    let fields = variant_field_types(variants);
    if tags.is_empty() {
        return gen_struct_fields_impl(
            repr,
            repr_alignment,
            name,
            generics,
            &fields,
            attrs,
            gen_mutability_family(generics, &fields),
        );
    }

    tags.iter()
        .map(|tag| {
            let spec = Families {
                layout: if repr.is_some() {
                    gen_stable_layout_family(generics, &fields)
                } else {
                    gen_unstable_layout_family()
                },
                size: Expr::Known(axis::Known::Nonzero),
                alignment: apply_repr_alignment(
                    gen_alignment_family(generics, &fields, Some(&tag.ty)),
                    repr_alignment,
                ),
                trap: gen_trap_family(generics, &fields, tag.has_trap_values || attrs.non_robust),
                niche: gen_enum_niche_family(tag.has_trap_values),
                drop: gen_drop_family(generics, &fields, attrs.custom_drop),
                mutability: gen_mutability_family(generics, &fields),
                indirect_trap: gen_indirect_trap_family(generics, &fields),
            };

            gen_type_spec_impl(name, generics, spec, tag.cfg.clone())
        })
        .collect()
}

fn gen_union_impl(
    repr: Option<&ReprKind>,
    repr_alignment: Option<usize>,
    name: &syn::Ident,
    generics: &syn::Generics,
    fields: &syn::FieldsNamed,
    attrs: RustSpecAttrs,
) -> proc_macro2::TokenStream {
    let fields = fields
        .named
        .iter()
        .map(|field| &field.ty)
        .collect::<Vec<_>>();

    let layout = if repr.is_some() {
        gen_stable_layout_family(generics, &fields)
    } else {
        gen_unstable_layout_family()
    };
    let size = gen_size_family(generics, &fields);
    let alignment = apply_repr_alignment(
        gen_alignment_family(generics, &fields, None),
        repr_alignment,
    );
    let trap = Expr::Known(if attrs.non_robust {
        axis::Known::NonRobust
    } else {
        axis::Known::Robust
    });
    let niche = Expr::Known(axis::Known::NoNiche);
    let drop = gen_drop_family(generics, &fields, attrs.custom_drop);

    let mutability = gen_mutability_family(generics, &fields);
    let indirect_trap = Expr::Known(axis::Known::Robust);

    let spec = Families {
        layout,
        size,
        alignment,
        trap,
        niche,
        drop,
        mutability,
        indirect_trap,
    };

    gen_type_spec_impl(name, generics, spec, quote! {})
}

fn gen_unstable_layout_family() -> Expr {
    Expr::Known(axis::Known::Unstable)
}

fn gen_stable_layout_family(generics: &syn::Generics, fields: &[&syn::Type]) -> Expr {
    Expr::aggregate(Axis::Layout, generics, fields).unwrap_or(Expr::Known(axis::Known::Stable))
}

fn gen_indirect_trap_family(generics: &syn::Generics, fields: &[&syn::Type]) -> Expr {
    Expr::aggregate(Axis::IndirectTrap, generics, fields)
        .unwrap_or(Expr::Known(axis::Known::Robust))
}

fn gen_drop_family(generics: &syn::Generics, fields: &[&syn::Type], custom_drop: bool) -> Expr {
    let mut family = Expr::aggregate_with_seed(Axis::Drop, axis::Known::NoDrop, generics, fields);
    if custom_drop {
        family = Expr::CustomDrop(Box::new(family));
    }
    family
}

fn gen_trap_family(generics: &syn::Generics, fields: &[&syn::Type], has_trap_values: bool) -> Expr {
    if has_trap_values {
        return Expr::Known(axis::Known::NonRobust);
    }
    let init = axis::Known::Robust;

    Expr::aggregate_with_seed(Axis::Trap, init, generics, fields)
}

fn gen_size_family(generics: &syn::Generics, fields: &[&syn::Type]) -> Expr {
    Expr::aggregate(Axis::Size, generics, fields).unwrap_or(Expr::Known(axis::Known::Zero))
}

fn gen_alignment_family(
    generics: &syn::Generics,
    fields: &[&syn::Type],
    tag: Option<&syn::Type>,
) -> Expr {
    let mut family =
        Expr::aggregate(Axis::Alignment, generics, fields).unwrap_or(Expr::Known(axis::Known::One));

    if let Some(tag) = tag {
        family = Expr::combine(
            Axis::Alignment,
            Expr::Projection(tag.clone(), Axis::Alignment),
            family,
        );
    }

    family
}

fn apply_repr_alignment(mut family: Expr, align: Option<usize>) -> Expr {
    let crate_ = crate_path();

    if matches!(align, Some(value) if value > 1) {
        let minimum = quote! { #crate_::Gt<#crate_::One> };
        family = Expr::combine(Axis::Alignment, Expr::Fixed(minimum), family);
    }

    family
}

fn gen_niche_family(generics: &syn::Generics, fields: &[&syn::Type]) -> Expr {
    Expr::aggregate_with_seed(Axis::Niche, axis::Known::NoNiche, generics, fields)
}

fn gen_transparent_niche_family(generics: &syn::Generics, fields: &[&syn::Type]) -> Expr {
    match fields {
        [] => Expr::Known(axis::Known::NoNiche),
        [field] => axis::field(field, Axis::Niche, generics),
        _ => gen_niche_family(generics, fields),
    }
}

fn gen_mutability_family(generics: &syn::Generics, fields: &[&syn::Type]) -> Expr {
    Expr::aggregate(Axis::Mutability, generics, fields)
        .unwrap_or_else(gen_exclusive_mutability_family)
}

fn gen_exclusive_mutability_family() -> Expr {
    Expr::Known(axis::Known::Exclusive)
}

fn gen_type_spec_impl(
    name: &syn::Ident,
    generics: &syn::Generics,
    families: Families<Expr>,
    attrs: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let (families, wrapper_checks) = axis::resolve_self(families, name, generics);
    let wrapper_checks = wrapper_checks.emit(name, generics);
    let families = families.map(|expression| expression.emit(generics));

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let predicates = where_clause
        .as_ref()
        .map(|where_clause| &where_clause.predicates);

    let crate_ = crate_path();
    let mut seen_bounds = std::collections::HashSet::new();
    let mut aggregate_bounds = Vec::new();
    let associated_types = families
        .into_iter()
        .map(|(axis, family)| {
            aggregate_bounds.extend(
                family
                    .aggregate_bounds
                    .into_iter()
                    .filter(|bound| seen_bounds.insert(bound.to_string())),
            );
            let name = axis.name();
            let kind = family.kind;
            quote!(type #name = #kind;)
        })
        .collect::<Vec<_>>();

    quote! {
        #attrs
        #wrapper_checks
        #attrs
        unsafe impl #impl_generics #crate_::RustSpec for #name #ty_generics where
            #(#aggregate_bounds,)*
            #predicates
        {
            #(#associated_types)*
            type Opacity = #crate_::NonOpaque;
        }
    }
}

fn gen_enum_niche_family(has_trap_tag_values: bool) -> Expr {
    if has_trap_tag_values {
        Expr::Niche(Box::new(Expr::Known(axis::Known::Unstable)))
    } else {
        Expr::Known(axis::Known::NoNiche)
    }
}

#[cfg(test)]
mod enum_tests {
    use super::*;
    use syn::visit::Visit;

    #[derive(Default)]
    struct TrapClassifications(Vec<String>);

    impl<'ast> Visit<'ast> for TrapClassifications {
        fn visit_impl_item_type(&mut self, item: &'ast syn::ImplItemType) {
            if item.ident == "Trap" {
                let ty = &item.ty;
                self.0.push(quote!(#ty).to_string());
            }
            syn::visit::visit_impl_item_type(self, item);
        }
    }

    #[test]
    fn non_robust_overrides_every_c_enum_tag_branch() {
        let crate_ = crate_path();
        let robust = quote!(#crate_::layout::Robust).to_string();
        let non_robust = quote!(#crate_::layout::NonRobust).to_string();

        // Inspect every target branch on the host, for both fieldless and
        // payload enums. Only the 8-bit tag is exhausted by 256 variants.
        for fields in [quote!(), quote!((()))] {
            for force_non_robust in [false, true] {
                let attribute = force_non_robust.then(|| quote!(#[rust_spec(non_robust)]));
                let variants = (0usize..256).map(|index| {
                    let name = format_ident!("V{}", index);
                    quote!(#name #fields)
                });
                let input = syn::parse_quote! {
                    #[repr(C)]
                    #attribute
                    enum Exhaustive { #(#variants),* }
                };
                let expanded: syn::File = syn::parse2(expand(&input).unwrap()).unwrap();
                let mut traps = TrapClassifications::default();
                traps.visit_file(&expanded);
                assert_eq!(
                    traps.0,
                    [
                        if force_non_robust {
                            non_robust.as_str()
                        } else {
                            robust.as_str()
                        },
                        non_robust.as_str(),
                        non_robust.as_str(),
                    ],
                    "fields={fields}, non_robust={force_non_robust}",
                );
            }
        }
    }
}

#[cfg(test)]
mod axis_short_circuit_tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn known_fields_remove_generic_axis_combinations() {
        let generics = parse_quote!(<T, U>);
        let first: syn::Type = parse_quote!(T);
        let second: syn::Type = parse_quote!(U);
        let crate_ = crate_path();

        for known in [
            parse_quote!((T,)),
            parse_quote!(&[T]),
            parse_quote!(&mut [T]),
            parse_quote!(*const [T]),
            parse_quote!(*mut [T]),
        ] {
            let family =
                gen_stable_layout_family(&generics, &[&first, &second, &known]).emit(&generics);
            assert!(family.aggregate_bounds.is_empty());
            assert_eq!(
                family.kind.to_string(),
                quote!(#crate_::Unstable).to_string()
            );
        }
        for known in [parse_quote!(&T), parse_quote!(&mut T)] {
            let family =
                gen_trap_family(&generics, &[&first, &second, &known], false).emit(&generics);
            assert!(family.aggregate_bounds.is_empty());
            assert_eq!(
                family.kind.to_string(),
                quote!(#crate_::layout::NonRobust).to_string()
            );
        }
        let tagged = gen_trap_family(&generics, &[&first, &second], true).emit(&generics);
        assert!(tagged.aggregate_bounds.is_empty());
        assert_eq!(
            tagged.kind.to_string(),
            quote!(#crate_::layout::NonRobust).to_string()
        );

        let tail = parse_quote!([U]);
        let family = gen_size_family(&generics, &[&first, &tail]).emit(&generics);
        assert!(family.aggregate_bounds.is_empty());
        assert_eq!(
            family.kind.to_string(),
            quote!(#crate_::size::MetaSized<#crate_::size::SliceLike>).to_string()
        );
    }
}
