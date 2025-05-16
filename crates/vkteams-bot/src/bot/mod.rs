#[cfg(feature = "grpc")]
pub mod grpc;
#[cfg(feature = "longpoll")]
pub mod longpoll;
pub mod net;
#[cfg(feature = "ratelimit")]
pub mod ratelimit;
#[cfg(feature = "webhook")]
pub mod webhook;

use crate::api::types::*;
#[cfg(feature = "ratelimit")]
use crate::bot::ratelimit::RateLimiter;
use crate::error::{BotError, Result};
use net::*;
use reqwest::Url;
use net::ConnectionPool;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

#[derive(Debug, Clone)]
/// Bot class with attributes
/// - `connection_pool`: [`ConnectionPool`] - Pool of HTTP connections for API requests
/// - `token`: [`String`] - Bot API token 
/// - `base_api_url`: [`reqwest::Url`] - Base API URL
/// - `base_api_path`: [`String`] - Base API path
/// - `event_id`: [`std::sync::Arc<_>`] - Last event ID
///
/// [`reqwest::Url`]: https://docs.rs/reqwest/latest/reqwest/struct.Url.html
/// [`std::sync::Arc<_>`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
pub struct Bot {
    pub(crate) connection_pool: ConnectionPool,
    pub(crate) token: String,
    pub(crate) base_api_url: Url,
    pub(crate) base_api_path: String,
    pub(crate) event_id: Arc<Mutex<EventId>>,
    #[cfg(feature = "ratelimit")]
    pub(crate) rate_limiter: Arc<Mutex<RateLimiter>>,
}

impl Bot {
    /// Creates a new `Bot` with API version [`APIVersionUrl`]
    ///
    /// Uses ConnectionPool with optimized settings for HTTP requests.
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
    /// - `BotError::Network` - network client creation error
    ///
    /// ## Panics
    /// - Unable to find token in .env file
    /// - Unable to find or parse url in .env file
    /// - Unable to create connection pool
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
        
        let connection_pool = ConnectionPool::optimized()
            .unwrap_or_else(|e| panic!("Failed to create connection pool: {}", e));
        debug!("Connection pool successfully created");

        Self {
            connection_pool,
            token,
            base_api_url,
            base_api_path: base_api_path.to_string(),
            event_id: Arc::new(Mutex::new(0)),
            #[cfg(feature = "ratelimit")]
            rate_limiter: Default::default(),
        }
    }

    /// Get last event id
    ///
    /// ## Errors
    /// - `BotError::System` - mutex lock error
    pub async fn get_last_event_id(&self) -> EventId {
        *self.event_id.lock().await
    }

    /// Set last event id
    /// ## Parameters
    /// - `id`: [`EventId`] - last event id
    ///
    /// ## Errors
    /// - `BotError::System` - mutex lock error
    pub async fn set_last_event_id(&self, id: EventId) {
        *self.event_id.lock().await = id;
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
    /// Get response body using connection pool
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
    /// - Unable to deserialize response
    ///
    #[tracing::instrument(skip(self, message))]
    pub async fn send_api_request<Rq>(&self, message: Rq) -> Result<<Rq>::ResponseType>
    where
        Rq: BotRequest + Serialize + std::fmt::Debug,
    {
        debug!("Starting send_api_request");
        // Check rate limit for this chat
        #[cfg(feature = "ratelimit")]
        {
            if let Some(chat_id) = message.get_chat_id() {
                let mut rate_limiter = self.rate_limiter.lock().await;
                if !rate_limiter.wait_if_needed(chat_id).await {
                    return Err(BotError::Validation(
                        "Rate limit exceeded for this chat".to_string(),
                    ));
                }
            } else {
                debug!("No chat_id found in message");
            }
        }

        let query = serde_url_params::to_string(&message)?;
        let url = self.get_parsed_url(self.set_path(<Rq>::METHOD.to_string()), query.to_owned())?;

        debug!("Request URL: {}", url.path());

        let body = match <Rq>::HTTP_METHOD {
            HTTPMethod::POST => {
                debug!("Sending POST request with file");
                let form = file_to_multipart(message.get_file()).await?;

                self.connection_pool.post_file(url, form).await?
            }
            HTTPMethod::GET => {
                debug!("Sending GET request");
                self.connection_pool.get_text(url).await?
            }
        };

        let response: <Rq>::ResponseType = serde_json::from_str(&body)?;
        Ok(response)
    }
}

impl Default for Bot {
    fn default() -> Self {
        Self::new(APIVersionUrl::V1)
    }
}

// Helper function to create a new bot with a custom connection pool
impl Bot {
    /// Create a new bot with a custom connection pool
    /// Useful for testing or specific connection requirements
    pub fn with_connection_pool(version: APIVersionUrl, connection_pool: ConnectionPool) -> Result<Self> {
        debug!("Creating new bot with custom connection pool");
        
        let token = std::env::var(VKTEAMS_BOT_API_TOKEN)
            .map_err(|e| BotError::Config(format!("Failed to get token: {}", e)))?;
            
        let base_api_url = std::env::var(VKTEAMS_BOT_API_URL)
            .map_err(|e| BotError::Config(format!("Failed to get API URL: {}", e)))?;
            
        let base_api_url = Url::parse(&base_api_url)
            .map_err(|e| BotError::Config(format!("Invalid API URL format: {}", e)))?;
            
        let base_api_path = match version {
            APIVersionUrl::V1 => "/bot/v1/",
        };
        
        Ok(Self {
            connection_pool,
            token,
            base_api_url,
            base_api_path: base_api_path.to_string(),
            event_id: Arc::new(Mutex::new(0)),
            #[cfg(feature = "ratelimit")]
            rate_limiter: Default::default(),
        })
    }
}
