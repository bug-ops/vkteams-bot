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
    pub(crate) connection_pool: OnceCell<ConnectionPool>,
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
            connection_pool: OnceCell::new(),
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

                self.connection_pool
                    .get_or_init(ConnectionPool::optimized)
                    .post_file(url, form)
                    .await?
            }
            HTTPMethod::GET => {
                debug!("Sending GET request");
                self.connection_pool
                    .get_or_init(ConnectionPool::optimized)
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
    use reqwest::Url;
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    fn test_runtime() -> Runtime {
        Runtime::new().unwrap()
    }

    #[test]
    fn test_bot_with_params_valid() {
        let url = Url::parse("https://example.com/api").unwrap();
        let token: Arc<str> = Arc::from("test_token");
        let path: Arc<str> = Arc::from("/api");
        let event_id = Arc::new(Mutex::new(0u32));
        let bot = Bot {
            connection_pool: OnceCell::new(),
            token: token.clone(),
            base_api_url: url.clone(),
            base_api_path: path.clone(),
            event_id: event_id.clone(),
            #[cfg(feature = "ratelimit")]
            rate_limiter: OnceCell::new(),
        };
        assert_eq!(bot.token.as_ref(), "test_token");
        assert_eq!(bot.base_api_url, url);
        assert_eq!(bot.base_api_path.as_ref(), "/api");
    }

    #[test]
    fn test_bot_with_params_invalid_url() {
        let url = Url::parse("");
        assert!(url.is_err());
    }

    #[test]
    fn test_bot_with_default_version_valid() {
        let url = Url::parse("https://example.com/api").unwrap();
        let token: Arc<str> = Arc::from("test_token");
        let bot = Bot {
            connection_pool: OnceCell::new(),
            token: token.clone(),
            base_api_url: url.clone(),
            base_api_path: Arc::from("/api"),
            event_id: Arc::new(Mutex::new(0u32)),
            #[cfg(feature = "ratelimit")]
            rate_limiter: OnceCell::new(),
        };
        assert_eq!(bot.token.as_ref(), "test_token");
    }

    #[test]
    fn test_bot_with_default_version_invalid_url() {
        let url = Url::parse("not a url");
        assert!(url.is_err());
    }

    #[test]
    fn test_set_and_get_last_event_id() {
        let url = Url::parse("https://example.com/api").unwrap();
        let token: Arc<str> = Arc::from("test_token");
        let bot = Bot {
            connection_pool: OnceCell::new(),
            token: token.clone(),
            base_api_url: url.clone(),
            base_api_path: Arc::from("/api"),
            event_id: Arc::new(Mutex::new(0u32)),
            #[cfg(feature = "ratelimit")]
            rate_limiter: OnceCell::new(),
        };
        let rt = test_runtime();
        rt.block_on(async {
            let mut lock = bot.event_id.lock().await;
            *lock = 42u32;
        });
        let rt = test_runtime();
        rt.block_on(async {
            let lock = bot.event_id.lock().await;
            assert_eq!(*lock, 42u32);
        });
    }

    #[tokio::test]
    async fn test_get_and_set_last_event_id_async() {
        let bot =
            Bot::with_params(&APIVersionUrl::V1, "test_token", "https://example.com").unwrap();

        // Test initial value
        assert_eq!(bot.get_last_event_id().await, 0);

        // Test setting and getting
        bot.set_last_event_id(123).await;
        assert_eq!(bot.get_last_event_id().await, 123);

        // Test setting another value
        bot.set_last_event_id(456).await;
        assert_eq!(bot.get_last_event_id().await, 456);
    }

    #[test]
    fn test_set_path() {
        let bot =
            Bot::with_params(&APIVersionUrl::V1, "test_token", "https://example.com").unwrap();

        let path = bot.set_path("/messages/sendText");
        assert_eq!(path, "/api/v1/messages/sendText");

        let path2 = bot.set_path("/chats/getInfo");
        assert_eq!(path2, "/api/v1/chats/getInfo");
    }

    #[test]
    fn test_get_parsed_url_basic() {
        let bot =
            Bot::with_params(&APIVersionUrl::V1, "test_token", "https://api.example.com").unwrap();

        let path = "/api/v1/messages/sendText".to_string();
        let query = "chatId=test@chat.agent&text=hello".to_string();

        let result = bot.get_parsed_url(path, query);
        assert!(result.is_ok());

        let url = result.unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("api.example.com"));
        assert_eq!(url.path(), "/api/v1/messages/sendText");
        assert!(url.query().unwrap().contains("token=test_token"));
        assert!(url.query().unwrap().contains("chatId=test@chat.agent"));
        assert!(url.query().unwrap().contains("text=hello"));
    }

    #[test]
    fn test_get_parsed_url_with_special_chars() {
        let bot = Bot::with_params(
            &APIVersionUrl::V1,
            "special_token",
            "https://api.example.com",
        )
        .unwrap();

        let path = "/api/v1/messages/sendText".to_string();
        let query = "text=hello world&chatId=test+chat".to_string();

        let result = bot.get_parsed_url(path, query);
        assert!(result.is_ok());

        let url = result.unwrap();
        assert!(url.query().unwrap().contains("token=special_token"));
    }

    #[test]
    fn test_bot_debug_format() {
        let bot = Bot::with_params(
            &APIVersionUrl::V1,
            "debug_token",
            "https://debug.example.com",
        )
        .unwrap();
        let debug_str = format!("{:?}", bot);

        assert!(debug_str.contains("Bot"));
        assert!(debug_str.contains("debug_token"));
        assert!(debug_str.contains("debug.example.com"));
        assert!(debug_str.contains("<pool>"));
    }

    #[test]
    fn test_bot_clone() {
        let bot1 = Bot::with_params(
            &APIVersionUrl::V1,
            "clone_token",
            "https://clone.example.com",
        )
        .unwrap();
        let bot2 = bot1.clone();

        assert_eq!(bot1.token, bot2.token);
        assert_eq!(bot1.base_api_url, bot2.base_api_url);
        assert_eq!(bot1.base_api_path, bot2.base_api_path);
    }

    #[test]
    fn test_bot_with_default_version() {
        let result = Bot::with_default_version("default_token", "https://default.example.com");
        assert!(result.is_ok());

        let bot = result.unwrap();
        assert_eq!(bot.token.as_ref(), "default_token");
        assert_eq!(bot.base_api_path.as_ref(), "/api/v1");
        assert_eq!(bot.base_api_url.as_str(), "https://default.example.com/");
    }

    #[test]
    fn test_bot_with_params_invalid_urls() {
        let invalid_urls = [
            "",
            "not-a-url",
            "ftp://invalid-scheme.com",
            "://missing-scheme.com",
        ];

        for invalid_url in invalid_urls.iter() {
            let result = Bot::with_params(&APIVersionUrl::V1, "token", invalid_url);
            assert!(result.is_err(), "Should fail for URL: {}", invalid_url);

            match result.unwrap_err() {
                BotError::Url(_) => {} // Expected
                _ => panic!("Expected URL error for: {}", invalid_url),
            }
        }
    }

    #[test]
    fn test_bot_with_empty_token() {
        let result = Bot::with_params(&APIVersionUrl::V1, "", "https://example.com");
        assert!(result.is_ok()); // Empty token is allowed, validation happens at API level

        let bot = result.unwrap();
        assert_eq!(bot.token.as_ref(), "");
    }

    #[tokio::test]
    async fn test_concurrent_event_id_access() {
        let bot = Bot::with_params(
            &APIVersionUrl::V1,
            "concurrent_token",
            "https://example.com",
        )
        .unwrap();

        let bot_clone = bot.clone();
        let handle1 = tokio::spawn(async move {
            for i in 0..100 {
                bot_clone.set_last_event_id(i).await;
                tokio::task::yield_now().await;
            }
        });

        let bot_clone2 = bot.clone();
        let handle2 = tokio::spawn(async move {
            for _ in 0..100 {
                let _ = bot_clone2.get_last_event_id().await;
                tokio::task::yield_now().await;
            }
        });

        let _ = tokio::join!(handle1, handle2);

        // Test completed without deadlock
        assert!(true);
    }
}
