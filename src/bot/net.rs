use crate::types::*;
use anyhow::Result;
use reqwest::{
    self,
    multipart::{Form, Part},
    Body, Client, Url,
};
use std::time::Duration;
use tokio_util::codec::{BytesCodec, FramedRead};
/// Get raw response from API
/// Send request with [`Client`] `get` method and get body with [`Response`] `text` method
pub async fn get_response(client: Client, url: Url) -> Result<String> {
    debug!("Get response from API path {}...", url.to_string());
    let response = client.get(url.as_str()).send().await;
    match response {
        Ok(r) => {
            debug!("Response status: OK");
            Ok(r.text().await?)
        }
        Err(e) => {
            error!("Response status: {}", e);
            Err(e.into())
        }
    }
}
/// Upload file stream to API in multipart form
fn file_to_multipart(part: MultipartName) -> Form {
    let name = part.to_string();
    let file = part.get_file();
    Form::new().part(
        name,
        Part::stream(Body::wrap_stream(FramedRead::new(file, BytesCodec::new()))),
    )
}
/// Get raw response from API
/// Send request with [`Client`] `post` method with body file streaming and get body with [`Response`] `text` method
pub async fn post_response_file(client: Client, url: Url, part: MultipartName) -> Result<String> {
    let response = client
        .post(url.as_str())
        .multipart(file_to_multipart(part))
        .send()
        .await;
    match response {
        Ok(r) => {
            debug!("Response status: OK");
            Ok(r.text().await?)
        }
        Err(e) => {
            error!("Response status: {}", e);
            Err(e.into())
        }
    }
}
/// Set default request settings: timeout, tcp
/// Set connection timeout to [`POLL_DURATION`] constant
/// Set `timeout` to 5 secs
/// Set `tcp_nodelay` to true
pub fn default_reqwest_settings() -> reqwest::ClientBuilder {
    debug!("Setting up reqwest client");
    Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(POLL_DURATION)
        .tcp_nodelay(true)
}
/// Set default path depending on API version
/// default is [`APIVersionUrl::V1`]
/// ## Panics
/// - Another API version is not supported
pub fn set_default_path(version: &APIVersionUrl) -> String {
    // Choose path depending on API version
    match version {
        APIVersionUrl::V1 => version.to_string(),
        // Another API version is not supported
        // _ => panic!("Unknown API version"),
    }
}
/// Get token from [`VKTEAMS_BOT_API_TOKEN`] environment variable
/// ## Panics
/// - Unable to find environment variable
pub fn get_env_token() -> String {
    std::env::var(VKTEAMS_BOT_API_TOKEN).expect("Unable to find VKTEAMS_BOT_API_TOKEN in .env file")
}
/// Get base api url from [`VKTEAMS_BOT_API_URL`] environment variable
/// ## Panics
/// - Unable to find environment variable
/// - Unable to parse url
pub fn get_env_url() -> Url {
    Url::parse(
        std::env::var(VKTEAMS_BOT_API_URL)
            .expect("Unable to find VKTEAMS_BOT_API_URL in .env file")
            .as_str(),
    )
    .expect("Unable to parse VKTEAMS_BOT_API_URL")
}

impl Bot {
    /// Send request, get response
    /// Clone [`reqwest::Client`] and send request and get body with [`get_response`].
    ///
    /// Build full url from path with [`set_path`] and [`get_parsed_url`] methods
    ///
    /// [`get_parsed_url`]: #method.get_parsed_url
    /// [`set_path`]: #method.set_path
    /// [`reqwest::Client`]: https://docs.rs/reqwest/latest/reqwest/struct.Client.html
    pub async fn response(
        &self,
        path: String,
        query: String,
        file: MultipartName,
    ) -> Result<String> {
        match file {
            MultipartName::File(_) | MultipartName::Image(_) => {
                // Send file POST request
                post_response_file(
                    self.client.clone(),
                    self.get_parsed_url(self.set_path(path), query).unwrap(),
                    file,
                )
                .await
            }
            _ => {
                // Simple GET request
                get_response(
                    self.client.clone(),
                    self.get_parsed_url(self.set_path(path), query).unwrap(),
                )
                .await
            }
        }
    }
    /// Append method path to `base_api_path`
    pub fn set_path(&self, path: String) -> String {
        let mut full_path = self.base_api_path.clone();
        full_path.push_str(&path);
        full_path
    }
    /// Build full URL with optional query parameters
    /// Append path to [`base_api_url`]
    /// Append `token` query parameter to URL
    /// Parse with [`Url::parse`]
    ///
    /// [`base_api_url`]: #structfield.base_api_url
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
            Err(e) => {
                error!("Error parse URL: {}", e);
                Err(e.into())
            }
        }
    }
    /// Send request, get response
    ///
    /// Serialize request type `Rq` with [`serde_url_params::to_string`] into query string
    ///
    /// Get response body with [`response`] method type `M`
    ///
    /// Deserialize response with [`serde_json::from_str`] into association type `Rs`
    ///
    /// [`response`]: #method.response
    pub async fn send_get_request<Rq, Rs>(
        &self,
        request: Rq,
        file: MultipartName,
        method: Methods,
    ) -> Result<Rs>
    where
        Rq: serde::ser::Serialize,
        Rs: serde::de::DeserializeOwned,
    {
        let query = serde_url_params::to_string(&request).unwrap();
        let body = self.response(method.to_string(), query, file).await;
        match body {
            Ok(b) => {
                let rs = serde_json::from_str::<Rs>(b.as_str());
                match rs {
                    Ok(r) => Ok(r),
                    Err(e) => Err(e.into()),
                }
            }
            Err(e) => Err(e),
        }
    }
}
