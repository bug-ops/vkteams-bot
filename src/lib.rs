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
/// Bot client
pub mod bot;
/// API methods
pub mod api {
    pub mod default;
    pub mod display;
    pub mod net;
    pub mod types;
    pub mod utils;
    /// API `/chats/` methods
    pub mod chats {
        pub mod avatar_set;
        pub mod block_user;
        pub mod get_admins;
        pub mod get_blocked_users;
        pub mod get_info;
        pub mod get_members;
        pub mod get_pending_users;
        pub mod members_delete;
        pub mod pin_message;
        pub mod resolve_pendings;
        pub mod send_action;
        pub mod set_about;
        pub mod set_rules;
        pub mod set_title;
        pub mod unblock_user;
        pub mod unpin_message;
    }
    /// API `/events/` methods
    pub mod events {
        pub mod get;
    }
    /// API `/files/` methods
    pub mod files {
        pub mod get_info;
    }
    /// API `/messages/` methods
    pub mod messages {
        pub mod answer_callback_query;
        pub mod delete_messages;
        pub mod edit_text;
        pub mod send_file;
        pub mod send_text;
        // pub mod send_text_with_deep_link;
        pub mod send_voice;
    }
    /// API `/myself/` methods
    pub mod myself {
        pub mod get;
    }
}
