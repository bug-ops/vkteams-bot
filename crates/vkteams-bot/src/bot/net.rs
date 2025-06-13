//! Network module
use crate::api::types::*;
use crate::config::CONFIG;
use crate::error::{BotError, Result};
use reqwest::{
    Body, Client, ClientBuilder, StatusCode, Url,
    multipart::{Form, Part},
};
use std::time::Duration;
use tokio::fs::File;
use tokio::signal;
use tokio::time::sleep;
use tokio_util::codec::{BytesCodec, FramedRead};
use tracing::{debug, error, trace, warn};
/// Connection pool for managing HTTP connections
#[derive(Debug, Clone)]
pub struct ConnectionPool {
    client: Client,
    retries: usize,
    max_backoff: Duration,
}

impl Default for ConnectionPool {
    fn default() -> Self {
        let cfg = &CONFIG.network;
        Self::new(
            Client::new(),
            cfg.retries,
            Duration::from_millis(cfg.max_backoff_ms),
        )
    }
}

impl ConnectionPool {
    /// Create a new connection pool with custom settings
    pub fn new(client: Client, retries: usize, max_backoff: Duration) -> Self {
        Self {
            client,
            retries,
            max_backoff,
        }
    }

    /// Create a connection pool with optimized settings for the VK Teams Bot API
    pub fn optimized() -> Self {
        let cfg = &CONFIG.network;
        let client = build_optimized_client().unwrap_or_else(|e| {
            warn!(
                "Failed to build optimized client. Use default instead: {}",
                e
            );
            Client::new()
        });
        let retries = cfg.retries;
        let max_backoff = Duration::from_millis(cfg.max_backoff_ms);

        Self {
            client,
            retries,
            max_backoff,
        }
    }

    /// Execute a request with exponential backoff retry strategy
    pub async fn execute_with_retry<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        let mut retries = 0;
        let mut backoff_ms = 100;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if let BotError::Network(ref req_err) = e {
                        if !should_retry(req_err) || retries >= self.retries {
                            return Err(e);
                        }

                        retries += 1;
                        let jitter = rand::random::<u64>() % 100;
                        let delay = Duration::from_millis(backoff_ms + jitter);

                        warn!(
                            "Request failed, retrying ({}/{}): {} after {:?}",
                            retries, self.retries, req_err, delay
                        );

                        sleep(delay).await;
                        backoff_ms =
                            std::cmp::min(backoff_ms * 2, self.max_backoff.as_millis() as u64);
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }

    /// Get text response from API with retry capability
    #[tracing::instrument(skip(self))]
    pub async fn get_text(&self, url: Url) -> Result<String> {
        debug!("Getting response from API at path {}...", url);

        self.execute_with_retry(|| {
            let client = self.client.clone();
            let url = url.clone();

            async move {
                let response = client.get(url.as_str()).send().await?;
                trace!("Response status: {}", response.status());

                validate_response(&response.status())?;

                let text = response.text().await?;
                trace!("Response body length: {} bytes", text.len());
                Ok(text)
            }
        })
        .await
    }

    /// Get bytes response from API with retry capability
    #[tracing::instrument(skip(self))]
    pub async fn get_bytes(&self, url: Url) -> Result<Vec<u8>> {
        debug!("Getting binary response from API at path {}...", url);

        self.execute_with_retry(|| {
            let client = self.client.clone();
            let url = url.clone();

            async move {
                let response = client.get(url.as_str()).send().await?;
                trace!("Response status: {}", response.status());

                validate_response(&response.status())?;

                let bytes = response.bytes().await?;
                trace!("Response body size: {} bytes", bytes.len());
                Ok(bytes.to_vec())
            }
        })
        .await
    }

    /// Post file to API with retry capability
    #[tracing::instrument(skip(self, form))]
    pub async fn post_file(&self, url: Url, form: Form) -> Result<String> {
        debug!("Sending file to API at path {}...", url);

        // Since Form doesn't implement Clone, we need to handle it specially
        // We'll pass the form directly to the first attempt,
        // and if retries are needed, we'll notify the caller
        let response = self.client.post(url.as_str()).multipart(form).send().await;

        match response {
            Ok(response) => {
                trace!("Response status: {}", response.status());

                // Validate the response
                validate_response(&response.status())?;

                // Get the response text
                let text = response.text().await?;
                trace!("Response body length: {} bytes", text.len());
                Ok(text)
            }
            Err(err) => {
                // Handle error with retry logic
                if !should_retry(&err) || self.retries == 0 {
                    return Err(BotError::Network(err));
                }

                warn!(
                    "File upload failed, but cannot retry because Form doesn't implement Clone. Error: {}",
                    err
                );
                Err(BotError::Network(err))
            }
        }
    }
}

/// Validate HTTP response status
fn validate_response(status: &StatusCode) -> Result<()> {
    if status.is_success() {
        Ok(())
    } else if status.is_server_error() {
        warn!("Server error: {}", status);
        Err(BotError::System(format!("Server error: HTTP {}", status)))
    } else if status.is_client_error() {
        error!("Client error: {}", status);
        Err(BotError::Validation(format!("HTTP error: {}", status)))
    } else {
        warn!("Unexpected status code: {}", status);
        Err(BotError::System(format!(
            "Unexpected HTTP status code: {}",
            status
        )))
    }
}

/// Determine if the request should be retried based on the error
fn should_retry(err: &reqwest::Error) -> bool {
    err.is_timeout()
        || err.is_connect()
        || err.is_request()
        || (err.status().is_some_and(|s| s.is_server_error()))
}

/// Build a client with optimized settings for the API
fn build_optimized_client() -> Result<Client> {
    let cfg = &CONFIG.network;
    let builder = ClientBuilder::new()
        .timeout(Duration::from_secs(cfg.request_timeout_secs))
        .connect_timeout(Duration::from_secs(cfg.connect_timeout_secs))
        .pool_idle_timeout(Duration::from_secs(cfg.pool_idle_timeout_secs))
        .tcp_nodelay(true)
        .pool_max_idle_per_host(cfg.max_idle_connections)
        .use_rustls_tls();

    builder.build().map_err(BotError::Network)
}
/// Get bytes response from API
/// Send request with [`Client`] `get` method and get body with [`reqwest::Response`] `bytes` method
/// - `url` - file URL
///
/// ## Errors
/// - `BotError::Network` - network error when sending request or receiving response
///
/// @deprecated Use ConnectionPool::get_bytes instead
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
pub async fn file_to_multipart(file: &MultipartName) -> Result<Form> {
    //Get name of the form part
    let name = file.to_string();
    //Get filename
    let filename = match file {
        MultipartName::File(name) | MultipartName::Image(name) => name,
        _ => return Err(BotError::Validation("File not specified".to_string())),
    };
    //Create stream from file
    let file_stream = make_stream(filename).await?;
    //Create part from stream
    let part = Part::stream(file_stream).file_name(filename.to_string());
    //Create multipart form
    Ok(Form::new().part(name, part))
}
/// Create stream from file
/// - `path` - file path
///
/// ## Errors
/// - `BotError::Io` - error opening file
#[tracing::instrument(skip(path))]
async fn make_stream(path: &String) -> Result<Body> {
    //Open file and check if it exists
    let file = File::open(path).await?;
    //Create stream from file
    let file_stream = Body::wrap_stream(FramedRead::new(file, BytesCodec::new()));
    Ok(file_stream)
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

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;
    use std::time::Duration;

    #[tokio::test]
    async fn test_connection_pool_new_and_default() {
        let client = reqwest::Client::new();
        let pool = ConnectionPool::new(client.clone(), 2, Duration::from_millis(100));
        assert_eq!(pool.retries, 2);
        assert_eq!(pool.max_backoff, Duration::from_millis(100));
        let _default = ConnectionPool::default();
    }

    #[tokio::test]
    async fn test_validate_response_success() {
        assert!(validate_response(&StatusCode::OK).is_ok());
    }

    #[tokio::test]
    async fn test_validate_response_client_error() {
        let err = validate_response(&StatusCode::BAD_REQUEST).unwrap_err();
        match err {
            BotError::Validation(msg) => assert!(msg.contains("HTTP error")),
            _ => panic!("Expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_validate_response_server_error() {
        let err = validate_response(&StatusCode::INTERNAL_SERVER_ERROR).unwrap_err();
        match err {
            BotError::System(msg) => assert!(msg.contains("Server error")),
            _ => panic!("Expected System error"),
        }
    }

    #[tokio::test]
    async fn test_validate_response_unexpected_status() {
        let status = StatusCode::SWITCHING_PROTOCOLS;
        let err = validate_response(&status).unwrap_err();
        match err {
            BotError::System(msg) => assert!(msg.contains("Unexpected HTTP status code")),
            _ => panic!("Expected System error"),
        }
    }
}
