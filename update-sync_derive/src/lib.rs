use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_macro_input, punctuated::Punctuated, Data, DataEnum, DataStruct, DeriveInput, Field,
    Fields, FieldsNamed, FieldsUnnamed, Ident, Index, Token, Variant,
};

/// Automatically derives `UpdateSync` to update the fields of structs, so long as they are all themselves `UpdateSync`
/// It will do the same for enums, but syncing to different variants where appropriate
#[proc_macro_derive(UpdateSync)]
pub fn derive_update_sync(input: TokenStream1) -> TokenStream1 {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    match data {
        Data::Struct(DataStruct { fields, .. }) => {
            let update_fields = struct_update_fields(&fields);
            quote! {
                impl ::update_sync::UpdateSync for #ident {
                    fn update_sync(last_base: Self, new_base: Self, set: Self) -> Self {
                        #ident #update_fields
                    }
                }
            }
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let mod_ident = format_ident!("__{}__Mod__Internals", ident);
            let mod_pseudo_structs: TokenStream2 = variants
                .iter()
                .map(
                    |Variant {
                         ident: v_ident,
                         fields,
                         ..
                     }| {
                        let terminator =
                            if std::matches!(fields, Fields::Unit | Fields::Unnamed(_)) {
                                <Token![;]>::default().to_token_stream()
                            } else {
                                quote! {}
                            };
                        let field_assign = field_match_assign(&ident, v_ident, fields);
                        let unassign = field_match_unassign(v_ident, fields);
                        quote! {
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
            let matches: TokenStream2 = variants
                .iter()
                .map(|Variant { ident: v_ident, .. }| {
                    quote! {
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
            quote! {
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
        Data::Union(_) => quote! {},
    }
    .into()
}

fn struct_update_fields(fields: &Fields) -> TokenStream2 {
    match fields {
        Fields::Named(FieldsNamed { named: fields, .. })
        | Fields::Unnamed(FieldsUnnamed {
            unnamed: fields, ..
        }) => {
            let fields = struct_update_named_or_unnamed(fields);
            (quote! { { #fields } }).into()
        }
        Fields::Unit => quote! {},
    }
}

fn struct_update_named_or_unnamed<T>(fields: &Punctuated<Field, T>) -> TokenStream2 {
    fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let field = f.ident.clone().map(|n| n.to_token_stream()).unwrap_or(
                Index {
                    index: i as u32,
                    span: Span2::call_site(),
                }
                .to_token_stream(),
            );
            quote! {
                #field: ::update_sync::UpdateSync::update_sync(
                    last_base.#field,
                    new_base.#field,
                    set.#field,
                ),
            }
        })
        .collect()
}

fn field_match_assign(e_ident: &Ident, v_ident: &Ident, fields: &Fields) -> TokenStream2 {
    match fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            let fields: TokenStream2 = named
                .iter()
                .map(|Field { ident, .. }| {
                    let ident = ident.as_ref().unwrap();
                    quote! {
                        #ident,
                    }
                })
                .collect();
            quote! {
                #e_ident :: #v_ident { #fields } => #v_ident { #fields },
            }
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let fields: TokenStream2 = unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    let ident = format_ident!(
                        "__{}",
                        Index {
                            index: i as u32,
                            span: Span2::call_site(),
                        }
                    );
                    quote! {
                        #ident,
                    }
                })
                .collect();
            quote! {
                #e_ident :: #v_ident ( #fields ) => #v_ident ( #fields ),
            }
        }
        Fields::Unit => quote! {
            #e_ident :: #v_ident => #v_ident,
        },
    }
}
fn field_match_unassign(v_ident: &Ident, fields: &Fields) -> TokenStream2 {
    match fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            let fields: TokenStream2 = named
                .iter()
                .map(|Field { ident, .. }| {
                    let ident = ident.as_ref().unwrap();
                    quote! {
                        #ident,
                    }
                })
                .collect();
            quote! {
                #v_ident { #fields }
            }
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let fields: TokenStream2 = unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    let ident = format_ident!(
                        "__{}",
                        Index {
                            index: i as u32,
                            span: Span2::call_site(),
                        }
                    );
                    quote! {
                        #ident,
                    }
                })
                .collect();
            quote! {
                #v_ident ( #fields )
            }
        }
        Fields::Unit => quote! {
            #v_ident
        },
    }
}
