use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

#[proc_macro_derive(ChatId)]
pub fn derive_chat_id(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => return quote! {}.into(),
        },
        _ => return quote! {}.into(),
    };

    // Helper function for recursive type checking
    fn is_chat_id_type(ty: &Type) -> bool {
        match ty {
            Type::Path(type_path) => type_path
                .path
                .segments
                .iter()
                .any(|seg| seg.ident == "ChatId"),
            Type::Group(group) => is_chat_id_type(&group.elem),
            _ => false,
        }
    }

    let chat_id_field = fields.iter().find(|field| {
        if let Some(ident) = &field.ident {
            if ident == "chat_id" {
                is_chat_id_type(&field.ty)
            } else {
                false
            }
        } else {
            false
        }
    });

    let impl_block = if let Some(field) = chat_id_field {
        let field_name = &field.ident;
        quote! {
            impl #name {
                pub fn _get_chat_id(&self) -> Option<&crate::api::types::ChatId> {
                    Some(&self.#field_name)
                }
            }
        }
    } else {
        quote! {
            impl #name {
                pub fn _get_chat_id(&self) -> Option<&crate::api::types::ChatId> {
                    None
                }
            }
        }
    };

    impl_block.into()
}

#[proc_macro_derive(GetField)]
pub fn derive_get_field(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => return quote! {}.into(),
        },
        _ => return quote! {}.into(),
    };

    // Check for chat_id: ChatId
    let has_chat_id = fields.iter().any(|f| {
        f.ident.as_ref().map(|id| id == "chat_id").unwrap_or(false)
            && match &f.ty {
                Type::Path(type_path) => type_path
                    .path
                    .segments
                    .last()
                    .map(|s| s.ident == "ChatId")
                    .unwrap_or(false),
                _ => false,
            }
    });
    // Check for multipart: MultipartName
    let has_multipart = fields.iter().any(|f| {
        f.ident
            .as_ref()
            .map(|id| id == "multipart")
            .unwrap_or(false)
            && is_type_named(&f.ty, "MultipartName")
    });

    let chat_id_impl = if has_chat_id {
        quote! {
            impl #name {
                pub fn _get_chat_id(&self) -> Option<&crate::api::types::ChatId> {
                    Some(&self.chat_id)
                }
            }
        }
    } else {
        quote! {
            impl #name {
                pub fn _get_chat_id(&self) -> Option<&crate::api::types::ChatId> {
                    None
                }
            }
        }
    };
    let multipart_impl = if has_multipart {
        quote! {
            impl #name {
                pub fn _get_multipart(&self) -> &crate::api::types::MultipartName {
                    &self.multipart
                }
            }
        }
    } else {
        quote! {
            impl #name {
                pub fn _get_multipart(&self) -> &crate::api::types::MultipartName {
                    &crate::api::types::MultipartName::None
                }
            }
        }
    };
    let expanded = quote! {
        #chat_id_impl
        #multipart_impl
    };
    expanded.into()
}

fn is_type_named(ty: &Type, name: &str) -> bool {
    match ty {
        Type::Path(type_path) => type_path.path.segments.iter().any(|seg| seg.ident == name),
        Type::Group(group) => is_type_named(&group.elem, name),
        _ => false,
    }
}
