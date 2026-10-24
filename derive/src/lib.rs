use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Token, punctuated::Punctuated};

use crate::repr::{
    ReprKind, has_unresolved_discriminants, infer_repr, is_exhaustive_enum, parse_repr,
};

mod classify;
mod repr;

/// Derives `RustSpec` for a struct, enum, or union.
///
/// Add `#[rust_spec(custom_drop)]` if the type itself implements [`Drop`].
/// Add `#[rust_spec(custom_niche)]` if a struct has a custom niche.
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

struct AggregateFamily {
    kind: proc_macro2::TokenStream,
    aggregate_bounds: Vec<proc_macro2::TokenStream>,
}

struct TypeSpecFamilies {
    layout: AggregateFamily,
    size: AggregateFamily,
    alignment: AggregateFamily,
    trap: AggregateFamily,
    niche: AggregateFamily,
    drop: AggregateFamily,
    mutability: AggregateFamily,
    indirect_trap: AggregateFamily,
}

impl AggregateFamily {
    fn fixed(kind: proc_macro2::TokenStream) -> Self {
        Self {
            kind,
            aggregate_bounds: Vec::new(),
        }
    }

    fn aggregate_parts(
        operator: proc_macro2::TokenStream,
        axis: proc_macro2::TokenStream,
        generics: &syn::Generics,
        fields: &[&syn::Type],
    ) -> (
        Option<proc_macro2::TokenStream>,
        Option<proc_macro2::TokenStream>,
        Vec<proc_macro2::TokenStream>,
    ) {
        let mut aggregate_bounds = Vec::new();

        let (concrete_fields, parameterized_fields): (Vec<_>, Vec<_>) = fields
            .iter()
            .copied()
            .partition(|field| !field_has_type_params(field, generics));

        let mut concrete_fields = concrete_fields.into_iter();
        let concrete_kind = concrete_fields.next().map(|field| {
            let mut kind = field_axis_kind(field, &axis);

            for field in concrete_fields {
                let field_kind = field_axis_kind(field, &axis);
                kind = quote! { <#kind as #operator<#field_kind>>::Output };
            }

            kind
        });

        let mut parameterized_fields = parameterized_fields.into_iter();
        let parameterized_kind = parameterized_fields.next().map(|field| {
            let mut kind = field_axis_kind(field, &axis);

            for field in parameterized_fields {
                let field_kind = field_axis_kind(field, &axis);

                aggregate_bounds.push(quote! { #kind: #operator<#field_kind> });
                kind = quote! { <#kind as #operator<#field_kind>>::Output };
            }

            kind
        });

        (concrete_kind, parameterized_kind, aggregate_bounds)
    }

    fn aggregate(
        operator: proc_macro2::TokenStream,
        axis: proc_macro2::TokenStream,
        generics: &syn::Generics,
        fields: &[&syn::Type],
    ) -> Option<Self> {
        let (concrete_kind, parameterized_kind, mut aggregate_bounds) =
            Self::aggregate_parts(operator.clone(), axis, generics, fields);
        let kind = match (concrete_kind, parameterized_kind) {
            (Some(concrete), Some(parameterized)) => {
                aggregate_bounds.push(quote! { #concrete: #operator<#parameterized> });

                quote! { <#concrete as #operator<#parameterized>>::Output }
            }
            (Some(concrete), None) => concrete,
            (None, Some(parameterized)) => parameterized,
            (None, None) => return None,
        };

        Some(Self {
            kind,
            aggregate_bounds,
        })
    }

    fn aggregate_with_seed(
        operator: proc_macro2::TokenStream,
        axis: proc_macro2::TokenStream,
        seed: proc_macro2::TokenStream,
        generics: &syn::Generics,
        fields: &[&syn::Type],
    ) -> Self {
        let (concrete_kind, parameterized_kind, mut aggregate_bounds) =
            Self::aggregate_parts(operator.clone(), axis, generics, fields);

        let kind = match (concrete_kind, parameterized_kind) {
            (Some(concrete), Some(parameterized)) => {
                aggregate_bounds.push(
                    quote! { <#seed as #operator<#concrete>>::Output: #operator<#parameterized> },
                );
                quote! {
                    <<#seed as #operator<#concrete>>::Output as #operator<#parameterized>>::Output
                }
            }
            (Some(concrete), None) => quote! { <#seed as #operator<#concrete>>::Output },
            (None, Some(parameterized)) => {
                aggregate_bounds.push(quote! { #seed: #operator<#parameterized> });
                quote! { <#seed as #operator<#parameterized>>::Output }
            }
            (None, None) => return Self::fixed(seed),
        };

        Self {
            kind,
            aggregate_bounds,
        }
    }
}

fn field_axis_kind(field: &syn::Type, axis: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let crate_ = crate_path();

    quote! { <#field as #crate_::RustSpec>::#axis }
}

fn field_has_type_params(ty: &syn::Type, generics: &syn::Generics) -> bool {
    use syn::visit::Visit;

    struct Visitor<'a> {
        type_params: Vec<&'a syn::Ident>,
        found: bool,
    }

    impl<'a> Visitor<'a> {
        fn new(generics: &'a syn::Generics) -> Self {
            Self {
                type_params: generics.type_params().map(|p| &p.ident).collect(),
                found: false,
            }
        }
    }

    impl syn::visit::Visit<'_> for Visitor<'_> {
        fn visit_type_path(&mut self, type_path: &syn::TypePath) {
            if type_path.qself.is_none()
                && let Some(first_segment) = type_path.path.segments.first()
                && self.type_params.contains(&&first_segment.ident)
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
            .push(syn::parse_quote!(Self: Sized));
        let (impl_generics, ty_generics, where_clause) = rust_spec_generics.split_for_impl();
        return Ok(quote! {
            unsafe impl #impl_generics #crate_::RustSpec for #name #ty_generics #where_clause {
                type Layout = #crate_::Stable;
                type Size = #crate_::size::Sized<#crate_::Gt<#crate_::Zero>>;
                type Alignment = #crate_::One;
                type Trap = #crate_::layout::Robust;
                type Niche = #crate_::niche::WithoutNiche;
                type Drop = #crate_::drop::NoDrop;
                type Mutability = #crate_::mutability::Exclusive;
                type Opacity = #crate_::Opaque;
                type __IndirectTrap = #crate_::layout::Robust;
            }
        });
    }
    let raw_pointer_field = match &input.data {
        syn::Data::Struct(data) => data
            .fields
            .iter()
            .find(|field| classify::contains_raw_pointer(&field.ty)),
        syn::Data::Enum(data) => data
            .variants
            .iter()
            .flat_map(|variant| variant.fields.iter())
            .find(|field| classify::contains_raw_pointer(&field.ty)),
        syn::Data::Union(data) => data
            .fields
            .named
            .iter()
            .find(|field| classify::contains_raw_pointer(&field.ty)),
    };
    if let Some(field) = raw_pointer_field {
        return Err(syn::Error::new_spanned(
            &field.ty,
            "RustSpec cannot be derived for a raw pointer field",
        ));
    }
    if attrs.custom_niche && !matches!(&input.data, syn::Data::Struct(_)) {
        return Err(syn::Error::new_spanned(
            name,
            "`custom_niche` is only supported on structs",
        ));
    }
    let discriminant_assert = if repr.is_none()
        && let syn::Data::Enum(data) = &input.data
    {
        let tag = infer_repr(&data.variants)?;
        has_unresolved_discriminants(&data.variants)
            .then(|| gen_discriminant_assert(name, &data.variants, &tag))
    } else {
        None
    };
    let rust_spec_impl = match &input.data {
        syn::Data::Struct(data) => {
            gen_struct_impl(repr, alignment, name, generics, &data.fields, attrs)
        }
        syn::Data::Enum(data) if is_fieldless_enum(data) => {
            gen_fieldless_enum_impl(repr, alignment, name, generics, &data.variants, attrs)
        }
        syn::Data::Enum(data)
            if matches!(repr, Some(ReprKind::Transparent))
                || repr.is_none() && data.variants.len() == 1 =>
        {
            let Some(variant) = data.variants.first() else {
                return Ok(quote! {});
            };

            let fields = field_types(&variant.fields);
            gen_struct_fields_impl(
                repr,
                alignment,
                name,
                generics,
                &fields,
                attrs,
                gen_mutability_family(generics, &fields),
            )
        }
        syn::Data::Enum(data) => {
            gen_enum_impl(repr, alignment, name, generics, &data.variants, attrs)
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
    Ok(quote! { #discriminant_assert #drop_assert #rust_spec_impl })
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
        let value = quote! { (core::convert::identity::<isize>(#base) as i128 + #offset as i128) };
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
                        fn require_drop<T: core::ops::Drop>() {}
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
                        impl<T: ?Sized> DropImplIsNotSupported<()> for T {}
                        struct DropDetected;
                        impl<T: ?Sized + core::ops::Drop> DropImplIsNotSupported<DropDetected> for T {}
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

fn is_fieldless_enum(data: &syn::DataEnum) -> bool {
    data.variants
        .iter()
        .all(|variant| matches!(variant.fields, syn::Fields::Unit))
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
    mutability: AggregateFamily,
) -> proc_macro2::TokenStream {
    let layout = if repr.is_some() {
        gen_stable_layout_family(generics, fields)
    } else {
        gen_unstable_layout_family()
    };
    let size = gen_size_family(generics, fields);
    let alignment = apply_repr_alignment(gen_alignment_family(generics, fields, None), alignment);
    let trap = if attrs.custom_niche {
        let crate_ = crate_path();
        AggregateFamily::fixed(quote! { #crate_::layout::NonRobust })
    } else {
        gen_trap_family(generics, fields, false)
    };
    let niche = if attrs.custom_niche {
        let crate_ = crate_path();
        AggregateFamily::fixed(quote! { #crate_::niche::WithNiche<#crate_::Unstable> })
    } else if let Some(ReprKind::Transparent) = repr {
        gen_transparent_niche_family(generics, fields)
    } else {
        gen_niche_family(generics, fields)
    };

    let drop = gen_drop_family(generics, fields, attrs.custom_drop);
    let indirect_trap = gen_indirect_trap_family(generics, fields);

    let spec = TypeSpecFamilies {
        layout,
        size,
        alignment,
        trap,
        niche,
        drop,
        mutability,
        indirect_trap,
    };

    gen_type_spec_impl(name, generics, fields, spec, quote! {})
}

fn gen_enum_impl(
    repr: Option<&ReprKind>,
    repr_alignment: Option<usize>,
    name: &syn::Ident,
    generics: &syn::Generics,
    variants: &Punctuated<syn::Variant, Token![,]>,
    attrs: RustSpecAttrs,
) -> proc_macro2::TokenStream {
    if matches!(repr, Some(ReprKind::C(None))) {
        return gen_plain_c_enum_impls(repr_alignment, name, generics, variants, attrs);
    }

    let crate_ = crate_path();
    let fields = variant_field_types(variants);
    let has_trap_tag_values = match repr {
        None => rust_tag_has_traps(variants),
        Some(ReprKind::C(Some(tag)) | ReprKind::Primitive(tag)) => {
            primitive_tag_has_traps(tag, variants.len())
        }
        Some(ReprKind::C(None)) => unreachable!("handled by gen_plain_c_enum_impls"),
        Some(ReprKind::Transparent) => false,
    };
    let layout = if repr.is_some() {
        gen_enum_layout_family(generics, &fields)
    } else {
        gen_unstable_layout_family()
    };
    let size = AggregateFamily::fixed(quote! {
        #crate_::size::Sized<#crate_::Gt<#crate_::Zero>>
    });
    let tag = match repr {
        Some(ReprKind::Primitive(tag) | ReprKind::C(Some(tag))) => Some(tag.as_ref().clone()),
        Some(ReprKind::C(None)) => unreachable!("handled by gen_plain_c_enum_impls"),
        Some(ReprKind::Transparent) => None,
        None => Some(infer_repr(variants).expect("validated enum discriminants")),
    };
    let alignment = apply_repr_alignment(
        gen_alignment_family(generics, &fields, tag.as_ref()),
        repr_alignment,
    );
    let trap = gen_trap_family(generics, &fields, has_trap_tag_values);

    let niche = gen_enum_niche_family(has_trap_tag_values);
    let drop = gen_drop_family(generics, &fields, attrs.custom_drop);
    let mutability = gen_mutability_family(generics, &fields);
    let indirect_trap = gen_indirect_trap_family(generics, &fields);

    let spec = TypeSpecFamilies {
        layout,
        size,
        alignment,
        trap,
        niche,
        drop,
        mutability,
        indirect_trap,
    };

    gen_type_spec_impl(name, generics, &fields, spec, quote! {})
}

fn gen_union_impl(
    repr: Option<&ReprKind>,
    repr_alignment: Option<usize>,
    name: &syn::Ident,
    generics: &syn::Generics,
    fields: &syn::FieldsNamed,
    attrs: RustSpecAttrs,
) -> proc_macro2::TokenStream {
    let crate_ = crate_path();

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
    let trap = AggregateFamily::fixed(quote! {
        #crate_::layout::Robust
    });
    let niche = AggregateFamily::fixed(quote! {
        #crate_::niche::WithoutNiche
    });
    let drop = gen_drop_family(generics, &fields, attrs.custom_drop);

    let mutability = gen_mutability_family(generics, &fields);
    let indirect_trap = AggregateFamily::fixed(quote! {
        #crate_::layout::Robust
    });

    let spec = TypeSpecFamilies {
        layout,
        size,
        alignment,
        trap,
        niche,
        drop,
        mutability,
        indirect_trap,
    };

    gen_type_spec_impl(name, generics, &fields, spec, quote! {})
}

fn gen_fieldless_enum_impl(
    repr: Option<&ReprKind>,
    repr_alignment: Option<usize>,
    name: &syn::Ident,
    generics: &syn::Generics,
    variants: &Punctuated<syn::Variant, Token![,]>,
    attrs: RustSpecAttrs,
) -> proc_macro2::TokenStream {
    if matches!(repr, Some(ReprKind::C(None))) {
        return gen_plain_c_enum_impls(repr_alignment, name, generics, variants, attrs);
    }

    let crate_ = crate_path();
    let layout_kind = match repr {
        None => quote! { #crate_::Unstable },
        Some(ReprKind::C(None)) => unreachable!("handled by gen_plain_c_enum_impls"),
        Some(ReprKind::Transparent) => quote! {
            #crate_::Stable
        },
        Some(ReprKind::C(Some(_)) | ReprKind::Primitive(_)) => {
            quote! { #crate_::Stable }
        }
    };
    let has_tag = !variants.is_empty()
        && match repr {
            Some(ReprKind::C(_) | ReprKind::Primitive(_)) => true,
            Some(ReprKind::Transparent) | None => variants.len() > 1,
        };

    let size = if has_tag {
        AggregateFamily::fixed(quote! { #crate_::size::Sized<#crate_::Gt<#crate_::Zero>> })
    } else {
        AggregateFamily::fixed(quote! { #crate_::size::Sized<#crate_::size::Zero> })
    };

    let niche = if has_tag {
        let has_trap_tag_values = match repr {
            None => rust_tag_has_traps(variants),
            Some(ReprKind::C(Some(tag)) | ReprKind::Primitive(tag)) => {
                primitive_tag_has_traps(tag, variants.len())
            }
            Some(ReprKind::C(None)) => unreachable!("handled by gen_plain_c_enum_impls"),
            Some(ReprKind::Transparent) => false,
        };
        gen_enum_niche_family(has_trap_tag_values)
    } else {
        AggregateFamily::fixed(quote! { #crate_::niche::WithoutNiche })
    };

    let has_trap_tag_values = has_tag
        && match repr {
            None => rust_tag_has_traps(variants),
            Some(ReprKind::C(Some(tag)) | ReprKind::Primitive(tag)) => {
                primitive_tag_has_traps(tag, variants.len())
            }
            Some(ReprKind::C(None)) => unreachable!("handled by gen_plain_c_enum_impls"),
            Some(ReprKind::Transparent) => false,
        };
    let layout = AggregateFamily::fixed(layout_kind);
    let alignment = if has_tag {
        let tag = match repr {
            Some(ReprKind::Primitive(tag) | ReprKind::C(Some(tag))) => tag.as_ref().clone(),
            Some(ReprKind::C(None)) => unreachable!("handled by gen_plain_c_enum_impls"),
            Some(ReprKind::Transparent) => {
                infer_repr(variants).expect("validated enum discriminants")
            }
            None => infer_repr(variants).expect("validated enum discriminants"),
        };
        gen_alignment_family(generics, &[], Some(&tag))
    } else {
        gen_alignment_family(generics, &[], None)
    };
    let alignment = apply_repr_alignment(alignment, repr_alignment);
    let trap = AggregateFamily::fixed(if has_trap_tag_values {
        quote! { #crate_::layout::NonRobust }
    } else {
        quote! { #crate_::layout::Robust }
    });
    let mutability = gen_exclusive_mutability_family();
    let drop = gen_drop_family(generics, &[], attrs.custom_drop);
    let indirect_trap = AggregateFamily::fixed(quote! { #crate_::layout::Robust });

    let spec = TypeSpecFamilies {
        layout,
        size,
        alignment,
        trap,
        niche,
        drop,
        mutability,
        indirect_trap,
    };

    gen_type_spec_impl(name, generics, &[], spec, quote! {})
}

fn gen_unstable_layout_family() -> AggregateFamily {
    let crate_ = crate_path();
    AggregateFamily::fixed(quote! { #crate_::Unstable })
}

fn gen_stable_layout_family(generics: &syn::Generics, fields: &[&syn::Type]) -> AggregateFamily {
    let crate_ = crate_path();

    AggregateFamily::aggregate(
        quote! { core::ops::Add },
        quote! { Layout },
        generics,
        fields,
    )
    .unwrap_or_else(|| AggregateFamily::fixed(quote! { #crate_::Stable }))
}

fn gen_enum_layout_family(generics: &syn::Generics, fields: &[&syn::Type]) -> AggregateFamily {
    gen_stable_layout_family(generics, fields)
}

fn gen_plain_c_enum_impls(
    repr_alignment: Option<usize>,
    name: &syn::Ident,
    generics: &syn::Generics,
    variants: &Punctuated<syn::Variant, Token![,]>,
    attrs: RustSpecAttrs,
) -> proc_macro2::TokenStream {
    let crate_ = crate_path();
    let fields = variant_field_types(variants);

    let eight_bit_targets = quote! {
        #[cfg(all(
            target_arch = "arm",
            any(target_os = "none", target_os = "nuttx", target_os = "rtems"),
        ))]
    };
    let sixteen_bit_targets = quote! {
        #[cfg(any(target_arch = "avr", target_arch = "msp430"))]
    };
    let normal_targets = quote! {
        #[cfg(not(any(
            all(
                target_arch = "arm",
                any(target_os = "none", target_os = "nuttx", target_os = "rtems"),
            ),
            target_arch = "avr",
            target_arch = "msp430",
        )))]
    };

    [
        (eight_bit_targets, 8),
        (sixteen_bit_targets, 16),
        (normal_targets, 32),
    ]
    .into_iter()
    .map(|(cfg, bits)| {
        let tag = c_tag_type(bits);
        let has_trap_tag_values = primitive_tag_has_traps(&tag, variants.len());
        let layout = gen_stable_layout_family(generics, &fields);
        let niche = gen_enum_niche_family(has_trap_tag_values);
        let spec = TypeSpecFamilies {
            layout,
            size: AggregateFamily::fixed(quote! {
                #crate_::size::Sized<#crate_::Gt<#crate_::Zero>>
            }),
            alignment: apply_repr_alignment(
                gen_alignment_family(generics, &fields, Some(&tag)),
                repr_alignment,
            ),
            trap: gen_trap_family(generics, &fields, has_trap_tag_values),
            niche,
            drop: gen_drop_family(generics, &fields, attrs.custom_drop),
            mutability: gen_mutability_family(generics, &fields),
            indirect_trap: gen_indirect_trap_family(generics, &fields),
        };

        gen_type_spec_impl(name, generics, &fields, spec, cfg)
    })
    .collect()
}

fn gen_indirect_trap_family(generics: &syn::Generics, fields: &[&syn::Type]) -> AggregateFamily {
    let crate_ = crate_path();

    AggregateFamily::aggregate(
        quote! { core::ops::Add },
        quote! { __IndirectTrap },
        generics,
        fields,
    )
    .unwrap_or_else(|| AggregateFamily::fixed(quote! { #crate_::layout::Robust }))
}

fn gen_drop_family(
    generics: &syn::Generics,
    fields: &[&syn::Type],
    custom_drop: bool,
) -> AggregateFamily {
    let crate_ = crate_path();
    let mut family = AggregateFamily::aggregate_with_seed(
        quote! { core::ops::Add },
        quote! { Drop },
        quote! { #crate_::drop::NoDrop },
        generics,
        fields,
    );
    if custom_drop {
        let kind = family.kind;
        family
            .aggregate_bounds
            .push(quote! { #kind: #crate_::drop::DropKind });
        family.kind = quote! { #crate_::drop::CustomDrop<#kind> };
    }
    family
}

fn gen_trap_family(
    generics: &syn::Generics,
    fields: &[&syn::Type],
    has_trap_values: bool,
) -> AggregateFamily {
    let crate_ = crate_path();

    let init = if has_trap_values {
        quote! { #crate_::layout::NonRobust }
    } else {
        quote! { #crate_::layout::Robust }
    };

    AggregateFamily::aggregate_with_seed(
        quote! { core::ops::Add },
        quote! { Trap },
        init,
        generics,
        fields,
    )
}

fn gen_size_family(generics: &syn::Generics, fields: &[&syn::Type]) -> AggregateFamily {
    let crate_ = crate_path();
    if fields
        .iter()
        .all(|field| classify::classify_nonzero(field) == classify::NonZeroClass::No)
    {
        return AggregateFamily::fixed(quote! { #crate_::size::Sized<#crate_::Zero> });
    }
    if let Some(tail) = fields.last()
        && classify::classify_type(tail, generics) == classify::TypeClass::Sized
        && classify::classify_nonzero(tail) == classify::NonZeroClass::Yes
    {
        return AggregateFamily::fixed(quote! { #crate_::size::Sized<#crate_::Gt<#crate_::Zero>> });
    }

    AggregateFamily::aggregate(quote! { core::ops::Add }, quote! { Size }, generics, fields)
        .unwrap_or_else(|| AggregateFamily::fixed(quote! { #crate_::size::Sized<#crate_::Zero> }))
}

fn gen_alignment_family(
    generics: &syn::Generics,
    fields: &[&syn::Type],
    tag: Option<&syn::Type>,
) -> AggregateFamily {
    let crate_ = crate_path();

    let mut family = AggregateFamily::aggregate(
        quote! { #crate_::Max },
        quote! { Alignment },
        generics,
        fields,
    )
    .unwrap_or_else(|| AggregateFamily::fixed(quote! { #crate_::One }));

    if let Some(tag) = tag {
        let kind = family.kind;

        let tag_kind = quote! { <#tag as #crate_::RustSpec>::Alignment };
        family.kind = quote! { <#tag_kind as #crate_::Max<#kind>>::Output };
    }

    family
}

fn apply_repr_alignment(mut family: AggregateFamily, align: Option<usize>) -> AggregateFamily {
    let crate_ = crate_path();

    if matches!(align, Some(value) if value > 1) {
        let minimum = quote! { #crate_::Gt<#crate_::One> };
        let kind = family.kind;
        family.kind = quote! { <#minimum as #crate_::Max<#kind>>::Output };
    }

    family
}

fn gen_niche_family(generics: &syn::Generics, fields: &[&syn::Type]) -> AggregateFamily {
    let crate_ = crate_path();

    AggregateFamily::aggregate_with_seed(
        quote! { core::ops::Add },
        quote! { Niche },
        quote! { #crate_::niche::WithoutNiche },
        generics,
        fields,
    )
}

fn gen_transparent_niche_family(
    generics: &syn::Generics,
    fields: &[&syn::Type],
) -> AggregateFamily {
    match fields {
        [] => {
            let crate_ = crate_path();
            AggregateFamily::fixed(quote! { #crate_::niche::WithoutNiche })
        }
        [field] => AggregateFamily::fixed(field_axis_kind(field, &quote! { Niche })),
        _ => gen_niche_family(generics, fields),
    }
}

fn gen_mutability_family(generics: &syn::Generics, fields: &[&syn::Type]) -> AggregateFamily {
    AggregateFamily::aggregate(
        quote! { core::ops::Add },
        quote! { Mutability },
        generics,
        fields,
    )
    .unwrap_or_else(gen_exclusive_mutability_family)
}

fn gen_exclusive_mutability_family() -> AggregateFamily {
    let crate_ = crate_path();
    AggregateFamily::fixed(quote! { #crate_::mutability::Exclusive })
}

fn gen_type_spec_impl(
    name: &syn::Ident,
    generics: &syn::Generics,
    fields: &[&syn::Type],
    families: TypeSpecFamilies,
    attrs: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let TypeSpecFamilies {
        layout,
        size,
        alignment,
        trap,
        niche,
        drop,
        mutability,
        indirect_trap,
    } = families;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let predicates = where_clause
        .as_ref()
        .map(|where_clause| &where_clause.predicates);

    let crate_ = crate_path();
    let field_bounds = fields
        .iter()
        .filter(|field| field_has_type_params(field, generics))
        .map(|field| quote! { #field: #crate_::RustSpec });

    let aggregate_bounds = layout
        .aggregate_bounds
        .into_iter()
        .chain(size.aggregate_bounds)
        .chain(alignment.aggregate_bounds)
        .chain(trap.aggregate_bounds)
        .chain(niche.aggregate_bounds)
        .chain(drop.aggregate_bounds)
        .chain(mutability.aggregate_bounds)
        .chain(indirect_trap.aggregate_bounds);

    let layout_kind = layout.kind;
    let size_kind = size.kind;
    let alignment_kind = alignment.kind;
    let trap_kind = trap.kind;
    let niche_kind = niche.kind;
    let drop_kind = drop.kind;
    let mutability_kind = mutability.kind;
    let indirect_trap_kind = indirect_trap.kind;

    quote! {
        #attrs
        unsafe impl #impl_generics #crate_::RustSpec for #name #ty_generics where
            #(#field_bounds,)*
            #(#aggregate_bounds,)*
            #predicates
        {
            type Layout = #layout_kind;
            type Size = #size_kind;
            type Alignment = #alignment_kind;
            type Trap = #trap_kind;
            type Niche = #niche_kind;
            type Drop = #drop_kind;
            type Mutability = #mutability_kind;
            type Opacity = #crate_::NonOpaque;
            type __IndirectTrap = #indirect_trap_kind;
        }
    }
}

fn gen_enum_niche_family(has_trap_tag_values: bool) -> AggregateFamily {
    let crate_ = crate_path();

    let niche_kind = if has_trap_tag_values {
        quote! { #crate_::niche::WithNiche<#crate_::Unstable> }
    } else {
        quote! { #crate_::niche::WithoutNiche }
    };

    AggregateFamily::fixed(niche_kind)
}

fn primitive_tag_has_traps(tag: &syn::Type, variant_count: usize) -> bool {
    !is_exhaustive_enum(variant_count, tag)
}

fn rust_tag_has_traps(variants: &Punctuated<syn::Variant, Token![,]>) -> bool {
    let tag = infer_repr(variants).expect("validated enum discriminants");
    primitive_tag_has_traps(&tag, variants.len())
}

fn c_tag_type(c_enum_bits: u32) -> syn::Type {
    match c_enum_bits {
        8 => syn::parse_quote!(u8),
        16 => syn::parse_quote!(u16),
        32 => syn::parse_quote!(u32),
        _ => unreachable!("unsupported C enum width"),
    }
}
