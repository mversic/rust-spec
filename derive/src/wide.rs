use quote::quote;

pub(crate) fn expand(
    family: &proc_macro2::TokenStream,
    input: &syn::DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let syn::Data::Struct(data) = &input.data else {
        return Ok(quote! {});
    };
    let fields = &data.fields;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let predicates = where_clause.as_ref().map(|w| &w.predicates);

    if fields.len() != 1 {
        return Ok(quote! {});
    }
    let Some(field) = fields.iter().last() else {
        return Ok(quote! {});
    };

    let name = &input.ident;
    let field_ty = &field.ty;
    let field_ref = field.ident.as_ref().map_or_else(
        || quote! { self.0 },
        |field_name| quote! { self.#field_name },
    );

    let for_dummy = if input.generics.params.is_empty() {
        quote! { for<'__dummy> }
    } else {
        quote! {}
    };

    let alloc_methods = if cfg!(feature = "alloc") {
        quote! {
            #[inline(always)]
            fn into_non_null(self: Box<Self>) -> core::ptr::NonNull<Self::Data> {
                let field = Box::into_raw(self) as *mut #field_ty;
                unsafe { <#field_ty as #family::size::Wide>::into_non_null(Box::from_raw(field)) }
            }

            #[inline(always)]
            unsafe fn from_non_null(
                data: core::ptr::NonNull<Self::Data>,
                metadata: Self::Metadata,
            ) -> Box<Self> {
                let field = unsafe { <#field_ty as #family::size::Wide>::from_non_null(data, metadata) };
                unsafe { Box::from_raw(Box::into_raw(field) as *mut Self) }
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        impl #impl_generics #family::size::Wide for #name #ty_generics
        where
            #for_dummy #field_ty: #family::size::Wide,
            #predicates
        {
            type Data = <#field_ty as #family::size::Wide>::Data;
            type Metadata = <#field_ty as #family::size::Wide>::Metadata;

            #[inline(always)]
            fn metadata(&self) -> Self::Metadata {
                #family::size::Wide::metadata(&#field_ref)
            }

            #[inline(always)]
            fn as_ptr(&self) -> *const Self::Data {
                #family::size::Wide::as_ptr(&#field_ref)
            }

            #[inline(always)]
            fn as_mut_ptr(&mut self) -> *mut Self::Data {
                #family::size::Wide::as_mut_ptr(&mut #field_ref)
            }

            #[inline(always)]
            unsafe fn from_raw_parts<'__rust_spec>(
                data: *const Self::Data,
                metadata: Self::Metadata,
            ) -> &'__rust_spec Self {
                let field = unsafe { <#field_ty as #family::size::Wide>::from_raw_parts(data, metadata) };
                unsafe { &*(field as *const #field_ty as *const Self) }
            }

            #[inline(always)]
            unsafe fn from_raw_parts_mut<'__rust_spec>(
                data: *mut Self::Data,
                metadata: Self::Metadata,
            ) -> &'__rust_spec mut Self {
                let field = unsafe { <#field_ty as #family::size::Wide>::from_raw_parts_mut(data, metadata) };
                unsafe { &mut *(field as *mut #field_ty as *mut Self) }
            }

            #alloc_methods
        }
    })
}
