use proc_macro::TokenStream;
use proc_macro2::Delimiter;
use quote::{format_ident, quote};
use syn::{
    Attribute, Meta, Token,
    parse::{Parse, ParseStream, Parser as _},
    punctuated::Punctuated,
    visit::Visit,
};

#[derive(Clone)]
enum ReprKind {
    Transparent,
    C(Option<Box<syn::Type>>),
    Primitive(Box<syn::Type>),
}

enum ReprToken {
    Kind(ReprKind),
    Align,
}

impl Parse for ReprToken {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.step(|cursor| {
            let Some((ident, after_token)) = cursor.ident() else {
                return Err(cursor.error("Expected repr kind"));
            };

            match ident.to_string().as_str() {
                "transparent" => Ok((ReprToken::Kind(ReprKind::Transparent), after_token)),
                "C" => Ok((ReprToken::Kind(ReprKind::C(None)), after_token)),
                "u8" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(u8))),
                    after_token,
                )),
                "i8" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(i8))),
                    after_token,
                )),
                "u16" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(u16))),
                    after_token,
                )),
                "i16" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(i16))),
                    after_token,
                )),
                "u32" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(u32))),
                    after_token,
                )),
                "i32" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(i32))),
                    after_token,
                )),
                "u64" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(u64))),
                    after_token,
                )),
                "i64" => Ok((
                    ReprToken::Kind(ReprKind::Primitive(syn::parse_quote!(i64))),
                    after_token,
                )),
                "packed" => Ok((ReprToken::Align, after_token)),
                "align"
                    if let Some((_inside, _span, after_group)) =
                        after_token.group(Delimiter::Parenthesis) =>
                {
                    Ok((ReprToken::Align, after_group))
                }
                "align" => Ok((ReprToken::Align, after_token)),
                _ => Err(cursor.error("Unrecognized repr kind")),
            }
        })
    }
}

#[derive(Default)]
struct FamilyAttrs {
    has_custom_niche: bool,
    has_trap_values: bool,
    is_view: bool,
}

#[proc_macro_derive(TypeSpec, attributes(reprC))]
pub fn type_spec(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    expand_family(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_family(input: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let family = family_path();
    let repr = parse_repr(&input.attrs)?;
    let attrs = parse_family_attrs(&input.attrs)?;

    if attrs.is_view {
        return Ok(gen_view_delegate_impls(
            &family,
            &input.ident,
            &input.generics,
        ));
    }

    Ok(match &input.data {
        syn::Data::Struct(data) => gen_struct_family_impls(
            &family,
            repr.as_ref(),
            &input.ident,
            &input.generics,
            &data.fields,
            attrs.has_trap_values,
            attrs.has_custom_niche,
        ),
        syn::Data::Enum(data)
            if data
                .variants
                .iter()
                .all(|v| matches!(v.fields, syn::Fields::Unit)) =>
        {
            gen_fieldless_enum_family_impls(
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
            let variant_attrs = parse_family_attrs(&variant.attrs)?;
            gen_struct_family_impls(
                &family,
                repr.as_ref(),
                &input.ident,
                &input.generics,
                &variant.fields,
                variant_attrs.has_trap_values,
                false,
            )
        }
        syn::Data::Enum(data) => gen_enum_family_impls(
            &family,
            repr.as_ref(),
            &input.ident,
            &input.generics,
            &data.variants,
        ),
        syn::Data::Union(_) => {
            return Err(syn::Error::new_spanned(input, "Unions are not supported"));
        }
    })
}

fn family_path() -> proc_macro2::TokenStream {
    let rust_spec = proc_macro_crate::crate_name("rust-spec");
    match rust_spec {
        Ok(proc_macro_crate::FoundCrate::Itself) => return quote! { crate },
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

fn parse_family_attrs(attrs: &[Attribute]) -> syn::Result<FamilyAttrs> {
    let mut output = FamilyAttrs::default();

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("reprC")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("NICHE_VALUE") {
                let _: syn::Expr = meta.value()?.parse()?;
                output.has_custom_niche = true;
                return Ok(());
            }

            if meta.path.is_ident("is_valid") {
                let _: syn::ExprClosure = meta.value()?.parse()?;
                output.has_trap_values = true;
                return Ok(());
            }

            if meta.path.is_ident("unsafe") {
                meta.parse_nested_meta(|meta| {
                    if meta.path.is_ident("is_valid") {
                        let _: syn::ExprClosure = meta.value()?.parse()?;
                        output.has_trap_values = true;
                        return Ok(());
                    }

                    Err(meta.error("unknown unsafe type kind"))
                })?;
                return Ok(());
            }

            if meta.path.is_ident("id") {
                let content;
                syn::parenthesized!(content in meta.input);
                let _: syn::Type = content.parse()?;
                return Ok(());
            }

            if meta.path.is_ident("view") {
                output.is_view = true;
                return Ok(());
            }

            Err(meta.error("unknown type kind"))
        })?;
    }

    Ok(output)
}

fn parse_repr(attrs: &[Attribute]) -> syn::Result<Option<ReprKind>> {
    let repr_attrs = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("repr"))
        .collect::<Vec<_>>();

    if repr_attrs.len() > 1 {
        return Err(syn::Error::new_spanned(
            repr_attrs[1],
            "Multiple repr attributes",
        ));
    }

    let Some(&attr) = repr_attrs.first() else {
        return Ok(None);
    };

    let Meta::List(list) = &attr.meta else {
        return Ok(None);
    };

    let tokens =
        Punctuated::<ReprToken, Token![,]>::parse_terminated.parse2(list.tokens.clone())?;
    let mut kind = None;

    for token in tokens {
        match token {
            ReprToken::Kind(new_kind) => match (&mut kind, new_kind) {
                (Some(ReprKind::C(None)), ReprKind::Primitive(prim)) => {
                    kind = Some(ReprKind::C(Some(prim)));
                }
                (Some(ReprKind::Primitive(prim)), ReprKind::C(None)) => {
                    kind = Some(ReprKind::C(Some(prim.clone())));
                }
                (Some(_), _) => {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "Duplicate repr kind within attribute",
                    ));
                }
                (None, new_kind) => kind = Some(new_kind),
            },
            ReprToken::Align => {}
        }
    }

    Ok(kind)
}

fn gen_view_delegate_impls(
    family: &proc_macro2::TokenStream,
    name: &syn::Ident,
    generics: &syn::Generics,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let predicates = where_clause.as_ref().map(|w| &w.predicates);

    let has_view_lifetime = matches!(
        generics.params.first(),
        Some(syn::GenericParam::Lifetime(param)) if param.lifetime.ident == "_dšč"
    );
    let owner_ty_generics =
        generic_param_idents(generics.params.iter().skip(has_view_lifetime as usize))
            .collect::<Vec<_>>();

    let owner_name = gen_view_owner_name(name);
    let owner_ty = quote! { #owner_name<#(#owner_ty_generics),*> };

    let for_dummy = if owner_ty_generics.is_empty() {
        quote! { for<'_dummy> }
    } else {
        quote! {}
    };

    quote! {
        impl #impl_generics #family::repr::ReprFamily for #name #ty_generics
        where
            #owner_ty: #family::repr::ReprFamily,
            #predicates
        {
            type Kind = <#owner_ty as #family::repr::ReprFamily>::Kind;
        }

        unsafe impl #impl_generics #family::size::SizeFamily for #name #ty_generics
        where
            #owner_ty: #family::size::SizeFamily,
            #predicates
        {
            type Kind = <#owner_ty as #family::size::SizeFamily>::Kind;
        }

        impl #impl_generics #family::niche::NicheFamily for #name #ty_generics
        where
            #for_dummy #owner_ty: #family::niche::NicheFamily,
            #predicates
        {
            type Kind = <#owner_ty as #family::niche::NicheFamily>::Kind;
        }

        impl #impl_generics #family::mutability::MutabilityFamily for #name #ty_generics
        where
            #owner_ty: #family::mutability::MutabilityFamily,
            #predicates
        {
            type Kind = <#owner_ty as #family::mutability::MutabilityFamily>::Kind;
        }
    }
}

fn gen_view_owner_name(view_name: &syn::Ident) -> syn::Ident {
    let view_name_str = view_name.to_string();
    let owned_name = view_name_str.strip_suffix("View").unwrap();
    syn::Ident::new(owned_name, view_name.span())
}

fn generic_param_idents<'a>(
    generics: impl IntoIterator<Item = &'a syn::GenericParam>,
) -> impl Iterator<Item = proc_macro2::TokenStream> {
    generics.into_iter().map(|param| match param {
        syn::GenericParam::Lifetime(param) => {
            let lifetime = &param.lifetime;
            quote! { #lifetime }
        }
        syn::GenericParam::Type(param) => {
            let ident = &param.ident;
            quote! { #ident }
        }
        syn::GenericParam::Const(param) => {
            let ident = &param.ident;
            quote! { #ident }
        }
    })
}

fn gen_struct_family_impls(
    family: &proc_macro2::TokenStream,
    repr: Option<&ReprKind>,
    name: &syn::Ident,
    generics: &syn::Generics,
    fields: &syn::Fields,
    has_trap_values: bool,
    has_custom_niche: bool,
) -> proc_macro2::TokenStream {
    let fields = fields.iter().map(|f| &f.ty).collect::<Vec<_>>();
    let repr_family_impl = if repr.is_some() {
        gen_repr_family_impl(family, name, generics, &fields, has_trap_values)
    } else {
        gen_rust_repr_family_impl(family, name, generics, &fields)
    };
    let size_family_impl = gen_size_family_impl(family, name, generics, &fields);
    let niche_family_impl = if has_custom_niche {
        gen_niche_family_impl_with_kind(
            family,
            name,
            generics,
            quote! { #family::niche::WithNiche<#family::niche::Custom> },
        )
    } else {
        gen_niche_family_impl(family, name, generics, &fields)
    };
    let mutability_family_impl = gen_mutability_family_impl(family, name, generics, &fields);

    quote! {
        #repr_family_impl
        #size_family_impl
        #niche_family_impl
        #mutability_family_impl
    }
}

fn gen_enum_family_impls(
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

    let repr_family_impl = if repr.is_some() {
        let has_trap_values = enum_tag_type(repr, variants.len())
            .is_some_and(|tag| !is_exhaustive_enum(variants.len(), &tag));
        gen_repr_family_impl(family, name, generics, &fields, has_trap_values)
    } else {
        gen_rust_repr_family_impl(family, name, generics, &fields)
    };
    // FIXME: Sometimes enums with variants are ZSTs and don't have a tag
    // This happens if all variants are uninhabited but one is ZST/fieldless.
    let size_family_impl = gen_non_zst_sized_family_impl(family, name, generics);
    let niche_family_impl = gen_enum_niche_family_impl(family, repr, name, generics, variants);
    let mutability_family_impl = gen_mutability_family_impl(family, name, generics, &fields);

    quote! {
        #repr_family_impl
        #size_family_impl
        #niche_family_impl
        #mutability_family_impl
    }
}

fn gen_fieldless_enum_family_impls(
    family: &proc_macro2::TokenStream,
    repr: Option<&ReprKind>,
    name: &syn::Ident,
    generics: &syn::Generics,
    variants: &Punctuated<syn::Variant, Token![,]>,
) -> proc_macro2::TokenStream {
    let repr_family = match repr {
        None => quote! { #family::repr::Unstable<#family::repr::Robust> },
        Some(ReprKind::C(None)) => unreachable!(),
        Some(ReprKind::Transparent) => quote! { #family::repr::Robust },
        Some(ReprKind::C(Some(tag)) | ReprKind::Primitive(tag)) => {
            let robustness = if is_exhaustive_enum(variants.len(), tag) {
                quote! { #family::repr::Robust }
            } else {
                quote! { #family::repr::NonRobust }
            };

            quote! { #family::repr::Stable<#robustness> }
        }
    };
    let tag_type = if repr.is_none() && variants.len() == 1 {
        None
    } else {
        enum_tag_type(repr, variants.len())
    };
    let size_family_impl = if tag_type.is_none() {
        gen_size_family_impl(family, name, generics, &[])
    } else {
        gen_non_zst_sized_family_impl(family, name, generics)
    };
    let niche_family_impl = if tag_type.is_none() {
        gen_niche_family_impl_with_kind(
            family,
            name,
            generics,
            quote! { #family::niche::WithoutNiche },
        )
    } else {
        gen_enum_niche_family_impl(family, repr, name, generics, variants)
    };
    let mutability_family_impl = gen_mutability_family_impl(family, name, generics, &[]);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #family::repr::ReprFamily for #name #ty_generics #where_clause {
            type Kind = #repr_family;
        }

        #size_family_impl
        #niche_family_impl
        #mutability_family_impl
    }
}

fn gen_rust_repr_family_impl(
    family: &proc_macro2::TokenStream,
    name: &syn::Ident,
    generics: &syn::Generics,
    fields: &[&syn::Type],
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let predicates = where_clause.as_ref().map(|w| &w.predicates);
    let aggregate = gen_aggregate_family(
        quote! { #family::repr::ReprFamily },
        quote! { #family::repr::Unstable<#family::repr::Robust> },
        generics,
        fields,
    );
    let repr_kind = aggregate.kind;
    let field_bounds = aggregate.field_bounds;
    let aggregate_bounds = aggregate.aggregate_bounds;

    quote! {
        impl #impl_generics #family::repr::ReprFamily for #name #ty_generics where
            #(#field_bounds,)*
            #(#aggregate_bounds,)*
            #predicates
        {
            type Kind = #repr_kind;
        }
    }
}

fn gen_repr_family_impl(
    family: &proc_macro2::TokenStream,
    name: &syn::Ident,
    generics: &syn::Generics,
    fields: &[&syn::Type],
    has_trap_values: bool,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let predicates = where_clause.as_ref().map(|w| &w.predicates);
    let init = if has_trap_values {
        quote! { #family::repr::NonRobust }
    } else {
        quote! { #family::repr::Robust }
    };
    let aggregate = gen_aggregate_family(
        quote! { #family::repr::ReprFamily },
        quote! { #family::repr::Stable<#init> },
        generics,
        fields,
    );
    let repr_kind = aggregate.kind;
    let field_bounds = aggregate.field_bounds;
    let aggregate_bounds = aggregate.aggregate_bounds;

    quote! {
        impl #impl_generics #family::repr::ReprFamily for #name #ty_generics where
            #(#field_bounds,)*
            #(#aggregate_bounds,)*
            #predicates
        {
            type Kind = #repr_kind;
        }
    }
}

fn gen_size_family_impl(
    family: &proc_macro2::TokenStream,
    name: &syn::Ident,
    generics: &syn::Generics,
    fields: &[&syn::Type],
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let predicates = where_clause.as_ref().map(|w| &w.predicates);
    let aggregate = gen_aggregate_family(
        quote! { #family::size::SizeFamily },
        quote! { #family::size::Sized<#family::size::Zst> },
        generics,
        fields,
    );
    let size_kind = aggregate.kind;
    let field_bounds = aggregate.field_bounds;
    let aggregate_bounds = aggregate.aggregate_bounds;

    quote! {
        unsafe impl #impl_generics #family::size::SizeFamily for #name #ty_generics where
            #(#field_bounds,)*
            #(#aggregate_bounds,)*
            #predicates
        {
            type Kind = #size_kind;
        }
    }
}

fn gen_non_zst_sized_family_impl(
    family: &proc_macro2::TokenStream,
    name: &syn::Ident,
    generics: &syn::Generics,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        unsafe impl #impl_generics #family::size::SizeFamily for #name #ty_generics #where_clause {
            type Kind = #family::size::Sized<#family::size::NonZst>;
        }
    }
}

fn gen_niche_family_impl(
    family: &proc_macro2::TokenStream,
    name: &syn::Ident,
    generics: &syn::Generics,
    fields: &[&syn::Type],
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let predicates = where_clause.as_ref().map(|w| &w.predicates);
    let aggregate = gen_aggregate_family(
        quote! { #family::niche::NicheFamily },
        // TODO: We're just using WithoutNiche for the ease of implementation. Remove it?
        quote! { #family::niche::WithoutNiche },
        generics,
        fields,
    );
    let niche_kind = aggregate.kind;
    let field_bounds = aggregate.field_bounds;
    let aggregate_bounds = aggregate.aggregate_bounds;

    quote! {
        impl #impl_generics #family::niche::NicheFamily for #name #ty_generics where
            #(#field_bounds,)*
            #(#aggregate_bounds,)*
            #predicates
        {
            type Kind = #niche_kind;
        }
    }
}

fn gen_niche_family_impl_with_kind(
    family: &proc_macro2::TokenStream,
    name: &syn::Ident,
    generics: &syn::Generics,
    kind: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #family::niche::NicheFamily for #name #ty_generics #where_clause {
            type Kind = #kind;
        }
    }
}

fn gen_mutability_family_impl(
    family: &proc_macro2::TokenStream,
    name: &syn::Ident,
    generics: &syn::Generics,
    fields: &[&syn::Type],
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let predicates = where_clause.as_ref().map(|w| &w.predicates);
    let init_kind = if fields.is_empty() {
        quote! { #family::mutability::Exclusive }
    } else {
        quote! { #family::mutability::Interior }
    };
    let aggregate = gen_aggregate_family(
        quote! { #family::mutability::MutabilityFamily },
        init_kind,
        generics,
        fields,
    );
    let mutability_kind = aggregate.kind;
    let field_bounds = aggregate.field_bounds;
    let aggregate_bounds = aggregate.aggregate_bounds;

    quote! {
        impl #impl_generics #family::mutability::MutabilityFamily for #name #ty_generics where
            #(#field_bounds,)*
            #(#aggregate_bounds,)*
            #predicates
        {
            type Kind = #mutability_kind;
        }
    }
}

struct AggregateFamily {
    kind: proc_macro2::TokenStream,
    field_bounds: Vec<proc_macro2::TokenStream>,
    aggregate_bounds: Vec<proc_macro2::TokenStream>,
}

fn gen_aggregate_family(
    family_trait: proc_macro2::TokenStream,
    init_kind: proc_macro2::TokenStream,
    generics: &syn::Generics,
    fields: &[&syn::Type],
) -> AggregateFamily {
    let (parametrized_fields, non_parametrized_fields): (Vec<&syn::Type>, Vec<_>) = fields
        .iter()
        .partition(|ty| is_type_parameterized(ty, generics));
    let mut kind = init_kind;
    let field_bounds = parametrized_fields
        .iter()
        .map(|ty| quote! { #ty: #family_trait })
        .collect::<Vec<_>>();
    let mut aggregate_bounds = Vec::new();

    for &field in &non_parametrized_fields {
        let field_kind = quote! { <#field as #family_trait>::Kind };
        kind = quote! { <#kind as core::ops::Add<#field_kind>>::Output };
    }

    for &field in &parametrized_fields {
        let field_kind = quote! { <#field as #family_trait>::Kind };
        aggregate_bounds.push(quote! { #field_kind: core::ops::Add<#kind> });
        kind = quote! { <#field_kind as core::ops::Add<#kind>>::Output };
    }

    AggregateFamily {
        kind,
        field_bounds,
        aggregate_bounds,
    }
}

fn gen_enum_niche_family_impl(
    family: &proc_macro2::TokenStream,
    repr: Option<&ReprKind>,
    name: &syn::Ident,
    generics: &syn::Generics,
    variants: &Punctuated<syn::Variant, Token![,]>,
) -> proc_macro2::TokenStream {
    let is_exhaustive = enum_tag_type(repr, variants.len())
        .is_none_or(|tag| is_exhaustive_enum(variants.len(), &tag));
    let niche_kind = if is_exhaustive {
        quote! { #family::niche::WithoutNiche }
    } else {
        quote! { #family::niche::WithNiche<#family::niche::Custom> }
    };

    gen_niche_family_impl_with_kind(family, name, generics, niche_kind)
}

fn enum_tag_type(repr: Option<&ReprKind>, variants_len: usize) -> Option<syn::Type> {
    fn infer_repr(num_variants: usize) -> syn::Type {
        const U8_CAPACITY: usize = u8::MAX as usize + 1;
        const U16_CAPACITY: usize = u16::MAX as usize + 1;
        const U32_CAPACITY: usize = u32::MAX as usize + 1;

        #[expect(clippy::match_overlapping_arm)]
        match num_variants {
            0..=U8_CAPACITY => syn::parse_quote!(u8),
            0..=U16_CAPACITY => syn::parse_quote!(u16),
            0..=U32_CAPACITY => syn::parse_quote!(u32),
            _ => syn::parse_quote!(u64),
        }
    }

    match repr {
        None => Some(infer_repr(variants_len)),
        Some(ReprKind::Transparent) => None,
        Some(ReprKind::C(None)) => None,
        Some(ReprKind::C(Some(repr))) | Some(ReprKind::Primitive(repr)) => Some(*repr.clone()),
    }
}

fn is_transparent_enum_repr(
    repr: Option<&ReprKind>,
    variants: &Punctuated<syn::Variant, Token![,]>,
) -> bool {
    matches!(repr, Some(ReprKind::Transparent)) || repr.is_none() && variants.len() == 1
}

fn is_exhaustive_enum(num_variants: usize, repr: &syn::Type) -> bool {
    fn repr_type_bit_width(repr: &syn::Type) -> Option<u32> {
        let syn::Type::Path(type_path) = repr else {
            return None;
        };
        let ident = type_path.path.get_ident()?.to_string();

        match ident.as_str() {
            "u8" | "i8" => Some(8),
            "u16" | "i16" => Some(16),
            "u32" | "i32" => Some(32),
            "u64" | "i64" => Some(64),
            _ => None,
        }
    }

    let max_values = match repr_type_bit_width(repr) {
        Some(8) => 1u64 << 8,
        Some(16) => 1u64 << 16,
        Some(32) => 1u64 << 32,
        Some(64) | None | Some(_) => return false,
    };

    num_variants as u64 == max_values
}

fn is_type_parameterized(ty: &syn::Type, generics: &syn::Generics) -> bool {
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
