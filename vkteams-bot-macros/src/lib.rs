use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

#[proc_macro_derive(HasChatId)]
pub fn derive_has_chat_id(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => return quote! {}.into(),
        },
        _ => return quote! {}.into(),
    };

    let chat_id_field = fields.iter().find(|field| {
        if let Type::Path(type_path) = &field.ty {
            if let Some(ident) = type_path.path.get_ident() {
                ident == "ChatId"
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
                pub fn _get_chat_id(&self) -> Option<&crate::prelude::types::ChatId> {
                    Some(&self.#field_name)
                }
            }
        }
    } else {
        quote! {
            impl #name {
                pub fn _get_chat_id(&self) -> Option<&crate::prelude::types::ChatId> {
                    None
                }
            }
        }
    };

    impl_block.into()
}
