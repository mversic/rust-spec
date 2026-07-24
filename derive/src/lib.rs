use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Token, punctuated::Punctuated};

use crate::repr::{ReprKind, enum_tag_type, is_exhaustive_enum, parse_repr};

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
    field_bounds: Vec<proc_macro2::TokenStream>,
    aggregate_bounds: Vec<proc_macro2::TokenStream>,
}

impl AggregateFamily {
    fn fixed(kind: proc_macro2::TokenStream) -> Self {
        Self {
            kind,
            field_bounds: Vec::new(),
            aggregate_bounds: Vec::new(),
        }
    }

    fn fold(
        axis: proc_macro2::TokenStream,
        init_kind: proc_macro2::TokenStream,
        generics: &syn::Generics,
        fields: &[&syn::Type],
    ) -> Self {
        let crate_ = crate_path();
        let (parametrized_fields, non_parametrized_fields): (Vec<&syn::Type>, Vec<_>) = fields
            .iter()
            .partition(|ty| is_type_parameterized(ty, generics));
        let mut kind = init_kind;
        let field_bounds = parametrized_fields
            .iter()
            .map(|ty| quote! { #ty: #crate_::RustSpec })
            .collect::<Vec<_>>();
        let mut aggregate_bounds = Vec::new();

        for &field in &non_parametrized_fields {
            let field_kind = quote! { <#field as #crate_::RustSpec>::#axis };
            kind = quote! { <#kind as core::ops::Add<#field_kind>>::Output };
        }

        for &field in &parametrized_fields {
            let field_kind = quote! { <#field as #crate_::RustSpec>::#axis };
            aggregate_bounds.push(quote! { #field_kind: core::ops::Add<#kind> });
            kind = quote! { <#field_kind as core::ops::Add<#kind>>::Output };
        }

        Self {
            kind,
            field_bounds,
            aggregate_bounds,
        }
    }
}

fn is_type_parameterized(ty: &syn::Type, generics: &syn::Generics) -> bool {
    use syn::visit::Visit;

    struct TypeParamVisitor<'a> {
        type_params: &'a [&'a syn::Ident],
        is_generic: bool,
    }

    impl Visit<'_> for TypeParamVisitor<'_> {
        fn visit_type_path(&mut self, type_path: &syn::TypePath) {
            if type_path.qself.is_none()
                && let Some(first_segment) = type_path.path.segments.first()
                && self.type_params.contains(&&first_segment.ident)
            {
                self.is_generic = true;
            }

            syn::visit::visit_type_path(self, type_path);
        }
    }

    let type_param_idents = generics
        .type_params()
        .map(|param| &param.ident)
        .collect::<Vec<_>>();
    let mut visitor = TypeParamVisitor {
        type_params: &type_param_idents,
        is_generic: false,
    };

    visitor.visit_type(ty);
    visitor.is_generic
}

fn expand(input: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let repr = parse_repr(&input.attrs)?;
    let repr = repr.as_ref();
    let name = &input.ident;
    let generics = &input.generics;

    let rust_spec_impl = match &input.data {
        syn::Data::Struct(data) => gen_struct_impl(repr, name, generics, &data.fields),
        syn::Data::Union(data) => gen_union_impl(repr, name, generics, &data.fields),
        syn::Data::Enum(data) if is_fieldless_enum(data) => {
            gen_fieldless_enum_impl(repr, name, generics, &data.variants)
        }
        syn::Data::Enum(data)
            if matches!(repr, Some(ReprKind::Transparent))
                || repr.is_none() && data.variants.len() == 1 =>
        {
            let Some(variant) = data.variants.first() else {
                return Ok(quote! {});
            };
            gen_struct_impl(repr, name, generics, &variant.fields)
        }
        syn::Data::Enum(data) => gen_enum_impl(repr, name, generics, &data.variants),
    };
    Ok(quote! {
        #rust_spec_impl
    })
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
    name: &syn::Ident,
    generics: &syn::Generics,
    fields: &syn::Fields,
) -> proc_macro2::TokenStream {
    let fields = field_types(fields);

    let layout = if repr.is_some() {
        gen_stable_layout_family(generics, &fields, false)
    } else {
        gen_rust_layout_family(generics, &fields)
    };

    let size = gen_size_family(generics, &fields);
    let niche = gen_niche_family(generics, &fields);
    let mutability = gen_mutability_family(generics, &fields);

    gen_type_spec_impl(name, generics, layout, size, niche, mutability)
}

fn gen_enum_impl(
    repr: Option<&ReprKind>,
    name: &syn::Ident,
    generics: &syn::Generics,
    variants: &Punctuated<syn::Variant, Token![,]>,
) -> proc_macro2::TokenStream {
    let crate_ = crate_path();
    let fields = variant_field_types(variants);
    let layout = if repr.is_some() {
        let has_trap_tag_values = enum_tag_type(repr, variants.len())
            .is_some_and(|tag| !is_exhaustive_enum(variants.len(), &tag));

        gen_stable_layout_family(generics, &fields, has_trap_tag_values)
    } else {
        gen_rust_layout_family(generics, &fields)
    };

    let size = AggregateFamily::fixed(quote! {
        #crate_::size::Sized<#crate_::size::NonZst>
    });

    let niche = gen_enum_niche_family(repr, variants);
    let mutability = gen_mutability_family(generics, &fields);

    gen_type_spec_impl(name, generics, layout, size, niche, mutability)
}

fn gen_union_impl(
    repr: Option<&ReprKind>,
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
        gen_stable_layout_family(generics, &fields, false)
    } else {
        gen_rust_layout_family(generics, &fields)
    };

    let niche = AggregateFamily::fixed(quote! {
        #crate_::niche::WithoutNiche
    });

    let size = gen_size_family(generics, &fields);
    let mutability = gen_mutability_family(generics, &fields);

    gen_type_spec_impl(name, generics, layout, size, niche, mutability)
}

fn gen_fieldless_enum_impl(
    repr: Option<&ReprKind>,
    name: &syn::Ident,
    generics: &syn::Generics,
    variants: &Punctuated<syn::Variant, Token![,]>,
) -> proc_macro2::TokenStream {
    let crate_ = crate_path();
    let layout_kind = match repr {
        None => quote! { #crate_::layout::Unstable<#crate_::layout::Robust> },
        Some(ReprKind::C(None)) => unreachable!(),
        Some(ReprKind::Transparent) => quote! { #crate_::layout::Robust },
        Some(ReprKind::C(Some(tag)) | ReprKind::Primitive(tag)) => {
            let robustness = if is_exhaustive_enum(variants.len(), tag) {
                quote! { #crate_::layout::Robust }
            } else {
                quote! { #crate_::layout::NonRobust }
            };

            quote! { #crate_::layout::Stable<#robustness> }
        }
    };
    let tag_type =
        (repr.is_some() || variants.len() != 1).then(|| enum_tag_type(repr, variants.len()));

    let size = if tag_type.is_none() {
        gen_size_family(generics, &[])
    } else {
        AggregateFamily::fixed(quote! { #crate_::size::Sized<#crate_::size::NonZst> })
    };

    let niche = if tag_type.is_none() {
        AggregateFamily::fixed(quote! { #crate_::niche::WithoutNiche })
    } else {
        gen_enum_niche_family(repr, variants)
    };

    let mutability = gen_mutability_family(generics, &[]);
    let layout = AggregateFamily::fixed(layout_kind);

    gen_type_spec_impl(name, generics, layout, size, niche, mutability)
}

fn gen_rust_layout_family(generics: &syn::Generics, fields: &[&syn::Type]) -> AggregateFamily {
    let crate_ = crate_path();
    AggregateFamily::fold(
        quote! { Layout },
        quote! { #crate_::layout::Unstable<#crate_::layout::Robust> },
        generics,
        fields,
    )
}

fn gen_stable_layout_family(
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

    AggregateFamily::fold(
        quote! { Layout },
        quote! { #crate_::layout::Stable<#init> },
        generics,
        fields,
    )
}

fn gen_size_family(generics: &syn::Generics, fields: &[&syn::Type]) -> AggregateFamily {
    let crate_ = crate_path();
    AggregateFamily::fold(
        quote! { Size },
        quote! { #crate_::size::Sized<#crate_::size::Zst> },
        generics,
        fields,
    )
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

fn gen_mutability_family(generics: &syn::Generics, fields: &[&syn::Type]) -> AggregateFamily {
    let crate_ = crate_path();
    let init_kind = if fields.is_empty() {
        quote! { #crate_::mutability::Exclusive }
    } else {
        quote! { #crate_::mutability::Interior }
    };

    AggregateFamily::fold(quote! { Mutability }, init_kind, generics, fields)
}

fn gen_type_spec_impl(
    name: &syn::Ident,
    generics: &syn::Generics,
    layout: AggregateFamily,
    size: AggregateFamily,
    niche: AggregateFamily,
    mutability: AggregateFamily,
) -> proc_macro2::TokenStream {
    let crate_ = crate_path();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let predicates = where_clause.as_ref().map(|w| &w.predicates);
    let field_bounds = layout
        .field_bounds
        .into_iter()
        .chain(size.field_bounds)
        .chain(niche.field_bounds)
        .chain(mutability.field_bounds)
        .collect::<Vec<_>>();
    let aggregate_bounds = layout
        .aggregate_bounds
        .into_iter()
        .chain(size.aggregate_bounds)
        .chain(niche.aggregate_bounds)
        .chain(mutability.aggregate_bounds)
        .collect::<Vec<_>>();
    let layout_kind = layout.kind;
    let size_kind = size.kind;
    let niche_kind = niche.kind;
    let mutability_kind = mutability.kind;

    quote! {
        unsafe impl #impl_generics #crate_::RustSpec for #name #ty_generics where
            #(#field_bounds,)*
            #(#aggregate_bounds,)*
            #layout_kind: #crate_::layout::LayoutSpec,
            #size_kind: #crate_::size::SizeSpec,
            #niche_kind: #crate_::niche::NicheSpec,
            #mutability_kind: #crate_::mutability::MutabilitySpec,
            #predicates
        {
            type Layout = #layout_kind;
            type Size = #size_kind;
            type Niche = #niche_kind;
            type Mutability = #mutability_kind;
        }
    }
}

fn gen_enum_niche_family(
    repr: Option<&ReprKind>,
    variants: &Punctuated<syn::Variant, Token![,]>,
) -> AggregateFamily {
    let crate_ = crate_path();
    let is_exhaustive = enum_tag_type(repr, variants.len())
        .is_none_or(|tag| is_exhaustive_enum(variants.len(), &tag));
    let niche_kind = if is_exhaustive {
        quote! { #crate_::niche::WithoutNiche }
    } else {
        quote! { #crate_::niche::WithNiche<#crate_::niche::Unstable> }
    };

    AggregateFamily::fixed(niche_kind)
}
