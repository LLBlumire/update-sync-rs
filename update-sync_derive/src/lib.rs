/// Automatically derives `UpdateSync` to update the fields of structs, so long as they are all themselves `UpdateSync`
/// It will do the same for enums, but syncing to different variants where appropriate
#[proc_macro_derive(UpdateSync)]
pub fn derive_update_sync(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let syn::DeriveInput { ident, data, .. } = syn::parse_macro_input!(input as syn::DeriveInput);

    match data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Unit,
            ..
        }) => quote::quote! {
            impl ::update_sync::UpdateSync for #ident {
                fn update_sync(last_base: Self, new_base: Self, set: Self) -> Self {
                    #ident
                }
            }
        },
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) => {
            let mut fields = quote::quote! {};
            fields.extend(named.iter().map(|f| {
                let field = f.ident.clone().unwrap(); // this struct has named fields
                quote::quote! {
                    #field: UpdateSync::update_sync(
                        last_base.#field,
                        new_base.#field,
                        set.#field,
                    ),
                }
            }));
            quote::quote! {
                impl ::update_sync::UpdateSync for #ident {
                    fn update_sync(last_base: Self, new_base: Self, set: Self) -> Self {
                        #ident {
                            #fields
                        }
                    }
                }
            }
        }
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }),
            ..
        }) => {
            let mut fields = quote::quote! {};
            fields.extend(unnamed.iter().enumerate().map(|(i, _)| {
                let field = syn::Index {
                    index: i as u32,
                    span: proc_macro2::Span::call_site(),
                };
                quote::quote! {
                    UpdateSync::update_sync(
                        last_base.#field,
                        new_base.#field,
                        set.#field
                    ),
                }
            }));
            quote::quote! {
                impl ::update_sync::UpdateSync for #ident {
                    fn update_sync(last_base: Self, new_base: Self, set: Self) -> Self {
                        #ident (
                            #fields
                        )
                    }
                }
            }
        }
        syn::Data::Enum(_) => quote::quote! {},
        syn::Data::Union(_) => quote::quote! {},
    }
    .into()
}
