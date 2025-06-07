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
use net::ConnectionPool;
use net::*;
use once_cell::sync::OnceCell;
use reqwest::Url;
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
    pub(crate) connection_pool: OnceCell<ConnectionPool>,
    pub(crate) token: Arc<str>,
    pub(crate) base_api_url: Url,
    pub(crate) base_api_path: Arc<str>,
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

        let token = get_env_token().expect("Failed to get token from environment");
        debug!("Token successfully obtained from environment");

        let base_api_url = get_env_url().expect("Failed to get API URL from environment");
        debug!("API URL successfully obtained from environment");

        Self::with_params(&version, token.as_str(), base_api_url.as_str())
            .expect("Failed to create bot")
    }

    /// Creates a new `Bot` with direct parameters instead of environment variablesx
    ///
    /// This method allows you to create a bot by directly providing the token and API URL,
    /// instead of reading them from environment variables. This is particularly useful
    /// when:
    /// - You want to manage credentials programmatically
    /// - You're integrating with a system that doesn't use environment variables
    /// - You're testing with different credentials
    ///
    /// Uses ConnectionPool with optimized settings for HTTP requests.
    ///
    /// ## Parameters
    /// - `version`: [`APIVersionUrl`] - API version
    /// - `token`: [`String`] - Bot API token
    /// - `api_url`: [`String`] - Base API URL
    ///
    /// ## Errors
    /// - `BotError::Url` - URL parsing error
    /// - `BotError::Network` - network client creation error
    ///
    /// ## Example
    /// ```no_run
    /// use vkteams_bot::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let bot = Bot::with_params(
    ///         APIVersionUrl::V1,
    ///         "your_bot_token".to_string(),
    ///         "https://api.example.com".to_string()
    ///     )?;
    ///
    ///     // Now use the bot...
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// For most cases, consider using [`with_default_version`](#method.with_default_version)
    /// which uses V1 API version and has a simpler signature.
    pub fn with_params(version: &APIVersionUrl, token: &str, api_url: &str) -> Result<Self> {
        debug!("Creating new bot with API version: {:?}", version);
        debug!("Using provided token and API URL");

        let base_api_url = Url::parse(&api_url).map_err(BotError::Url)?;
        debug!("API URL successfully parsed");

        let base_api_path = version.to_string();
        debug!("Set API base path: {}", base_api_path);

        Ok(Self {
            connection_pool: OnceCell::new(),
            token: Arc::<str>::from(token),
            base_api_url,
            base_api_path: Arc::<str>::from(base_api_path),
            event_id: Arc::new(Mutex::new(0)),
            #[cfg(feature = "ratelimit")]
            rate_limiter: Default::default(),
        })
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
    pub fn set_path(&self, path: &str) -> String {
        let mut full_path = self.base_api_path.as_ref().to_string();
        full_path.push_str(path);
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
        let url = self.get_parsed_url(self.set_path(<Rq>::METHOD), query.to_owned())?;

        debug!("Request URL: {}", url.path());

        let body = match <Rq>::HTTP_METHOD {
            HTTPMethod::POST => {
                debug!("Sending POST request with file");
                let form = file_to_multipart(message.get_file()).await?;

                self.connection_pool
                    .get_or_init(|| ConnectionPool::optimized())
                    .post_file(url, form)
                    .await?
            }
            HTTPMethod::GET => {
                debug!("Sending GET request");
                self.connection_pool
                    .get_or_init(|| ConnectionPool::optimized())
                    .get_text(url)
                    .await?
            }
        };

        let response: ApiResponseWrapper<<Rq>::ResponseType> = serde_json::from_str(&body)?;
        response.into()
    }
}

impl Default for Bot {
    fn default() -> Self {
        Self::new(APIVersionUrl::V1)
    }
}

impl Bot {
    /// Creates a new bot with default API version (V1) and direct parameters
    ///
    /// This is the recommended method for creating a bot with direct parameters.
    /// It uses the default connection pool with optimized settings.
    ///
    /// ## Parameters
    /// - `token`: [`String`] - Bot API token
    /// - `api_url`: [`String`] - Base API URL
    ///
    /// ## Errors
    /// - `BotError::Url` - URL parsing error
    /// - `BotError::Network` - network client creation error
    ///
    /// ## Example
    /// ```no_run
    /// use vkteams_bot::{Bot, error::Result};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let bot = Bot::with_default_version(
    ///         "your_bot_token".to_string(),
    ///         "https://api.example.com".to_string()
    ///     )?;
    ///
    ///     // Now use the bot...
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn with_default_version(token: &str, api_url: &str) -> Result<Self> {
        Self::with_params(&APIVersionUrl::V1, token, api_url)
    }
}

fn get_env_token() -> Result<String> {
    std::env::var(VKTEAMS_BOT_API_TOKEN).map_err(BotError::from)
}

fn get_env_url() -> Result<String> {
    std::env::var(VKTEAMS_BOT_API_URL).map_err(BotError::from)
}

fn set_default_path(version: &APIVersionUrl) -> String {
    version.to_string()
}

// Helper functions to create a new bot with a custom connection pool
impl Bot {
    /// Create a new bot with a custom connection pool
    /// Useful for testing or specific connection requirements
    ///
    /// Gets token and API URL from environment variables
    pub fn with_connection_pool(
        version: APIVersionUrl,
        connection_pool: OnceCell<ConnectionPool>,
    ) -> Result<Self> {
        debug!("Creating new bot with custom connection pool");

        let token = get_env_token()?;
        let base_api_url = get_env_url()?;

        Self::with_connection_pool_and_params(version, token, base_api_url, connection_pool)
    }

    /// Create a new bot with a custom connection pool and direct parameters
    ///
    /// This method provides maximum flexibility by allowing you to specify both
    /// direct parameters (token and API URL) and a custom connection pool.
    /// This is particularly useful for:
    /// - Advanced testing scenarios with custom network configurations
    /// - Specialized connection requirements (timeouts, retries, etc.)
    /// - Sharing connection pools between multiple bot instances
    ///
    /// ## Parameters
    /// - `version`: [`APIVersionUrl`] - API version to use
    /// - `token`: [`String`] - Bot API token for authentication
    /// - `api_url`: [`String`] - Base API URL for requests
    /// - `connection_pool`: [`ConnectionPool`] - Custom configured connection pool
    ///
    /// ## Errors
    /// - `BotError::Url` - URL parsing error if api_url is invalid
    /// - Other errors that might be propagated from connection pool operations
    ///
    /// ## Example
    /// ```no_run
    /// use vkteams_bot::prelude::*;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     // Create a custom connection pool with specific settings
    ///     let pool = ConnectionPool::new(
    ///         reqwest::Client::new(),
    ///         5, // 5 retries
    ///         Duration::from_secs(10) // 10 second max backoff
    ///     );
    ///
    ///     let bot = Bot::with_connection_pool_and_params(
    ///         APIVersionUrl::V1,
    ///         "your_bot_token".to_string(),
    ///         "https://api.example.com".to_string(),
    ///         pool
    ///     )?;
    ///
    ///     // Now use the bot...
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn with_connection_pool_and_params(
        version: APIVersionUrl,
        token: String,
        api_url: String,
        connection_pool: OnceCell<ConnectionPool>,
    ) -> Result<Self> {
        debug!("Creating new bot with custom connection pool and direct parameters");

        let base_api_url = Url::parse(&api_url).map_err(BotError::Url)?;
        debug!("API URL successfully parsed");

        let base_api_path = set_default_path(&version);

        Ok(Self {
            connection_pool,
            token: Arc::<str>::from(token),
            base_api_url,
            base_api_path: Arc::<str>::from(base_api_path),
            event_id: Arc::new(Mutex::new(0)),
            #[cfg(feature = "ratelimit")]
            rate_limiter: Default::default(),
        })
    }
}
