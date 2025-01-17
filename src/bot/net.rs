//! Network module
use crate::api::types::*;
use anyhow::Result;
use reqwest::{
    multipart::{Form, Part},
    Body, Client, Url,
};
use std::time::Duration;
use tokio::fs::File;
use tokio::signal;
use tokio_util::codec::{BytesCodec, FramedRead};
/// Get text response from API
/// Send request with [`Client`] `get` method and get body with [`reqwest::Response`] `text` method
/// - `url` - file URL
pub async fn get_text_response(client: Client, url: Url) -> Result<String> {
    debug!("Get response from API path {}...", url.to_string());
    match client.get(url.as_str()).send().await {
        Ok(r) => {
            debug!("Response status: OK");
            match r.text().await {
                Ok(t) => {
                    debug!("Response BODY: {}", t);
                    Ok(t)
                }
                Err(e) => Err(e.into()),
            }
        }
        Err(e) => {
            error!("Response status: {}", e);
            Err(e.into())
        }
    }
}
/// Get bytes response from API
/// Send request with [`Client`] `get` method and get body with [`reqwest::Response`] `bytes` method
/// - `url` - file URL
pub async fn get_bytes_response(client: Client, url: Url) -> Result<Vec<u8>> {
    match client.get(url.as_str()).send().await {
        Ok(r) => {
            debug!("Response status: OK");
            match r.bytes().await {
                Ok(b) => Ok(b.to_vec()),
                Err(e) => Err(e.into()),
            }
        }
        Err(e) => {
            error!("Response status: {}", e);
            Err(e.into())
        }
    }
}
/// Upload file stream to API in multipart form
/// - `file` - file name
pub async fn file_to_multipart(file: MultipartName) -> Result<Form> {
    //Get name of the form part
    let name = file.to_string();
    //Get filename
    let filename = match file {
        MultipartName::File(name) | MultipartName::Image(name) => name,
        _ => panic!("No file"),
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
async fn make_stream(path: String) -> Result<Body> {
    //Open file and check if it exists
    match File::open(path.to_owned()).await {
        Ok(file) => {
            //Create stream from file
            let file_stream = Body::wrap_stream(FramedRead::new(file, BytesCodec::new()));
            Ok(file_stream)
        }
        Err(e) => {
            error!("Unable to open file {}: {}", path.to_owned(), e);
            Err(e.into())
        }
    }
}
/// Get raw response from API
/// Send request with [`Client`] `post` method with body file streaming and get body with [`reqwest::Response`] `text` method
pub async fn post_response_file<'a>(
    client: Client,
    url: Url,
    form: Form, // part: MultipartName,
) -> Result<String> {
    match client.post(url.as_str()).multipart(form).send().await {
        Ok(r) => {
            debug!("Response status: OK");
            match r.text().await {
                Ok(t) => {
                    debug!("Response BODY: {}", t);
                    Ok(t)
                }
                Err(e) => Err(e.into()),
            }
        }
        Err(e) => {
            error!("Response status: {}", e);
            Err(e.into())
        }
    }
}
/// Set default request settings: timeout, tcp
///
/// Set connection timeout to [`POLL_DURATION`] constant
///
/// Set `timeout` to 5 secs
///
/// Set `tcp_nodelay` to true
pub fn default_reqwest_settings() -> reqwest::ClientBuilder {
    Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(*POLL_DURATION)
        .tcp_nodelay(true)
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
/// ## Panics
///
/// - Unable to find environment variable
pub fn get_env_token() -> String {
    std::env::var(VKTEAMS_BOT_API_TOKEN).expect("Unable to find VKTEAMS_BOT_API_TOKEN in .env file")
}
/// Get base api url from [`VKTEAMS_BOT_API_URL`] environment variable
///
/// ## Panics
///
/// - Unable to find environment variable
///
/// - Unable to parse url
pub fn get_env_url() -> Url {
    Url::parse(
        std::env::var(VKTEAMS_BOT_API_URL)
            .expect("Unable to find VKTEAMS_BOT_API_URL in .env file")
            .as_str(),
    )
    .expect("Unable to parse VKTEAMS_BOT_API_URL")
}
/// Graceful shutdown signal
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
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
