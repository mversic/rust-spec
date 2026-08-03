use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Token, punctuated::Punctuated};

use crate::repr::{ReprKind, infer_repr, is_exhaustive_enum, parse_repr};

mod repr;

#[proc_macro_derive(RustSpec)]
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

    fn fold(
        axis: proc_macro2::TokenStream,
        init_kind: proc_macro2::TokenStream,
        generics: &syn::Generics,
        fields: &[&syn::Type],
    ) -> Self {
        let (kind, aggregate_bounds) =
            Self::fold_fields(axis, init_kind, generics, false, fields, 0);

        Self {
            kind,
            aggregate_bounds,
        }
    }

    fn from_fields(
        axis: proc_macro2::TokenStream,
        empty_kind: proc_macro2::TokenStream,
        generics: &syn::Generics,
        fields: &[&syn::Type],
    ) -> Self {
        let [first, rest @ ..] = fields else {
            return Self::fixed(empty_kind);
        };

        let kind = field_axis_kind(first, 0, &axis, generics);
        let (kind, aggregate_bounds) = Self::fold_fields(
            axis,
            kind,
            generics,
            field_needs_bounds(first, generics),
            rest,
            1,
        );

        Self {
            kind,
            aggregate_bounds,
        }
    }

    fn fold_fields(
        axis: proc_macro2::TokenStream,
        mut kind: proc_macro2::TokenStream,
        generics: &syn::Generics,
        mut accumulated_is_parameterized: bool,
        fields: &[&syn::Type],
        field_offset: usize,
    ) -> (proc_macro2::TokenStream, Vec<proc_macro2::TokenStream>) {
        let mut aggregate_bounds = Vec::new();
        for (index, &field) in fields.iter().enumerate() {
            let index = index + field_offset;
            let field_kind = field_axis_kind(field, index, &axis, generics);
            let field_is_parameterized = field_needs_bounds(field, generics);
            if accumulated_is_parameterized || field_is_parameterized {
                let bound = quote! { #kind: core::ops::Add<#field_kind> };
                aggregate_bounds.push(bound);
            }
            kind = quote! { <#kind as core::ops::Add<#field_kind>>::Output };
            accumulated_is_parameterized |= field_is_parameterized;
        }

        (kind, aggregate_bounds)
    }
}

fn hrtb_projection(field: &syn::Type, generics: &syn::Generics) -> Option<syn::Lifetime> {
    let syn::Type::Path(type_path) = field else {
        return None;
    };
    let qself = type_path.qself.as_ref()?;
    let trait_path = syn::Path {
        leading_colon: type_path.path.leading_colon,
        segments: type_path
            .path
            .segments
            .iter()
            .take(qself.position)
            .cloned()
            .collect(),
    };
    let has_prerequisite = generics.where_clause.iter().flat_map(|clause| &clause.predicates).any(|predicate| {
        let syn::WherePredicate::Type(predicate) = predicate else { return false };
        predicate.lifetimes.is_some()
            && predicate.bounded_ty.to_token_stream().to_string() == qself.ty.to_token_stream().to_string()
            && predicate.bounds.iter().any(|bound| matches!(bound, syn::TypeParamBound::Trait(bound) if bound.path.to_token_stream().to_string() == trait_path.to_token_stream().to_string()))
    });
    has_prerequisite.then(|| {
        type_path
            .path
            .segments
            .iter()
            .skip(qself.position)
            .find_map(|segment| match &segment.arguments {
                syn::PathArguments::AngleBracketed(arguments) => {
                    arguments.args.iter().find_map(|argument| match argument {
                        syn::GenericArgument::Lifetime(lifetime) => Some(lifetime.clone()),
                        _ => None,
                    })
                }
                _ => None,
            })
            .unwrap_or_else(|| syn::Lifetime::new("'static", proc_macro2::Span::call_site()))
    })
}

fn field_axis_kind(
    field: &syn::Type,
    index: usize,
    axis: &proc_macro2::TokenStream,
    generics: &syn::Generics,
) -> proc_macro2::TokenStream {
    let crate_ = crate_path();

    if let Some(lifetime) = hrtb_projection(field, generics) {
        return quote! { <Self as #crate_::__HrtbAxes<#index>>::#axis<#lifetime> };
    }

    quote! { <#field as #crate_::RustSpec>::#axis }
}

fn hrtb_axes_impl(
    name: &syn::Ident,
    generics: &syn::Generics,
    field: &syn::Type,
    index: usize,
) -> Option<proc_macro2::TokenStream> {
    hrtb_projection(field, generics)?;
    let syn::Type::Path(type_path) = field else {
        return None;
    };
    let qself = type_path.qself.as_ref()?;
    let projection_input = &qself.ty;
    let projection_trait = syn::Path {
        leading_colon: type_path.path.leading_colon,
        segments: type_path
            .path
            .segments
            .iter()
            .take(qself.position)
            .cloned()
            .collect(),
    };

    let field_for_lifetime = field;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let predicates = where_clause
        .map(|where_clause| where_clause.predicates.iter().collect::<Vec<_>>())
        .unwrap_or_default();
    let crate_ = crate_path();
    Some(quote! {
        #[doc(hidden)]
        impl #impl_generics #crate_::__HrtbAxes<#index> for #name #ty_generics
        where
            #(#predicates,)*
            for<'__rust_spec> #projection_input: #projection_trait,
            for<'__rust_spec> #field_for_lifetime: #crate_::RustSpec,
        {
            type Layout<'__rust_spec> = <#field_for_lifetime as #crate_::RustSpec>::Layout;
            type Size<'__rust_spec> = <#field_for_lifetime as #crate_::RustSpec>::Size;
            type Alignment<'__rust_spec> = <#field_for_lifetime as #crate_::RustSpec>::Alignment;
            type Trap<'__rust_spec> = <#field_for_lifetime as #crate_::RustSpec>::Trap;
            type Niche<'__rust_spec> = <#field_for_lifetime as #crate_::RustSpec>::Niche;
            type Mutability<'__rust_spec> = <#field_for_lifetime as #crate_::RustSpec>::Mutability;
            type __IndirectTrap<'__rust_spec> = <#field_for_lifetime as #crate_::RustSpec>::__IndirectTrap;
        }
    })
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

        // Raw pointers have fixed classifications and do not follow their pointees.
        fn visit_type_ptr(&mut self, _: &syn::TypePtr) {}
    }

    let mut visitor = Visitor::new(generics);
    visitor.visit_type(ty);
    visitor.found
}

fn field_needs_bounds(field: &syn::Type, generics: &syn::Generics) -> bool {
    field_has_type_params(field, generics) || hrtb_projection(field, generics).is_some()
}

fn expand(input: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let repr = parse_repr(&input.attrs)?;
    let alignment = repr.align;
    let repr = repr.kind.as_ref();
    let name = &input.ident;
    let generics = &input.generics;
    let rust_spec_impl = match &input.data {
        syn::Data::Struct(data) => gen_struct_impl(repr, alignment, name, generics, &data.fields),
        syn::Data::Enum(data) if is_fieldless_enum(data) => {
            gen_fieldless_enum_impl(repr, alignment, name, generics, &data.variants)
        }
        syn::Data::Enum(data)
            if matches!(repr, Some(ReprKind::Transparent))
                || repr.is_none() && data.variants.len() == 1 =>
        {
            let Some(variant) = data.variants.first() else {
                return Ok(quote! {});
            };

            gen_struct_impl(repr, alignment, name, generics, &variant.fields)
        }
        syn::Data::Enum(data) => gen_enum_impl(repr, alignment, name, generics, &data.variants),
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
            )
        }
        syn::Data::Union(data) => gen_union_impl(repr, alignment, name, generics, &data.fields),
    };

    Ok(quote! { #rust_spec_impl })
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
) -> proc_macro2::TokenStream {
    let fields = field_types(fields);

    gen_struct_fields_impl(repr, repr_alignment, name, generics, &fields)
}

fn gen_struct_fields_impl(
    repr: Option<&ReprKind>,
    repr_alignment: Option<usize>,
    name: &syn::Ident,
    generics: &syn::Generics,
    fields: &[&syn::Type],
) -> proc_macro2::TokenStream {
    let layout = if repr.is_some() {
        gen_stable_layout_family(generics, fields)
    } else {
        gen_aggregate_rust_layout_family()
    };
    let size = gen_size_family(generics, fields);
    let alignment =
        apply_repr_alignment(gen_alignment_family(generics, fields, None), repr_alignment);
    let trap = gen_trap_family(generics, fields, false);
    let niche = if let Some(ReprKind::Transparent) = repr {
        gen_transparent_niche_family(generics, fields)
    } else {
        gen_niche_family(generics, fields)
    };

    let mutability = gen_single_field_mutability_family(generics, fields);
    let indirect_trap = gen_indirect_trap_family(generics, fields);

    let spec = TypeSpecFamilies {
        layout,
        size,
        alignment,
        trap,
        niche,
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
) -> proc_macro2::TokenStream {
    if matches!(repr, Some(ReprKind::C(None))) {
        return gen_plain_c_enum_impls(repr_alignment, name, generics, variants);
    }

    let crate_ = crate_path();
    let fields = variant_field_types(variants);
    let has_trap_tag_values = match repr {
        None => rust_tag_has_traps(variants.len()),
        Some(ReprKind::C(Some(tag)) | ReprKind::Primitive(tag)) => {
            primitive_tag_has_traps(tag, variants.len())
        }
        Some(ReprKind::C(None)) => unreachable!("handled by gen_plain_c_enum_impls"),
        Some(ReprKind::Transparent) => false,
    };
    let layout = if repr.is_some() {
        gen_enum_layout_family(generics, &fields)
    } else {
        gen_rust_enum_layout_family()
    };
    let size = AggregateFamily::fixed(quote! {
        #crate_::size::Sized<#crate_::Gt<#crate_::Zero>>
    });
    let tag = match repr {
        Some(ReprKind::Primitive(tag) | ReprKind::C(Some(tag))) => Some(tag.as_ref().clone()),
        Some(ReprKind::C(None)) => unreachable!("handled by gen_plain_c_enum_impls"),
        Some(ReprKind::Transparent) => None,
        None => Some(infer_repr(variants.len())),
    };
    let alignment = apply_repr_alignment(
        gen_alignment_family(generics, &fields, tag.as_ref()),
        repr_alignment,
    );
    let trap = gen_trap_family(generics, &fields, has_trap_tag_values);

    let niche = gen_enum_niche_family(has_trap_tag_values);
    let mutability = if matches!(repr, None | Some(ReprKind::Transparent)) && variants.len() == 1 {
        gen_single_field_mutability_family(generics, &fields)
    } else {
        gen_exclusive_mutability_family()
    };
    let indirect_trap = gen_indirect_trap_family(generics, &fields);

    let spec = TypeSpecFamilies {
        layout,
        size,
        alignment,
        trap,
        niche,
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
        gen_aggregate_rust_layout_family()
    };
    let size = gen_size_family(generics, &fields);
    let alignment = apply_repr_alignment(
        gen_alignment_family(generics, &fields, None),
        repr_alignment,
    );
    let trap = AggregateFamily::fixed(quote! { #crate_::layout::Robust });
    let niche = AggregateFamily::fixed(quote! {
        #crate_::niche::WithoutNiche
    });
    let mutability = gen_exclusive_mutability_family();
    let indirect_trap = gen_indirect_trap_family(generics, &fields);

    let spec = TypeSpecFamilies {
        layout,
        size,
        alignment,
        trap,
        niche,
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
) -> proc_macro2::TokenStream {
    if matches!(repr, Some(ReprKind::C(None))) {
        return gen_plain_c_enum_impls(repr_alignment, name, generics, variants);
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
            None => rust_tag_has_traps(variants.len()),
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
            None => rust_tag_has_traps(variants.len()),
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
            Some(ReprKind::Transparent) => infer_repr(variants.len()),
            None => infer_repr(variants.len()),
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
    let indirect_trap = AggregateFamily::fixed(quote! { #crate_::layout::Robust });

    let spec = TypeSpecFamilies {
        layout,
        size,
        alignment,
        trap,
        niche,
        mutability,
        indirect_trap,
    };

    gen_type_spec_impl(name, generics, &[], spec, quote! {})
}

fn gen_aggregate_rust_layout_family() -> AggregateFamily {
    let crate_ = crate_path();
    AggregateFamily::fixed(quote! { #crate_::Unstable })
}

fn gen_rust_enum_layout_family() -> AggregateFamily {
    let crate_ = crate_path();
    AggregateFamily::fixed(quote! { #crate_::Unstable })
}

fn gen_stable_layout_family(generics: &syn::Generics, fields: &[&syn::Type]) -> AggregateFamily {
    let crate_ = crate_path();

    AggregateFamily::fold(
        quote! { Layout },
        quote! { #crate_::Stable },
        generics,
        fields,
    )
}

fn gen_enum_layout_family(generics: &syn::Generics, fields: &[&syn::Type]) -> AggregateFamily {
    gen_stable_layout_family(generics, fields)
}

fn gen_plain_c_enum_impls(
    repr_alignment: Option<usize>,
    name: &syn::Ident,
    generics: &syn::Generics,
    variants: &Punctuated<syn::Variant, Token![,]>,
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
            mutability: gen_exclusive_mutability_family(),
            indirect_trap: gen_indirect_trap_family(generics, &fields),
        };

        gen_type_spec_impl(name, generics, &fields, spec, cfg)
    })
    .collect()
}

fn gen_indirect_trap_family(generics: &syn::Generics, fields: &[&syn::Type]) -> AggregateFamily {
    let crate_ = crate_path();
    AggregateFamily::from_fields(
        quote! { __IndirectTrap },
        quote! { #crate_::layout::Robust },
        generics,
        fields,
    )
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
    AggregateFamily::fold(quote! { Trap }, init, generics, fields)
}

fn gen_size_family(generics: &syn::Generics, fields: &[&syn::Type]) -> AggregateFamily {
    let crate_ = crate_path();

    AggregateFamily::from_fields(
        quote! { Size },
        quote! { #crate_::size::Sized<#crate_::Zero> },
        generics,
        fields,
    )
}

fn gen_alignment_family(
    generics: &syn::Generics,
    fields: &[&syn::Type],
    tag: Option<&syn::Type>,
) -> AggregateFamily {
    let crate_ = crate_path();
    let Some((&first, rest)) = fields.split_first() else {
        return AggregateFamily::fixed(tag.map_or_else(
            || quote! { #crate_::One },
            |tag| quote! { <#tag as #crate_::RustSpec>::Alignment },
        ));
    };

    let mut kind = field_axis_kind(first, 0, &quote! { Alignment }, generics);
    let mut aggregate_bounds = Vec::new();
    let mut accumulated_is_parameterized = field_needs_bounds(first, generics);
    for (index, &field) in rest.iter().enumerate() {
        let index = index + 1;
        let field_kind = field_axis_kind(field, index, &quote! { Alignment }, generics);
        let field_is_parameterized = field_needs_bounds(field, generics);
        if accumulated_is_parameterized || field_is_parameterized {
            aggregate_bounds.push(quote! {
                #kind: #crate_::Max<#field_kind>
            });
        }
        kind = quote! { <#kind as #crate_::Max<#field_kind>>::Output };
        accumulated_is_parameterized |= field_is_parameterized;
    }

    if let Some(tag) = tag {
        let tag_kind = quote! { <#tag as #crate_::RustSpec>::Alignment };
        kind = quote! { <#tag_kind as #crate_::Max<#kind>>::Output };
    }

    AggregateFamily {
        kind,
        aggregate_bounds,
    }
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
    AggregateFamily::fold(
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
        [field] => AggregateFamily::fixed(field_axis_kind(field, 0, &quote! { Niche }, generics)),
        _ => gen_niche_family(generics, fields),
    }
}

fn gen_single_field_mutability_family(
    generics: &syn::Generics,
    fields: &[&syn::Type],
) -> AggregateFamily {
    let crate_ = crate_path();

    match fields {
        [_] => AggregateFamily::from_fields(
            quote! { Mutability },
            quote! { #crate_::mutability::Exclusive },
            generics,
            fields,
        ),
        _ => gen_exclusive_mutability_family(),
    }
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
        .enumerate()
        .filter_map(|(index, field)| {
            let type_bound = field_has_type_params(field, generics)
                .then(|| quote! { #field: #crate_::RustSpec });
            let hrtb_bound = hrtb_projection(field, generics)
                .map(|_| quote! { Self: #crate_::__HrtbAxes<#index> });
            type_bound.or(hrtb_bound)
        })
        .collect::<Vec<_>>();

    let hrtb_axes_impls = fields
        .iter()
        .enumerate()
        .filter_map(|(index, field)| hrtb_axes_impl(name, generics, field, index))
        .collect::<Vec<_>>();

    let aggregate_bounds = layout
        .aggregate_bounds
        .into_iter()
        .chain(size.aggregate_bounds)
        .chain(alignment.aggregate_bounds)
        .chain(trap.aggregate_bounds)
        .chain(niche.aggregate_bounds)
        .chain(mutability.aggregate_bounds)
        .chain(indirect_trap.aggregate_bounds)
        .collect::<Vec<_>>();

    let layout_kind = layout.kind;
    let size_kind = size.kind;
    let alignment_kind = alignment.kind;
    let trap_kind = trap.kind;
    let niche_kind = niche.kind;
    let mutability_kind = mutability.kind;
    let indirect_trap_kind = indirect_trap.kind;

    quote! {
        #(#hrtb_axes_impls)*

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
            type Mutability = #mutability_kind;
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

fn rust_tag_has_traps(variant_count: usize) -> bool {
    let tag = infer_repr(variant_count);
    primitive_tag_has_traps(&tag, variant_count)
}

fn c_tag_type(c_enum_bits: u32) -> syn::Type {
    match c_enum_bits {
        8 => syn::parse_quote!(u8),
        16 => syn::parse_quote!(u16),
        32 => syn::parse_quote!(u32),
        _ => unreachable!("unsupported C enum width"),
    }
}
