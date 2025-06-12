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
use crate::bot::net::ConnectionPoolTrait;
#[cfg(feature = "ratelimit")]
use crate::bot::ratelimit::RateLimiter;
use crate::error::{BotError, Result};
use net::ConnectionPool;
use net::*;
use once_cell::sync::OnceCell;
use reqwest::Url;
use serde::Serialize;
use std::fmt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

#[derive(Clone)]
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
    pub(crate) connection_pool: Box<dyn ConnectionPoolTrait>,
    pub(crate) token: Arc<str>,
    pub(crate) base_api_url: Url,
    pub(crate) base_api_path: Arc<str>,
    pub(crate) event_id: Arc<Mutex<EventId>>,
    #[cfg(feature = "ratelimit")]
    pub(crate) rate_limiter: OnceCell<Arc<Mutex<RateLimiter>>>,
}

impl fmt::Debug for Bot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bot")
            .field("connection_pool", &"<pool>")
            .field("token", &self.token)
            .field("base_api_url", &self.base_api_url)
            .field("base_api_path", &self.base_api_path)
            .field("event_id", &self.event_id)
            .finish()
    }
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
    ///         &APIVersionUrl::V1,
    ///         "your_bot_token",
    ///         "https://api.example.com"
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

        let base_api_url = Url::parse(api_url).map_err(BotError::Url)?;
        debug!("API URL successfully parsed");

        let base_api_path = version.to_string();
        debug!("Set API base path: {}", base_api_path);

        Ok(Self {
            connection_pool: Box::new(ConnectionPool::optimized()),
            token: Arc::<str>::from(token),
            base_api_url,
            base_api_path: Arc::<str>::from(base_api_path),
            event_id: Arc::new(Mutex::new(0)),
            #[cfg(feature = "ratelimit")]
            rate_limiter: OnceCell::new(),
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
                let mut rate_limiter = self
                    .rate_limiter
                    .get_or_init(|| Arc::new(Mutex::new(RateLimiter::default())))
                    .lock()
                    .await;
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
                debug!(
                    "Sending POST request {:?} {:?}",
                    message,
                    message.get_multipart()
                );
                let form = file_to_multipart(message.get_multipart()).await?;

                self.connection_pool.post_file(url, form).await?
            }
            HTTPMethod::GET => {
                debug!("Sending GET request");
                self.connection_pool.get_text(url).await?
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
    ///         "your_bot_token",
    ///         "https://api.example.com"
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::APIVersionUrl;

    #[test]
    fn test_bot_with_params_valid() {
        let token = "test_token";
        let url = "https://api.example.com";
        let bot = Bot::with_params(&APIVersionUrl::V1, token, url);
        assert!(bot.is_ok());
        let bot = bot.unwrap();
        assert_eq!(bot.token.as_ref(), token);
        assert_eq!(
            bot.base_api_url.as_str().trim_end_matches('/'),
            url.trim_end_matches('/')
        );
    }

    #[test]
    fn test_bot_with_params_invalid_url() {
        let token = "test_token";
        let url = "not_a_url";
        let bot = Bot::with_params(&APIVersionUrl::V1, token, url);
        assert!(bot.is_err());
    }

    #[test]
    fn test_set_path() {
        let token = "test_token";
        let url = "https://api.example.com";
        let bot = Bot::with_params(&APIVersionUrl::V1, token, url).unwrap();
        let path = bot.set_path("/messages/send");
        assert!(path.ends_with("/messages/send"));
    }

    #[test]
    fn test_get_parsed_url() {
        let token = "test_token";
        let url = "https://api.example.com";
        let bot = Bot::with_params(&APIVersionUrl::V1, token, url).unwrap();
        let path = bot.set_path("/messages/send");
        let query = "foo=bar".to_string();
        let parsed = bot.get_parsed_url(path.clone(), query.clone());
        assert!(parsed.is_ok());
        let url = parsed.unwrap();
        assert_eq!(
            url.path().trim_start_matches('/'),
            path.trim_start_matches('/')
        );
        assert!(url.query().unwrap().contains("foo=bar"));
        assert!(url.query().unwrap().contains("token=test_token"));
    }

    #[tokio::test]
    async fn test_set_and_get_last_event_id() {
        let token = "test_token";
        let url = "https://api.example.com";
        let bot = Bot::with_params(&APIVersionUrl::V1, token, url).unwrap();
        bot.set_last_event_id(42).await;
        let id = bot.get_last_event_id().await;
        assert_eq!(id, 42);
    }

    #[tokio::test]
    async fn test_send_api_request_serialization_error() {
        #[derive(Debug, Default)]
        struct BadRequest;
        impl serde::Serialize for BadRequest {
            fn serialize<S>(&self, _: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                Err(serde::ser::Error::custom("fail"))
            }
        }
        impl crate::api::types::BotRequest for BadRequest {
            type Args = ();
            const METHOD: &'static str = "bad";
            type RequestType = Self;
            type ResponseType = ();
            fn new(_: Self::Args) -> Self {
                BadRequest
            }
            fn get_chat_id(&self) -> Option<&crate::api::types::ChatId> {
                None
            }
            fn get_multipart(&self) -> &crate::api::types::MultipartName {
                &crate::api::types::MultipartName::None
            }
        }
        let token = "test_token";
        let url = "https://api.example.com";
        let bot = Bot::with_params(&APIVersionUrl::V1, token, url).unwrap();
        let result = bot.send_api_request(BadRequest).await;
        assert!(matches!(result, Err(BotError::UrlParams(_))));
    }

    #[tokio::test]
    async fn test_send_api_request_deserialization_error() {
        use crate::api::types::{BotRequest, HTTPMethod, MultipartName};
        use crate::bot::net::ConnectionPoolTrait;
        use serde::Serialize;

        #[derive(Debug, Default)]
        struct DummyRequest;
        impl Serialize for DummyRequest {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_unit()
            }
        }
        impl BotRequest for DummyRequest {
            type Args = ();
            const METHOD: &'static str = "dummy";
            const HTTP_METHOD: HTTPMethod = HTTPMethod::GET;
            type RequestType = Self;
            type ResponseType = String;
            fn new(_: Self::Args) -> Self {
                DummyRequest
            }
            fn get_chat_id(&self) -> Option<&crate::api::types::ChatId> {
                None
            }
            fn get_multipart(&self) -> &MultipartName {
                &MultipartName::None
            }
        }
        struct BadPool;
        impl ConnectionPoolTrait for BadPool {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn clone_box(&self) -> Box<dyn ConnectionPoolTrait> {
                Box::new(BadPool)
            }
            fn get_text<'a>(
                &'a self,
                _url: reqwest::Url,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = crate::error::Result<String>> + Send + 'a>,
            > {
                Box::pin(async { Ok("not a json".to_string()) })
            }
            fn post_file<'a>(
                &'a self,
                _url: reqwest::Url,
                _form: reqwest::multipart::Form,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = crate::error::Result<String>> + Send + 'a>,
            > {
                Box::pin(async { Ok("not a json".to_string()) })
            }
        }
        let token = "test_token";
        let url = "https://api.example.com";
        let mut bot = Bot::with_params(&APIVersionUrl::V1, token, url).unwrap();
        bot.connection_pool = Box::new(BadPool);
        let result = bot.send_api_request(DummyRequest).await;
        assert!(matches!(result, Err(BotError::Serialization(_))));
    }

    #[tokio::test]
    async fn test_send_api_request_network_error() {
        use crate::api::types::{BotRequest, HTTPMethod, MultipartName};
        use crate::bot::net::ConnectionPoolTrait;
        use serde::Serialize;

        #[derive(Debug, Default)]
        struct DummyRequest;
        impl Serialize for DummyRequest {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_unit()
            }
        }
        impl BotRequest for DummyRequest {
            type Args = ();
            const METHOD: &'static str = "dummy";
            const HTTP_METHOD: HTTPMethod = HTTPMethod::GET;
            type RequestType = Self;
            type ResponseType = String;
            fn new(_: Self::Args) -> Self {
                DummyRequest
            }
            fn get_chat_id(&self) -> Option<&crate::api::types::ChatId> {
                None
            }
            fn get_multipart(&self) -> &MultipartName {
                &MultipartName::None
            }
        }
        let token = "test_token";
        // Используем гарантированно несуществующий адрес
        let url = "http://localhost:0";
        let mut bot = Bot::with_params(&APIVersionUrl::V1, token, url).unwrap();
        bot.connection_pool = Box::new(ConnectionPool::optimized());
        let result = bot.send_api_request(DummyRequest).await;
        assert!(matches!(result, Err(BotError::Network(_))));
    }

    #[test]
    fn test_bot_with_default_version_valid() {
        let token = "test_token";
        let url = "https://api.example.com";
        let bot = Bot::with_default_version(token, url);
        assert!(bot.is_ok());
        let bot = bot.unwrap();
        assert_eq!(bot.token.as_ref(), token);
        assert_eq!(
            bot.base_api_url.as_str().trim_end_matches('/'),
            url.trim_end_matches('/')
        );
    }

    #[test]
    fn test_bot_with_default_version_invalid_url() {
        let token = "test_token";
        let url = "not_a_url";
        let bot = Bot::with_default_version(token, url);
        assert!(bot.is_err());
    }

    #[test]
    fn test_set_path_edge_cases() {
        let token = "test_token";
        let url = "https://api.example.com";
        let bot = Bot::with_params(&APIVersionUrl::V1, token, url).unwrap();
        assert!(bot.set_path("").ends_with("bot/v1/"));
        assert!(bot.set_path("/foo").ends_with("bot/v1//foo"));
        assert!(bot.set_path("foo/bar").ends_with("bot/v1/foo/bar"));
    }

    #[test]
    fn test_get_parsed_url_edge_cases() {
        let token = "test_token";
        let url = "https://api.example.com";
        let bot = Bot::with_params(&APIVersionUrl::V1, token, url).unwrap();
        let path = bot.set_path("");
        let parsed = bot.get_parsed_url(path.clone(), "".to_string());
        assert!(parsed.is_ok());
        let url = parsed.unwrap();
        assert!(url.path().ends_with("bot/v1/"));
        assert!(url.query().unwrap().contains("token=test_token"));
    }
}
