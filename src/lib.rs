//! # VK Teams Bot API client
//! This crate provides a client for the [VK Teams Bot API] V1.
//! Asyncronous request is based on [`reqwest`] and [`tokio`].
//! JSON Serialization and Deserialization [`serde_json`].
//! Serialization Url query is based on [`serde_url_params`].
//!
//! ```toml
//! [dependencies]
//! vkteams = "0.1.1"
//! ```
//!  _Compiler support: requires rustc 1.67+_.
//!
//! [VK Teams Bot API]: https://teams.vk.com/botapi/?lang=en
//! [`reqwest`]: https://docs.rs/reqwest
//! [`tokio`]: https://docs.rs/tokio    
//! [`serde_json`]: https://docs.rs/serde_json
//! [`serde_url_params`]: https://docs.rs/serde_url_params
//! # Environment
//! - `RUST_LOG` - log level (default: `info`)
//! - `VKTEAMS_VKTEAMS_BOT_API_TOKEN` - bot token
//! - `VKTEAMS_BOT_API_URL` - bot api url
//! - `VKTEAMS_PROXY` - proxy url
//!
//! ```bash
//! # Unix-like
//! $ export VKTEAMS_VKTEAMS_BOT_API_TOKEN=<Your token here> #require
//! $ export VKTEAMS_BOT_API_URL=<Your base api url> #require
//! $ export VKTEAMS_PROXY=<Proxy> #optional
//!
//! # Windows
//! $ set VKTEAMS_VKTEAMS_BOT_API_TOKEN=<Your token here> #require
//! $ set VKTEAMS_BOT_API_URL=<Your base api url> #require
//! $ set VKTEAMS_PROXY=<Proxy> #optional
//! ```
#[macro_use]
extern crate log;

pub use self::{bot::types::*, bot::*};
pub mod bot;
