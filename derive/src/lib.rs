use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Token, punctuated::Punctuated};

use crate::{
    aggregate::AggregateFamily,
    repr::{ReprKind, enum_tag_type, is_exhaustive_enum, is_transparent_enum_repr, parse_repr},
};

mod aggregate;
mod repr;
mod wide;

#[proc_macro_derive(RustSpec)]
pub fn rust_spec(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn family_path() -> proc_macro2::TokenStream {
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

fn expand(input: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let family = family_path();
    let repr = parse_repr(&input.attrs)?;

    let rust_spec_impl = match &input.data {
        syn::Data::Struct(data) => gen_struct_impl(
            &family,
            repr.as_ref(),
            &input.ident,
            &input.generics,
            &data.fields,
        ),
        syn::Data::Enum(data)
            if data
                .variants
                .iter()
                .all(|v| matches!(v.fields, syn::Fields::Unit)) =>
        {
            gen_fieldless_enum_impl(
                &family,
                repr.as_ref(),
                &input.ident,
                &input.generics,
                &data.variants,
            )
        }
        syn::Data::Enum(data) if is_transparent_enum_repr(repr.as_ref(), &data.variants) => {
            let Some(variant) = data.variants.first() else {
                return Ok(quote! {});
            };
            gen_struct_impl(
                &family,
                repr.as_ref(),
                &input.ident,
                &input.generics,
                &variant.fields,
            )
        }
        syn::Data::Enum(data) => gen_enum_impl(
            &family,
            repr.as_ref(),
            &input.ident,
            &input.generics,
            &data.variants,
        ),
        syn::Data::Union(_) => {
            return Err(syn::Error::new_spanned(input, "Unions are not supported"));
        }
    };
    let wide_impl = if matches!(repr, Some(ReprKind::Transparent)) {
        wide::expand(&family, input)?
    } else {
        quote! {}
    };

    Ok(quote! {
        #rust_spec_impl
        #wide_impl
    })
}

fn gen_struct_impl(
    family: &proc_macro2::TokenStream,
    repr: Option<&ReprKind>,
    name: &syn::Ident,
    generics: &syn::Generics,
    fields: &syn::Fields,
) -> proc_macro2::TokenStream {
    let fields = fields.iter().map(|f| &f.ty).collect::<Vec<_>>();
    let layout = if repr.is_some() {
        gen_stable_layout_family(family, generics, &fields, false)
    } else {
        gen_rust_layout_family(family, generics, &fields)
    };
    let size = gen_size_family(family, generics, &fields);
    let niche = gen_niche_family(family, generics, &fields);
    let mutability = gen_mutability_family(family, generics, &fields);

    gen_type_spec_impl(family, name, generics, layout, size, niche, mutability)
}

fn gen_enum_impl(
    family: &proc_macro2::TokenStream,
    repr: Option<&ReprKind>,
    name: &syn::Ident,
    generics: &syn::Generics,
    variants: &Punctuated<syn::Variant, Token![,]>,
) -> proc_macro2::TokenStream {
    let fields = variants
        .iter()
        .flat_map(|variant| variant.fields.iter().map(|field| &field.ty))
        .collect::<Vec<_>>();

    let layout = if repr.is_some() {
        let has_uninhabited_tag_values = enum_tag_type(repr, variants.len())
            .is_some_and(|tag| !is_exhaustive_enum(variants.len(), &tag));
        gen_stable_layout_family(family, generics, &fields, has_uninhabited_tag_values)
    } else {
        gen_rust_layout_family(family, generics, &fields)
    };
    // FIXME: Sometimes enums with variants are ZSTs and don't have a tag
    // This happens if all variants are uninhabited but one is ZST/fieldless.
    let size = AggregateFamily::fixed(quote! { #family::size::Sized<#family::size::NonZst> });
    let niche = gen_enum_niche_family(family, repr, variants);
    let mutability = gen_mutability_family(family, generics, &fields);

    gen_type_spec_impl(family, name, generics, layout, size, niche, mutability)
}

fn gen_fieldless_enum_impl(
    family: &proc_macro2::TokenStream,
    repr: Option<&ReprKind>,
    name: &syn::Ident,
    generics: &syn::Generics,
    variants: &Punctuated<syn::Variant, Token![,]>,
) -> proc_macro2::TokenStream {
    let layout_kind = match repr {
        None => quote! { #family::layout::Unstable<#family::layout::Robust> },
        Some(ReprKind::C(None)) => unreachable!(),
        Some(ReprKind::Transparent) => quote! { #family::layout::Robust },
        Some(ReprKind::C(Some(tag)) | ReprKind::Primitive(tag)) => {
            let robustness = if is_exhaustive_enum(variants.len(), tag) {
                quote! { #family::layout::Robust }
            } else {
                quote! { #family::layout::NonRobust }
            };

            quote! { #family::layout::Stable<#robustness> }
        }
    };
    let tag_type = if repr.is_none() && variants.len() == 1 {
        None
    } else {
        enum_tag_type(repr, variants.len())
    };
    let size = if tag_type.is_none() {
        gen_size_family(family, generics, &[])
    } else {
        AggregateFamily::fixed(quote! { #family::size::Sized<#family::size::NonZst> })
    };
    let niche = if tag_type.is_none() {
        AggregateFamily::fixed(quote! { #family::niche::WithoutNiche })
    } else {
        gen_enum_niche_family(family, repr, variants)
    };
    let mutability = gen_mutability_family(family, generics, &[]);
    let layout = AggregateFamily::fixed(layout_kind);

    gen_type_spec_impl(family, name, generics, layout, size, niche, mutability)
}

fn gen_rust_layout_family(
    family: &proc_macro2::TokenStream,
    generics: &syn::Generics,
    fields: &[&syn::Type],
) -> AggregateFamily {
    AggregateFamily::fold(
        family,
        quote! { Layout },
        quote! { #family::layout::Unstable<#family::layout::Robust> },
        generics,
        fields,
    )
}

fn gen_stable_layout_family(
    family: &proc_macro2::TokenStream,
    generics: &syn::Generics,
    fields: &[&syn::Type],
    has_uninhabited_tag_values: bool,
) -> AggregateFamily {
    let init = if has_uninhabited_tag_values {
        quote! { #family::layout::NonRobust }
    } else {
        quote! { #family::layout::Robust }
    };
    AggregateFamily::fold(
        family,
        quote! { Layout },
        quote! { #family::layout::Stable<#init> },
        generics,
        fields,
    )
}

fn gen_size_family(
    family: &proc_macro2::TokenStream,
    generics: &syn::Generics,
    fields: &[&syn::Type],
) -> AggregateFamily {
    AggregateFamily::fold(
        family,
        quote! { Size },
        quote! { #family::size::Sized<#family::size::Zst> },
        generics,
        fields,
    )
}

fn gen_niche_family(
    family: &proc_macro2::TokenStream,
    generics: &syn::Generics,
    fields: &[&syn::Type],
) -> AggregateFamily {
    AggregateFamily::fold(
        family,
        quote! { Niche },
        // TODO: We're just using WithoutNiche for the ease of implementation. Remove it?
        quote! { #family::niche::WithoutNiche },
        generics,
        fields,
    )
}

fn gen_mutability_family(
    family: &proc_macro2::TokenStream,
    generics: &syn::Generics,
    fields: &[&syn::Type],
) -> AggregateFamily {
    let init_kind = if fields.is_empty() {
        quote! { #family::mutability::Exclusive }
    } else {
        quote! { #family::mutability::Interior }
    };

    AggregateFamily::fold(family, quote! { Mutability }, init_kind, generics, fields)
}

fn gen_type_spec_impl(
    family: &proc_macro2::TokenStream,
    name: &syn::Ident,
    generics: &syn::Generics,
    layout: AggregateFamily,
    size: AggregateFamily,
    niche: AggregateFamily,
    mutability: AggregateFamily,
) -> proc_macro2::TokenStream {
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
        unsafe impl #impl_generics #family::RustSpec for #name #ty_generics where
            #(#field_bounds,)*
            #(#aggregate_bounds,)*
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
    family: &proc_macro2::TokenStream,
    repr: Option<&ReprKind>,
    variants: &Punctuated<syn::Variant, Token![,]>,
) -> AggregateFamily {
    let is_exhaustive = enum_tag_type(repr, variants.len())
        .is_none_or(|tag| is_exhaustive_enum(variants.len(), &tag));
    let niche_kind = if is_exhaustive {
        quote! { #family::niche::WithoutNiche }
    } else {
        quote! { #family::niche::WithNiche<#family::niche::Unstable> }
    };

    AggregateFamily::fixed(niche_kind)
}
