use quote::quote;
use syn::visit::Visit;

pub(crate) struct AggregateFamily {
    pub(crate) kind: proc_macro2::TokenStream,
    pub(crate) field_bounds: Vec<proc_macro2::TokenStream>,
    pub(crate) aggregate_bounds: Vec<proc_macro2::TokenStream>,
}

impl AggregateFamily {
    pub(crate) fn fixed(kind: proc_macro2::TokenStream) -> Self {
        Self {
            kind,
            field_bounds: Vec::new(),
            aggregate_bounds: Vec::new(),
        }
    }

    pub(crate) fn fold(
        family: &proc_macro2::TokenStream,
        axis: proc_macro2::TokenStream,
        init_kind: proc_macro2::TokenStream,
        generics: &syn::Generics,
        fields: &[&syn::Type],
    ) -> Self {
        let (parametrized_fields, non_parametrized_fields): (Vec<&syn::Type>, Vec<_>) = fields
            .iter()
            .partition(|ty| is_type_parameterized(ty, generics));
        let mut kind = init_kind;
        let field_bounds = parametrized_fields
            .iter()
            .map(|ty| quote! { #ty: #family::RustSpec })
            .collect::<Vec<_>>();
        let mut aggregate_bounds = Vec::new();

        for &field in &non_parametrized_fields {
            let field_kind = quote! { <#field as #family::RustSpec>::#axis };
            kind = quote! { <#kind as core::ops::Add<#field_kind>>::Output };
        }

        for &field in &parametrized_fields {
            let field_kind = quote! { <#field as #family::RustSpec>::#axis };
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
