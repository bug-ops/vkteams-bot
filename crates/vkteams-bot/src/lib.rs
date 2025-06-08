#![forbid(unsafe_code)]
//! # VK Teams Bot API client
//! This crate provides a client for the [VK Teams Bot API] V1.
//! Asynchronous request is based on [`reqwest`] and [`tokio`].
//! JSON Serialization and Deserialization [`serde_json`].
//! Serialization Url query is based on [`serde_url_params`].
//!
//! ```toml
//! [dependencies]
//! vkteams_bot = "0.9"
//! log = "0.4"
//! ```
//!
//! [VK Teams Bot API]: https://teams.vk.com/botapi/?lang=en
//! [`reqwest`]: https://docs.rs/reqwest
//! [`tokio`]: https://docs.rs/tokio
//! [`serde_json`]: https://docs.rs/serde_json
//! [`serde_url_params`]: https://docs.rs/serde_url_params
//! [`axum`]: https://docs.rs/axum

#[macro_export]
macro_rules! bot_api_method {
    (
        $(#[$req_attr:meta])*
        method = $method:literal,
        $(http_method = $http_method:expr,)?
        request = $Req:ident {
            required {
                $( $req_f:ident : $ReqT:ty ),* $(,)?
            },
            optional {
                $( $(#[$opt_attr:meta])* $opt_f:ident : $OptT:ty ),* $(,)?
            }
        },
        response = $Res:ident {
            $( $(#[$res_attr:meta])* $res_f:ident : $ResT:ty ),* $(,)?
        },
    ) => {
        use serde::{Deserialize, Serialize};
        use vkteams_bot_macros::GetField;
        #[derive(Serialize, Deserialize, Clone, Debug, Default, GetField)]
        #[serde(rename_all = "camelCase")]
        #[non_exhaustive]
        $(#[$req_attr])*
        pub struct $Req {
            $( pub $req_f : $ReqT, )*
            $( $(#[$opt_attr])*
                #[serde(skip_serializing_if = "Option::is_none")]
                pub $opt_f : Option<$OptT>, )*
        }

        #[derive(Serialize, Deserialize, Clone, Debug, Default)]
        #[serde(rename_all = "camelCase")]
        pub struct $Res {
            $( $(#[$res_attr])* pub $res_f : $ResT, )*
        }

        impl $crate::api::types::BotRequest for $Req {
            type Args = ($($ReqT),*);
            const METHOD: &'static str = $method;
            $(const HTTP_METHOD: $crate::api::types::HTTPMethod = $http_method;)?
            type RequestType = Self;
            type ResponseType = $Res;

            fn new(($($req_f),*): ($($ReqT),*)) -> Self {
                Self {
                    $( $req_f, )*
                    $( $opt_f: None, )*
                }
            }

            fn get_chat_id(&self) -> Option<&$crate::api::types::ChatId> {
                self._get_chat_id()
            }

            fn get_multipart(&self) -> &$crate::api::types::MultipartName {
                self._get_multipart()
            }
        }

        impl $Req {
            paste::paste! {
                $(
                    #[doc = concat!("Sets the field `", stringify!($opt_f), "`")]
                    pub fn [<with_ $opt_f>](mut self, value: $OptT) -> Self {
                        self.$opt_f = Some(value);
                        self
                    }
                )*
            }
        }
    };
}

pub mod bot;
pub mod config;
pub mod error;
#[cfg(feature = "otlp")]
pub mod otlp;
pub mod prelude;
/// API methods
mod api {
    /// API `/chats/` methods
    pub mod chats;
    pub mod types;
    pub mod utils;
    /// API `/events/` methods
    pub mod events {
        pub mod get;
    }
    /// API `/files/` methods
    pub mod files {
        pub mod get_info;
    }
    /// API `/messages/` methods
    pub mod messages;
    /// API `/myself/` methods
    pub mod myself {
        pub mod get;
    }
}

pub use self::bot::Bot;
