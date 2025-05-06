//! Network module
use crate::api::types::*;
use crate::error::{BotError, Result};
use reqwest::{
    Body, Client, Url,
    multipart::{Form, Part},
};
use std::time::Duration;
use tokio::fs::File;
use tokio::signal;
use tokio_util::codec::{BytesCodec, FramedRead};
/// Get text response from API
/// Send request with [`Client`] `get` method and get body with [`reqwest::Response`] `text` method
/// - `url` - file URL
///
/// ## Errors
/// - `BotError::Network` - network error when sending request or receiving response
#[tracing::instrument(skip(client))]
pub async fn get_text_response(client: Client, url: Url) -> Result<String> {
    debug!("Getting response from API at path {}...", url);
    let response = client.get(url.as_str()).send().await?;
    trace!("Response status: {}", response.status());
    let text = response.text().await?;
    trace!("Response body: {}", text);
    Ok(text)
}
/// Get bytes response from API
/// Send request with [`Client`] `get` method and get body with [`reqwest::Response`] `bytes` method
/// - `url` - file URL
///
/// ## Errors
/// - `BotError::Network` - network error when sending request or receiving response
#[tracing::instrument(skip(client))]
pub async fn get_bytes_response(client: Client, url: Url) -> Result<Vec<u8>> {
    debug!("Getting binary response from API at path {}...", url);
    let response = client.get(url.as_str()).send().await?;
    trace!("Response status: {}", response.status());
    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}
/// Upload file stream to API in multipart form
/// - `file` - file name
///
/// ## Errors
/// - `BotError::Validation` - file not specified
/// - `BotError::Io` - error working with file
#[tracing::instrument(skip(file))]
pub async fn file_to_multipart(file: MultipartName) -> Result<Form> {
    //Get name of the form part
    let name = file.to_string();
    //Get filename
    let filename = match file {
        MultipartName::File(name) | MultipartName::Image(name) => name,
        _ => return Err(BotError::Validation("File not specified".to_string())),
    };
    //Create stream from file
    let file_stream = make_stream(filename.to_owned()).await?;
    //Create part from stream
    let part = Part::stream(file_stream).file_name(filename.to_owned());
    //Create multipart form
    Ok(Form::new().part(name, part))
}
/// Create stream from file
/// - `path` - file path
///
/// ## Errors
/// - `BotError::Io` - error opening file
#[tracing::instrument(skip(path))]
async fn make_stream(path: String) -> Result<Body> {
    //Open file and check if it exists
    let file = File::open(path.to_owned()).await?;
    //Create stream from file
    let file_stream = Body::wrap_stream(FramedRead::new(file, BytesCodec::new()));
    Ok(file_stream)
}
/// Get raw response from API
/// Send request with [`Client`] `post` method with body file streaming and get body with [`reqwest::Response`] `text` method
///
/// ## Errors
/// - `BotError::Network` - network error when sending request or receiving response
#[tracing::instrument(skip(client, form))]
pub async fn post_response_file(client: Client, url: Url, form: Form) -> Result<String> {
    debug!("Sending file to API at path {}...", url);
    let response = client.post(url.as_str()).multipart(form).send().await?;
    trace!("Response status: {}", response.status());
    let text = response.text().await?;
    trace!("Response body: {}", text);
    Ok(text)
}
/// Set default request settings: timeout, tcp
///
/// Set connection timeout to [`POLL_DURATION`] constant
///
/// Set `timeout` to 5 secs
///
/// Set `tcp_nodelay` to true
pub fn default_reqwest_settings() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .timeout(*POLL_DURATION)
        .tcp_nodelay(true)
        .connect_timeout(Duration::from_secs(5))
}
/// Set default path depending on API version
///
/// default is [`APIVersionUrl::V1`]
///
/// ## Panics
///
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
///
/// ## Errors
/// - `BotError::Config` - environment variable not found
///
/// ## Panics
///
/// - Unable to find environment variable
pub fn get_env_token() -> String {
    std::env::var(VKTEAMS_BOT_API_TOKEN)
        .map_err(|e| {
            BotError::Config(format!(
                "Failed to find environment variable VKTEAMS_BOT_API_TOKEN: {}",
                e
            ))
        })
        .unwrap_or_else(|e| panic!("{}", e))
}
/// Get base api url from [`VKTEAMS_BOT_API_URL`] environment variable
///
/// ## Errors
/// - `BotError::Config` - failed to find or parse URL
///
/// ## Panics
///
/// - Unable to find environment variable
/// - Unable to parse url
pub fn get_env_url() -> Url {
    let url_str = std::env::var(VKTEAMS_BOT_API_URL)
        .map_err(|e| {
            BotError::Config(format!(
                "Failed to find environment variable VKTEAMS_BOT_API_URL: {}",
                e
            ))
        })
        .unwrap_or_else(|e| panic!("{}", e));

    Url::parse(&url_str)
        .map_err(|e| BotError::Config(format!("Failed to parse URL VKTEAMS_BOT_API_URL: {}", e)))
        .unwrap_or_else(|e| panic!("{}", e))
}
/// Graceful shutdown signal
///
/// ## Errors
/// - `BotError::System` - error setting up signal handlers
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .map_err(|e| BotError::System(format!("Failed to set up Ctrl+C handler: {}", e)))
            .unwrap_or_else(|e| panic!("{}", e));
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .map_err(|e| BotError::System(format!("Failed to set up signal handler: {}", e)))
            .unwrap_or_else(|e| panic!("{}", e))
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
