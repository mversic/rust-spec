use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Token, punctuated::Punctuated};

use crate::repr::{ReprKind, infer_repr, is_exhaustive_enum, parse_repr};

mod repr;

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
            .enumerate()
            .map(|(index, &field)| (index, field))
            .partition(|(_, field)| !field_needs_bounds(field, generics));

        let mut concrete_fields = concrete_fields.into_iter();
        let concrete_kind = concrete_fields.next().map(|(index, field)| {
            let mut kind = field_axis_kind(field, index, &axis, generics);

            for (index, field) in concrete_fields {
                let field_kind = field_axis_kind(field, index, &axis, generics);
                kind = quote! { <#kind as #operator<#field_kind>>::Output };
            }

            kind
        });

        let mut parameterized_fields = parameterized_fields.into_iter();
        let parameterized_kind = parameterized_fields.next().map(|(index, field)| {
            let mut kind = field_axis_kind(field, index, &axis, generics);

            for (index, field) in parameterized_fields {
                let field_kind = field_axis_kind(field, index, &axis, generics);

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

fn hrtb_projection_bound(field: &syn::Type, generics: &syn::Generics) -> Vec<syn::WherePredicate> {
    use syn::visit::Visit;

    #[derive(Default)]
    struct ProjectionVisitor<'a> {
        projections: Vec<&'a syn::TypePath>,
    }

    impl<'ast> Visit<'ast> for ProjectionVisitor<'ast> {
        fn visit_type_path(&mut self, type_path: &'ast syn::TypePath) {
            if type_path.qself.is_some() {
                self.projections.push(type_path);
                for segment in &type_path.path.segments {
                    self.visit_path_arguments(&segment.arguments);
                }
                return;
            }
            syn::visit::visit_type_path(self, type_path);
        }
    }

    let mut visitor = ProjectionVisitor::default();
    visitor.visit_type(field);
    let crate_ = crate_path();

    let mut bounds = Vec::new();
    for projection in &visitor.projections {
        let Some(qself) = projection.qself.as_ref() else {
            continue;
        };
        let trait_path = syn::Path {
            leading_colon: projection.path.leading_colon,
            segments: projection
                .path
                .segments
                .iter()
                .take(qself.position)
                .cloned()
                .collect(),
        };

        for predicate in generics
            .where_clause
            .iter()
            .flat_map(|where_clause| &where_clause.predicates)
        {
            let syn::WherePredicate::Type(predicate) = predicate else {
                continue;
            };
            if predicate.lifetimes.is_none()
                || predicate.bounded_ty.to_token_stream().to_string()
                    != qself.ty.to_token_stream().to_string()
            {
                continue;
            }

            let Some(bound_index) = predicate.bounds.iter().position(|bound| {
                matches!(bound, syn::TypeParamBound::Trait(bound) if bound.path.to_token_stream().to_string() == trait_path.to_token_stream().to_string())
            }) else {
                continue;
            };
            let Some(associated_type) = projection.path.segments.iter().nth(qself.position) else {
                continue;
            };
            let mut predicate = predicate.clone();
            let syn::TypeParamBound::Trait(bound) = &mut predicate.bounds[bound_index] else {
                unreachable!("matched a trait bound");
            };
            *bound = syn::parse_quote! { #trait_path<#associated_type: #crate_::RustSpec> };

            bounds.push(syn::WherePredicate::Type(predicate));
        }
    }

    bounds
}

fn hrtb_lifetimes(field: &syn::Type, generics: &syn::Generics) -> Vec<syn::BoundLifetimes> {
    hrtb_projection_bound(field, generics)
        .into_iter()
        .filter_map(|predicate| match predicate {
            syn::WherePredicate::Type(predicate) => predicate.lifetimes,
            _ => None,
        })
        .collect()
}

fn field_axis_kind(
    field: &syn::Type,
    index: usize,
    axis: &proc_macro2::TokenStream,
    generics: &syn::Generics,
) -> proc_macro2::TokenStream {
    let crate_ = crate_path();

    if !hrtb_projection_bound(field, generics).is_empty() {
        return quote! { <Self as #crate_::__HrtbAxes<#index>>::#axis<'static> };
    }

    quote! { <#field as #crate_::RustSpec>::#axis }
}

fn hrtb_axes_impl(
    name: &syn::Ident,
    generics: &syn::Generics,
    field: &syn::Type,
    index: usize,
) -> Option<proc_macro2::TokenStream> {
    let hrtb_bounds = hrtb_projection_bound(field, generics);

    let crate_ = crate_path();
    if hrtb_bounds.is_empty() {
        return None;
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let predicates = where_clause.map(|w| &w.predicates);

    Some(quote! {
        #[doc(hidden)]
        impl #impl_generics #crate_::__HrtbAxes<#index> for #name #ty_generics
        where
            #(#hrtb_bounds,)*
            #predicates
        {
            type Layout<'__rust_spec> = <#field as #crate_::RustSpec>::Layout;
            type Size<'__rust_spec> = <#field as #crate_::RustSpec>::Size;
            type Alignment<'__rust_spec> = <#field as #crate_::RustSpec>::Alignment;
            type Trap<'__rust_spec> = <#field as #crate_::RustSpec>::Trap;
            type Niche<'__rust_spec> = <#field as #crate_::RustSpec>::Niche;
            type Mutability<'__rust_spec> = <#field as #crate_::RustSpec>::Mutability;
            type __IndirectTrap<'__rust_spec> = <#field as #crate_::RustSpec>::__IndirectTrap;
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
    field_has_type_params(field, generics) || !hrtb_projection_bound(field, generics).is_empty()
}

fn expand(input: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let custom_niche = parse_custom_niche_attr(&input.attrs)?;
    let repr = parse_repr(&input.attrs)?;
    let alignment = repr.align;
    let repr = repr.kind.as_ref();
    let name = &input.ident;
    let generics = &input.generics;
    let rust_spec_impl = match &input.data {
        syn::Data::Struct(data) => {
            gen_struct_impl(repr, alignment, name, generics, &data.fields, custom_niche)
        }
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

            gen_struct_impl(
                repr,
                alignment,
                name,
                generics,
                &variant.fields,
                custom_niche,
            )
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
                custom_niche,
            )
        }
        syn::Data::Union(data) => gen_union_impl(repr, alignment, name, generics, &data.fields),
    };

    Ok(quote! { #rust_spec_impl })
}

fn parse_custom_niche_attr(attrs: &[syn::Attribute]) -> syn::Result<bool> {
    let mut has_niche = false;

    for attr in attrs
        .iter()
        .filter(|attr| attr.path().is_ident("rust_spec"))
    {
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("with_custom_niche") {
                return Err(meta.error("unknown rust_spec attribute"));
            }

            if has_niche {
                return Err(meta.error("duplicate `with_custom_niche` within attribute"));
            }
            has_niche = true;
            Ok(())
        })?;
    }

    Ok(has_niche)
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
    custom_niche: bool,
) -> proc_macro2::TokenStream {
    let fields = field_types(fields);

    gen_struct_fields_impl(repr, repr_alignment, name, generics, &fields, custom_niche)
}

fn gen_struct_fields_impl(
    repr: Option<&ReprKind>,
    alignment: Option<usize>,
    name: &syn::Ident,
    generics: &syn::Generics,
    fields: &[&syn::Type],
    custom_niche: bool,
) -> proc_macro2::TokenStream {
    let layout = if repr.is_some() {
        gen_stable_layout_family(generics, fields)
    } else {
        gen_unstable_layout_family()
    };
    let size = gen_size_family(generics, fields);
    let alignment = apply_repr_alignment(gen_alignment_family(generics, fields, None), alignment);
    let trap = gen_trap_family(generics, fields, false);
    let niche = if custom_niche {
        let crate_ = crate_path();
        AggregateFamily::fixed(quote! { #crate_::niche::WithNiche<#crate_::Unstable> })
    } else if let Some(ReprKind::Transparent) = repr {
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
        gen_unstable_layout_family()
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

    let mutability = gen_exclusive_mutability_family();
    let indirect_trap = AggregateFamily::fixed(quote! {
        #crate_::layout::Robust
    });

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

    AggregateFamily::aggregate(
        quote! { core::ops::Add },
        quote! { __IndirectTrap },
        generics,
        fields,
    )
    .unwrap_or_else(|| AggregateFamily::fixed(quote! { #crate_::layout::Robust }))
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
        [field] => AggregateFamily::fixed(field_axis_kind(field, 0, &quote! { Niche }, generics)),
        _ => gen_niche_family(generics, fields),
    }
}

fn gen_single_field_mutability_family(
    generics: &syn::Generics,
    fields: &[&syn::Type],
) -> AggregateFamily {
    match fields {
        [field] => {
            AggregateFamily::fixed(field_axis_kind(field, 0, &quote! { Mutability }, generics))
        }
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
    let hrtb_axes = fields
        .iter()
        .filter_map(|field| {
            let lifetimes = hrtb_lifetimes(field, generics);
            (!lifetimes.is_empty()).then_some(lifetimes)
        })
        .collect::<Vec<_>>();

    let field_bounds = fields.iter().enumerate().flat_map(|(index, field)| {
        if field_has_type_params(field, generics) {
            vec![quote! { #field: #crate_::RustSpec }]
        } else {
            hrtb_lifetimes(field, generics)
                .into_iter()
                .map(|lifetimes| quote! { #lifetimes Self: #crate_::__HrtbAxes<#index> })
                .collect()
        }
    });

    let hrtb_axes_impls = fields
        .iter()
        .enumerate()
        .filter_map(|(index, field)| hrtb_axes_impl(name, generics, field, index));

    let aggregate_bounds = layout
        .aggregate_bounds
        .into_iter()
        .chain(size.aggregate_bounds)
        .chain(alignment.aggregate_bounds)
        .chain(trap.aggregate_bounds)
        .chain(niche.aggregate_bounds)
        .chain(mutability.aggregate_bounds)
        .chain(indirect_trap.aggregate_bounds)
        .flat_map(|bound| {
            if !bound.to_string().contains("__HrtbAxes") {
                return vec![bound.clone()];
            }

            hrtb_axes
                .iter()
                .flatten()
                .map(|lifetimes| quote! { #lifetimes #bound })
                .collect()
        });

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
