use crate::api::types::*;
use anyhow::{anyhow, Result};
use reqwest::{
    self,
    multipart::{Form, Part},
    Body, Client, Url,
};
use std::time::Duration;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

// use super::types::Response;
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
pub async fn file_to_multipart(file: Option<MultipartName>) -> Result<Form> {
    match file {
        Some(multipart) => {
            //Get name of the form part
            let name = multipart.to_string();
            //Get filename
            let filename = match multipart {
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
        None => Err(anyhow!("No file")),
    }
}
/// Create stream from file
async fn make_stream(name: String) -> Result<Body> {
    //Open file and check if it exists
    match File::open(name.to_owned()).await {
        Ok(file) => {
            //Create stream from file
            let file_stream = Body::wrap_stream(FramedRead::new(file, BytesCodec::new()));
            Ok(file_stream)
        }
        Err(e) => {
            error!("Unable to open file {}: {}", name.to_owned(), e);
            Err(e.into())
        }
    }
}
/// Get raw response from API
/// Send request with [`Client`] `post` method with body file streaming and get body with [`Response`] `text` method
pub async fn post_response_file<'a>(
    client: Client,
    url: Url,
    form: Form, // part: MultipartName,
) -> Result<String> {
    let response = client.post(url.as_str()).multipart(form).send().await;
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
    Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(*POLL_DURATION)
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
