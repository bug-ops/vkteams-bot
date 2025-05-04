#![forbid(unsafe_code)]
//! # VK Teams Bot API client
//! This crate provides a client for the [VK Teams Bot API] V1.
//! Asynchronous request is based on [`reqwest`] and [`tokio`].
//! JSON Serialization and Deserialization [`serde_json`].
//! Serialization Url query is based on [`serde_url_params`].
//!
//! ```toml
//! [dependencies]
//! vkteams_bot = "0.7"
//! log = "0.4"
//! ```
//!
//! [VK Teams Bot API]: https://teams.vk.com/botapi/?lang=en
//! [`reqwest`]: https://docs.rs/reqwest
//! [`tokio`]: https://docs.rs/tokio
//! [`serde_json`]: https://docs.rs/serde_json
//! [`serde_url_params`]: https://docs.rs/serde_url_params
//! [`axum`]: https://docs.rs/axum
//! # Environment
//! - `RUST_LOG` - log level (default: `info`)
//! - `VKTEAMS_BOT_API_TOKEN` - bot token
//! - `VKTEAMS_BOT_API_URL` - bot api url
//! - `VKTEAMS_PROXY` - proxy url
//!
//! ```bash
//! # Unix-like
//! $ export VKTEAMS_BOT_API_TOKEN=<Your token here> #require
//! $ export VKTEAMS_BOT_API_URL=<Your base api url> #require
//! $ export VKTEAMS_PROXY=<Proxy> #optional
//!
//! # Windows
//! $ set VKTEAMS_BOT_API_TOKEN=<Your token here> #require
//! $ set VKTEAMS_BOT_API_URL=<Your base api url> #require
//! $ set VKTEAMS_PROXY=<Proxy> #optional
//! ```
#[macro_use]
extern crate log;

macro_rules! bot_api_method {
    (
        method = $method:literal,
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
        #[derive(Serialize, Clone, Debug, Default)]
        #[serde(rename_all = "camelCase")]
        #[non_exhaustive]
        pub struct $Req {
            $( pub $req_f : $ReqT, )*
            $( $(#[$opt_attr])* pub $opt_f : Option<$OptT>, )*
        }

        #[derive(Serialize, Deserialize, Clone, Debug, Default)]
        #[serde(rename_all = "camelCase")]
        pub struct $Res {
            $( $(#[$res_attr])* pub $res_f : $ResT, )*
        }

        impl crate::api::types::BotRequest for $Req {
            type Args = ($($ReqT),*);
            const METHOD: &'static str = $method;
            type RequestType = Self;
            type ResponseType = $Res;

            fn new(($($req_f),*): ($($ReqT),*)) -> Self {
                Self {
                    $( $req_f, )*
                    ..Default::default()
                }
            }
        }

        impl $Req {
            paste::paste! {
                $(
                    #[doc = concat!("Устанавливает поле `", stringify!($opt_f), "`")]
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
pub mod prelude;
/// API methods
mod api {
    /// API `/chats/` methods
    pub mod chats;
    pub mod default;
    pub mod display;
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
