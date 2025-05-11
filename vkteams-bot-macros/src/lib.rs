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
            Type::Group(group) => is_chat_id_type(&*group.elem),
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
