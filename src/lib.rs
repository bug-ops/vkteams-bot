#![forbid(unsafe_code)]
//! # VK Teams Bot API client
//! This crate provides a client for the [VK Teams Bot API] V1.
//! Asynchronous request is based on [`reqwest`] and [`tokio`].
//! JSON Serialization and Deserialization [`serde_json`].
//! Serialization Url query is based on [`serde_url_params`].
//!
//! ```toml
//! [dependencies]
//! vkteams = "0.5"
//! ```
//!
//! [VK Teams Bot API]: https://teams.vk.com/botapi/?lang=en
//! [`reqwest`]: https://docs.rs/reqwest
//! [`tokio`]: https://docs.rs/tokio
//! [`serde_json`]: https://docs.rs/serde_json
//! [`serde_url_params`]: https://docs.rs/serde_url_params
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
mod bot;
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
