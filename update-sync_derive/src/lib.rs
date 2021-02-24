use quote::ToTokens;

/// Automatically derives `UpdateSync` to update the fields of structs, so long as they are all themselves `UpdateSync`
/// It will do the same for enums, but syncing to different variants where appropriate
#[proc_macro_derive(UpdateSync)]
pub fn derive_update_sync(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let syn::DeriveInput { ident, data, .. } = syn::parse_macro_input!(input as syn::DeriveInput);

    match data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => {
            let update_fields = struct_update_fields(&fields);
            quote::quote! {
                impl ::update_sync::UpdateSync for #ident {
                    fn update_sync(last_base: Self, new_base: Self, set: Self) -> Self {
                        #ident #update_fields
                    }
                }
            }
        }
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            let mod_ident = quote::format_ident!("__{}__Mod__Internals", ident);
            let mod_pseudo_structs: proc_macro2::TokenStream = variants
                .iter()
                .map(
                    |syn::Variant {
                         ident: v_ident,
                         fields,
                         ..
                     }| {
                        let terminator =
                            if std::matches!(fields, syn::Fields::Unit | syn::Fields::Unnamed(_)) {
                                <syn::Token![;]>::default().to_token_stream()
                            } else {
                                quote::quote! {}
                            };
                        let field_assign = field_match_assign(&ident, v_ident, fields);
                        let unassign = field_match_unassign(v_ident, fields);
                        quote::quote! {
                            #[derive(::update_sync::derive::UpdateSync, PartialEq)]
                            struct #v_ident #fields #terminator
                            impl #v_ident {
                                fn from_enum(from: #ident) -> Option<#v_ident> {
                                    Some(match from {
                                        #field_assign
                                        _ => None?
                                    })
                                }
                                fn to_enum(self) -> #ident {
                                    let #unassign = self;
                                    #ident :: #unassign
                                }
                            }
                        }
                    },
                )
                .collect();
            let matches: proc_macro2::TokenStream = variants
                .iter()
                .map(|syn::Variant { ident: v_ident, .. }| {
                    quote::quote! {
                        #ident :: #v_ident { .. } => {
                            let last_base = #mod_ident :: #v_ident :: from_enum ( last_base ).unwrap();
                            let new_base = #mod_ident :: #v_ident :: from_enum ( new_base ).unwrap();
                            let set = #mod_ident :: #v_ident :: from_enum ( set ).unwrap();
                            let new = ::update_sync::UpdateSync::update_sync(last_base, new_base, set);
                            new.to_enum()
                        },
                    }
                })
                .collect();
            quote::quote! {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub mod #mod_ident {
                    use super::*;
                    #mod_pseudo_structs
                    impl ::update_sync::UpdateSync for #ident {
                        fn update_sync(last_base: Self, new_base: Self, set: Self) -> Self {
                            let last_base_discriminant = std::mem::discriminant(&last_base);
                            let new_base_discriminant = std::mem::discriminant(&new_base);
                            let set_discriminant = std::mem::discriminant(&set);
                            if last_base_discriminant != set_discriminant || last_base_discriminant != new_base_discriminant {
                                set
                            } else {
                                // By here, all params are the same variant, so we can write a match that panics if they aren't
                                match last_base {
                                    #matches
                                    _ => std::unreachable!()
                                }
                            }
                        }
                    }
                }
            }
        }
        syn::Data::Union(_) => quote::quote! {},
    }
    .into()
}

fn struct_update_fields(fields: &syn::Fields) -> proc_macro2::TokenStream {
    match fields {
        syn::Fields::Named(syn::FieldsNamed { named: fields, .. })
        | syn::Fields::Unnamed(syn::FieldsUnnamed {
            unnamed: fields, ..
        }) => {
            let fields = struct_update_named_or_unnamed(fields);
            (quote::quote! { { #fields } }).into()
        }
        syn::Fields::Unit => quote::quote! {},
    }
}

fn struct_update_named_or_unnamed<T>(
    fields: &syn::punctuated::Punctuated<syn::Field, T>,
) -> proc_macro2::TokenStream {
    fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let field = f.ident.clone().map(|n| n.to_token_stream()).unwrap_or(
                syn::Index {
                    index: i as u32,
                    span: proc_macro2::Span::call_site(),
                }
                .to_token_stream(),
            );
            quote::quote! {
                #field: UpdateSync::update_sync(
                    last_base.#field,
                    new_base.#field,
                    set.#field,
                ),
            }
        })
        .collect()
}

fn field_match_assign(
    e_ident: &syn::Ident,
    v_ident: &syn::Ident,
    fields: &syn::Fields,
) -> proc_macro2::TokenStream {
    match fields {
        syn::Fields::Named(syn::FieldsNamed { named, .. }) => {
            let fields: proc_macro2::TokenStream = named
                .iter()
                .map(|syn::Field { ident, .. }| {
                    let ident = ident.as_ref().unwrap();
                    quote::quote! {
                        #ident,
                    }
                })
                .collect();
            quote::quote! {
                #e_ident :: #v_ident { #fields } => #v_ident { #fields },
            }
        }
        syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }) => {
            let fields: proc_macro2::TokenStream = unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    let ident = quote::format_ident!(
                        "__{}",
                        syn::Index {
                            index: i as u32,
                            span: proc_macro2::Span::call_site(),
                        }
                    );
                    quote::quote! {
                        #ident,
                    }
                })
                .collect();
            quote::quote! {
                #e_ident :: #v_ident ( #fields ) => #v_ident ( #fields ),
            }
        }
        syn::Fields::Unit => quote::quote! {
            #e_ident :: #v_ident => #v_ident,
        },
    }
}
fn field_match_unassign(v_ident: &syn::Ident, fields: &syn::Fields) -> proc_macro2::TokenStream {
    match fields {
        syn::Fields::Named(syn::FieldsNamed { named, .. }) => {
            let fields: proc_macro2::TokenStream = named
                .iter()
                .map(|syn::Field { ident, .. }| {
                    let ident = ident.as_ref().unwrap();
                    quote::quote! {
                        #ident,
                    }
                })
                .collect();
            quote::quote! {
                #v_ident { #fields }
            }
        }
        syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }) => {
            let fields: proc_macro2::TokenStream = unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    let ident = quote::format_ident!(
                        "__{}",
                        syn::Index {
                            index: i as u32,
                            span: proc_macro2::Span::call_site(),
                        }
                    );
                    quote::quote! {
                        #ident,
                    }
                })
                .collect();
            quote::quote! {
                #v_ident ( #fields )
            }
        }
        syn::Fields::Unit => quote::quote! {
            #v_ident
        },
    }
}
