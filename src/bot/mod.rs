#[cfg(feature = "grpc")]
pub mod grpc;
#[cfg(feature = "longpoll")]
pub mod longpoll;
pub mod net;
#[cfg(feature = "webhook")]
pub mod webhook;

use crate::api::types::*;
use anyhow::Result;
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
/// - `evtent_id`: [`std::sync::Arc<_>`]
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
impl Default for Bot {
    // default API version V1
    fn default() -> Self {
        Self::new(APIVersionUrl::V1)
    }
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
    /// ## Panics
    /// - Unable to parse proxy if its bound in `VKTEAMS_PROXY` env variable
    /// - Unable to build [`reqwest::Client`]
    /// - Unable to find token in .env file
    /// - Unable to find or parse url in .env file
    ///
    /// [`reqwest::Client`]: https://docs.rs/reqwest/latest/reqwest/struct.Client.html
    /// [`reqwest::Proxy::all`]: https://docs.rs/reqwest/latest/reqwest/struct.Proxy.html#method.all
    pub fn new(version: APIVersionUrl) -> Self {
        use reqwest::Proxy;
        // Set default reqwest settings
        let builder = default_reqwest_settings();
        // Set proxy if it is bound
        let client = match std::env::var(VKTEAMS_PROXY).ok() {
            Some(proxy) => builder.proxy(Proxy::all(proxy).expect("Unable to parse proxy")),
            None => builder,
        }
        .build()
        .expect("Unable to build reqwest client");

        Self {
            client,
            // Get token from .env file
            token: get_env_token(),
            // Get API URL from .env file
            base_api_url: get_env_url(),
            // Set default path depending on API version
            base_api_path: set_default_path(&version),
            // Default event id is 0
            event_id: Arc::new(Mutex::new(0)),
        }
    }
    /// Get last event id
    pub fn get_last_event_id(&self) -> EventId {
        *self.event_id.lock().unwrap()
    }
    /// Set last event id
    /// ## Parameters
    /// - `id`: [`EventId`] - last event id
    pub fn set_last_event_id(&self, id: EventId) {
        *self.event_id.lock().unwrap() = id;
    }
    /// Append method path to `base_api_path`
    /// - `path`: [`String`] - append path to `base_api_path`
    pub fn set_path(&self, path: String) -> String {
        // Get base API path
        let mut full_path = self.base_api_path.clone();
        // Append method path
        full_path.push_str(&path);
        // Return full path
        full_path
    }
    /// Build full URL with optional query parameters
    /// - `path`: [`String`] - append path to `base_api_path`
    /// - `query`: [`String`] - append `token` query parameter to URL
    ///
    /// Parse with [`Url::parse`]
    pub fn get_parsed_url(&self, path: String, query: String) -> Result<Url> {
        // Make URL with base API path
        let url = Url::parse(self.base_api_url.as_str());
        match url {
            Ok(mut u) => {
                // Append path to URL
                u.set_path(&path);
                //Set bound query
                u.set_query(Some(&query));
                // Append default query query
                u.query_pairs_mut().append_pair("token", &self.token);
                Ok(u)
            }
            // Error with URL
            Err(e) => {
                error!("Error parse URL: {}", e);
                Err(e.into())
            }
        }
    }
    /// Send request, get response
    /// Serialize request generic type `Rq` with [`serde_url_params::to_string`] into query string
    /// Get response body with [`response`]
    /// Deserialize response with [`serde_json::from_str`]
    /// - `message`: generic type `Rq` - request type
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
        Rq: BotRequest + Serialize,
    {
        // Serialize request type `Rq` with serde_url_params::to_string into query string
        match serde_url_params::to_string(&message) {
            Ok(query) => {
                // Try to parse URL
                match self.get_parsed_url(self.set_path(<Rq>::METHOD.to_string()), query.to_owned())
                {
                    Ok(url) => {
                        // Get response body
                        let body = match <Rq>::HTTP_METHOD {
                            HTTPMethod::POST => {
                                // For POST method get multipart form from file name
                                match file_to_multipart(message.get_file()).await {
                                    Ok(f) => {
                                        // Send file POST request with multipart form
                                        post_response_file(
                                            self.client.clone(),
                                            self.get_parsed_url(
                                                self.set_path(<Rq>::METHOD.to_string()),
                                                query,
                                            )
                                            .unwrap(),
                                            f,
                                        )
                                        .await
                                    }
                                    // Error with file
                                    Err(e) => return Err(e),
                                }
                            }
                            HTTPMethod::GET => {
                                // Simple GET request
                                get_text_response(self.client.clone(), url).await
                            }
                        };
                        // Deserialize response with serde_json::from_str
                        match body {
                            Ok(b) => {
                                let rs = serde_json::from_str::<<Rq>::ResponseType>(b.as_str());
                                match rs {
                                    Ok(r) => Ok(r),
                                    Err(e) => Err(e.into()),
                                }
                            }
                            // Error with response
                            Err(e) => Err(e),
                        }
                    }
                    // Error with URL
                    Err(e) => {
                        error!("Error parse URL: {}", e);
                        Err(e)
                    }
                }
            }
            // Error with parse query
            Err(e) => {
                error!("Error serialize request: {}", e);
                Err(e.into())
            }
        }
    }
}
