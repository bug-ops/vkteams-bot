#[cfg(feature = "grpc")]
pub mod grpc;
#[cfg(feature = "longpoll")]
pub mod longpoll;
pub mod net;
#[cfg(feature = "webhook")]
pub mod webhook;

use crate::api::types::*;
use crate::error::{BotError, Result};
use net::*;
use reqwest::{Client, Url};
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
/// Bot class with attributes
/// - `client`: [`reqwest::Client`]
/// - `token`: [`String`]
/// - `base_api_url`: [`reqwest::Url`]
/// - `base_api_path`: [`String`]
/// - `event_id`: [`std::sync::Arc<_>`]
///
/// [`reqwest::Client`]: https://docs.rs/reqwest/latest/reqwest/struct.Client.html
/// [`reqwest::Url`]: https://docs.rs/reqwest/latest/reqwest/struct.Url.html
/// [`std::sync::Arc<_>`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
pub struct Bot {
    pub(crate) client: Client,
    pub(crate) token: String,
    pub(crate) base_api_url: Url,
    pub(crate) base_api_path: String,
    pub(crate) event_id: Arc<Mutex<EventId>>,
}

impl Bot {
    /// Creates a new `Bot` with API version [`APIVersionUrl`]
    ///
    /// Build optional proxy from .env variable `VKTEAMS_PROXY` if its bound, passes to [`reqwest::Proxy::all`].
    ///
    /// Build [`reqwest::Client`] with default settings.
    ///
    /// Get token from variable `VKTEAMS_BOT_API_TOKEN` in .env file
    ///
    /// Get base url from variable `VKTEAMS_BOT_API_URL` in .env file
    ///
    /// Set default path depending on API version
    ///
    /// ## Errors
    /// - `BotError::Config` - configuration error (invalid token or URL)
    /// - `BotError::Url` - URL parsing error
    ///
    /// ## Panics
    /// - Unable to parse proxy if its bound in `VKTEAMS_PROXY` env variable
    /// - Unable to build [`reqwest::Client`]
    /// - Unable to find token in .env file
    /// - Unable to find or parse url in .env file
    ///
    /// [`reqwest::Client`]: https://docs.rs/reqwest/latest/reqwest/struct.Client.html
    /// [`reqwest::Proxy::all`]: https://docs.rs/reqwest/latest/reqwest/struct.Proxy.html#method.all
    pub fn new(version: APIVersionUrl) -> Self {
        debug!("Creating new bot with API version: {:?}", version);

        let token = std::env::var(VKTEAMS_BOT_API_TOKEN)
            .map_err(|e| BotError::Config(format!("Failed to get token: {}", e)))
            .unwrap_or_else(|e| panic!("{}", e));
        debug!("Token successfully obtained");

        let base_api_url = std::env::var(VKTEAMS_BOT_API_URL)
            .map_err(|e| BotError::Config(format!("Failed to get API URL: {}", e)))
            .unwrap_or_else(|e| panic!("{}", e));
        debug!("API URL successfully obtained");

        let base_api_url = Url::parse(&base_api_url)
            .map_err(|e| BotError::Config(format!("Invalid API URL format: {}", e)))
            .unwrap_or_else(|e| panic!("{}", e));
        debug!("API URL successfully parsed");

        let base_api_path = match version {
            APIVersionUrl::V1 => "/bot/v1/",
        };
        debug!("Set API base path: {}", base_api_path);

        Self {
            client: Client::new(),
            token,
            base_api_url,
            base_api_path: base_api_path.to_string(),
            event_id: Arc::new(Mutex::new(0)),
        }
    }

    /// Get last event id
    ///
    /// ## Errors
    /// - `BotError::System` - mutex lock error
    pub fn get_last_event_id(&self) -> EventId {
        *self
            .event_id
            .lock()
            .map_err(|e| BotError::System(format!("Mutex lock error: {}", e)))
            .unwrap_or_else(|e| panic!("{}", e))
    }

    /// Set last event id
    /// ## Parameters
    /// - `id`: [`EventId`] - last event id
    ///
    /// ## Errors
    /// - `BotError::System` - mutex lock error
    pub fn set_last_event_id(&self, id: EventId) {
        *self
            .event_id
            .lock()
            .map_err(|e| BotError::System(format!("Mutex lock error: {}", e)))
            .unwrap_or_else(|e| panic!("{}", e)) = id;
    }

    /// Append method path to `base_api_path`
    /// - `path`: [`String`] - append path to `base_api_path`
    pub fn set_path(&self, path: String) -> String {
        let mut full_path = self.base_api_path.clone();
        full_path.push_str(&path);
        full_path
    }

    /// Build full URL with optional query parameters
    /// - `path`: [`String`] - append path to `base_api_path`
    /// - `query`: [`String`] - append `token` query parameter to URL
    ///
    /// ## Errors
    /// - `BotError::Url` - URL parsing error
    ///
    /// Parse with [`Url::parse`]
    pub fn get_parsed_url(&self, path: String, query: String) -> Result<Url> {
        let mut url = self.base_api_url.clone();
        url.set_path(&path);
        url.set_query(Some(&query));
        url.query_pairs_mut().append_pair("token", &self.token);
        Ok(url)
    }

    /// Send request, get response
    /// Serialize request generic type `Rq` with [`serde_url_params::to_string`] into query string
    /// Get response body with [`response`]
    /// Deserialize response with [`serde_json::from_str`]
    /// - `message`: generic type `Rq` - request type
    ///
    /// ## Errors
    /// - `BotError::UrlParams` - URL parameters serialization error
    /// - `BotError::Url` - URL parsing error
    /// - `BotError::Network` - network error when sending request
    /// - `BotError::Serialization` - response deserialization error
    /// - `BotError::Io` - file operation error
    /// - `BotError::Api` - API error when processing request
    ///
    /// ## Panics
    /// - Unable to serialize request
    /// - Unable to parse URL
    /// - Unable to get response body
    /// - Unable to deserialize response
    /// - Unable to send request
    /// - Unable to get response
    /// - Unable to get response status
    ///
    /// [`response`]: #method.response
    pub async fn send_api_request<Rq>(&self, message: Rq) -> Result<<Rq>::ResponseType>
    where
        Rq: BotRequest + Serialize + std::fmt::Debug,
    {
        debug!("Sending API request: {:?}", message);

        let query = serde_url_params::to_string(&message)?;

        let url = self.get_parsed_url(self.set_path(<Rq>::METHOD.to_string()), query.to_owned())?;

        debug!("Request URL: {}", url);

        let body = match <Rq>::HTTP_METHOD {
            HTTPMethod::POST => {
                debug!("Sending POST request with file");
                let form = file_to_multipart(message.get_file()).await?;

                post_response_file(self.client.clone(), url, form).await?
            }
            HTTPMethod::GET => {
                debug!("Sending GET request");
                get_text_response(self.client.clone(), url).await?
            }
        };

        debug!("Received API response: {}", body);
        let response = serde_json::from_str::<<Rq>::ResponseType>(&body)?;

        debug!("Response successfully deserialized");

        Ok(response)
    }
}

impl Default for Bot {
    fn default() -> Self {
        Self::new(APIVersionUrl::V1)
    }
}
