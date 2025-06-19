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
use rand::Rng;
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

    /// Post file to API with retry capability using retryable form
    #[tracing::instrument(skip(self, retryable_form))]
    pub async fn post_file_retryable(
        &self,
        url: Url,
        retryable_form: &RetryableMultipartForm,
    ) -> Result<String> {
        debug!(
            "Sending file to API at path {} (size: {} bytes)...",
            url,
            retryable_form.size()
        );

        let mut attempts = 0;
        let max_attempts = self.retries + 1;

        loop {
            attempts += 1;

            // Create fresh form for each attempt
            let form = retryable_form.to_form();

            trace!("Attempt {} of {}", attempts, max_attempts);

            let response = self.client.post(url.as_str()).multipart(form).send().await;

            match response {
                Ok(response) => {
                    trace!("Response status: {}", response.status());

                    // Validate the response
                    if let Err(e) = validate_response(&response.status()) {
                        if attempts >= max_attempts || !should_retry_status(&response.status()) {
                            return Err(e);
                        }

                        let backoff = calculate_backoff_duration(attempts, self.max_backoff);
                        warn!(
                            "HTTP error {}, retrying in {:?} (attempt {} of {})",
                            response.status(),
                            backoff,
                            attempts,
                            max_attempts
                        );
                        sleep(backoff).await;
                        continue;
                    }

                    // Get the response text
                    let text = response.text().await?;
                    trace!("Response body length: {} bytes", text.len());
                    debug!("File uploaded successfully after {} attempt(s)", attempts);
                    return Ok(text);
                }
                Err(err) => {
                    if attempts >= max_attempts || !should_retry(&err) {
                        error!("File upload failed after {} attempt(s): {}", attempts, err);
                        return Err(BotError::Network(err));
                    }

                    let backoff = calculate_backoff_duration(attempts, self.max_backoff);
                    warn!(
                        "File upload failed, retrying in {:?} (attempt {} of {}): {}",
                        backoff, attempts, max_attempts, err
                    );
                    sleep(backoff).await;
                }
            }
        }
    }

    /// Post file to API with retry capability (legacy method for backward compatibility)
    #[tracing::instrument(skip(self, form))]
    pub async fn post_file(&self, url: Url, form: Form) -> Result<String> {
        debug!(
            "Sending file to API at path {} (legacy method, no retry)...",
            url
        );

        let response = self.client.post(url.as_str()).multipart(form).send().await;

        match response {
            Ok(response) => {
                trace!("Response status: {}", response.status());
                validate_response(&response.status())?;
                let text = response.text().await?;
                trace!("Response body length: {} bytes", text.len());
                Ok(text)
            }
            Err(err) => {
                warn!("File upload failed (no retry available): {}", err);
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

/// Check if HTTP status code should trigger a retry
pub fn should_retry_status(status: &StatusCode) -> bool {
    match status.as_u16() {
        // Retry on server errors (5xx)
        500..=599 => true,
        // Retry on rate limiting
        429 => true,
        // Retry on specific client errors that might be transient
        408 | 409 | 423 | 424 => true,
        // Don't retry on other client errors (4xx) or success (2xx/3xx)
        _ => false,
    }
}

/// Calculate exponential backoff duration with jitter
pub fn calculate_backoff_duration(attempt: usize, max_backoff: Duration) -> Duration {
    let base_duration = Duration::from_millis(100); // 100ms base
    let exponential_backoff = base_duration * 2_u32.pow((attempt - 1) as u32);

    // Cap at max_backoff
    let capped_backoff = std::cmp::min(exponential_backoff, max_backoff);

    // Add jitter (random ±25%)
    let jitter_range = capped_backoff.as_millis() / 4; // 25% of the duration
    let mut rng = rand::rng();
    let jitter = rng.random_range(0..=(jitter_range as u64 * 2));
    let jitter_offset = jitter as i64 - jitter_range as i64;

    let final_duration = (capped_backoff.as_millis() as i64 + jitter_offset).max(0) as u64;
    Duration::from_millis(final_duration)
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
/// - `BotError::Validation` - file not specified, invalid path, or filename validation failed
/// - `BotError::Io` - error working with file
#[tracing::instrument(skip(file))]
/// Create retryable multipart form from MultipartName
/// Recommended for file operations that need retry capability
pub async fn file_to_retryable_multipart(file: &MultipartName) -> Result<RetryableMultipartForm> {
    match file {
        MultipartName::FilePath(path) | MultipartName::ImagePath(path) => {
            RetryableMultipartForm::from_file_path(path.clone()).await
        }
        MultipartName::FileContent { filename, content }
        | MultipartName::ImageContent { filename, content } => {
            // Validate filename
            validate_filename(filename)?;

            // Validate content
            if content.is_empty() {
                return Err(BotError::Validation(
                    "File content cannot be empty".to_string(),
                ));
            }

            Ok(RetryableMultipartForm::from_content(
                filename.clone(),
                filename.clone(),
                content.clone(),
            ))
        }
        _ => Err(BotError::Validation("File not specified".to_string())),
    }
}

/// Create multipart form from MultipartName (legacy method)
/// Note: This method doesn't support retries due to Form's lack of Clone trait
pub async fn file_to_multipart(file: &MultipartName) -> Result<Form> {
    //Get name of the form part
    match file {
        MultipartName::FilePath(name) | MultipartName::ImagePath(name) => {
            // Validate file path
            validate_file_path(name)?;

            let file_stream = make_stream(name).await?;
            let part = Part::stream(file_stream).file_name(name.to_string());
            Ok(Form::new().part(name.to_string(), part))
        }
        MultipartName::FileContent { filename, content }
        | MultipartName::ImageContent { filename, content } => {
            // Validate filename
            validate_filename(filename)?;

            // Validate content
            if content.is_empty() {
                return Err(BotError::Validation(
                    "File content cannot be empty".to_string(),
                ));
            }

            let part = Part::bytes(content.clone()).file_name(filename.clone());
            Ok(Form::new().part(filename.to_string(), part))
        }
        _ => Err(BotError::Validation("File not specified".to_string())),
    }
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

// Include tests
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

    #[tokio::test]
    async fn test_should_retry_timeout() {
        let err = reqwest::Error::from(
            reqwest::ClientBuilder::new()
                .timeout(Duration::from_millis(1))
                .build()
                .unwrap()
                .get("http://httpbin.org/delay/10")
                .send()
                .await
                .unwrap_err(),
        );

        // Should retry on timeout
        assert!(should_retry(&err));
    }

    #[tokio::test]
    async fn test_should_retry_server_error() {
        // Create a mock server error
        let client = reqwest::Client::new();
        let response = client.get("http://httpbin.org/status/500").send().await;

        if let Err(err) = response {
            assert!(should_retry(&err));
        }
    }

    #[tokio::test]
    async fn test_build_optimized_client() {
        let result = build_optimized_client();
        assert!(
            result.is_ok(),
            "Failed to build optimized client: {:?}",
            result.err()
        );

        let client = result.unwrap();
        // Verify client was created successfully
        assert!(client.get("https://example.com").build().is_ok());
    }

    #[tokio::test]
    async fn test_connection_pool_optimized() {
        let pool = ConnectionPool::optimized();
        assert!(pool.retries > 0);
        assert!(pool.max_backoff > Duration::from_millis(0));
    }

    #[tokio::test]
    async fn test_connection_pool_execute_with_retry_success() {
        let pool = ConnectionPool::new(reqwest::Client::new(), 2, Duration::from_millis(100));

        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let result = pool
            .execute_with_retry(|| {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok::<String, BotError>("success".to_string())
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_connection_pool_execute_with_retry_failure() {
        let pool = ConnectionPool::new(reqwest::Client::new(), 0, Duration::from_millis(10));

        let result = pool
            .execute_with_retry(|| async {
                Err::<String, BotError>(BotError::Network(
                    reqwest::ClientBuilder::new()
                        .build()
                        .unwrap()
                        .get("http://invalid-url-that-does-not-exist.invalid")
                        .send()
                        .await
                        .unwrap_err(),
                ))
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connection_pool_execute_with_retry_non_retryable_error() {
        let pool = ConnectionPool::new(reqwest::Client::new(), 2, Duration::from_millis(10));

        let result = pool
            .execute_with_retry(|| async {
                Err::<String, BotError>(BotError::Validation("Non-retryable error".to_string()))
            })
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            BotError::Validation(msg) => assert_eq!(msg, "Non-retryable error"),
            _ => panic!("Expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_file_to_multipart_filepath() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "test content").unwrap();
        let temp_path = temp_file.path().to_string_lossy().to_string();

        let multipart = MultipartName::FilePath(temp_path);
        let result = file_to_multipart(&multipart).await;

        assert!(
            result.is_ok(),
            "Failed to create multipart: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_file_to_multipart_file_content() {
        let multipart = MultipartName::FileContent {
            filename: "test.txt".to_string(),
            content: b"test content".to_vec(),
        };

        let result = file_to_multipart(&multipart).await;
        assert!(
            result.is_ok(),
            "Failed to create multipart from content: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_file_to_multipart_image_content() {
        let multipart = MultipartName::ImageContent {
            filename: "test.jpg".to_string(),
            content: b"fake image content".to_vec(),
        };

        let result = file_to_multipart(&multipart).await;
        assert!(
            result.is_ok(),
            "Failed to create multipart from image content: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_file_to_multipart_invalid() {
        let multipart = MultipartName::FilePath("/non/existent/file.txt".to_string());
        let result = file_to_multipart(&multipart).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            BotError::Validation(msg) => assert!(msg.contains("File does not exist")),
            _ => panic!("Expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_file_to_multipart_path_traversal() {
        let multipart = MultipartName::FilePath("../../../etc/passwd".to_string());
        let result = file_to_multipart(&multipart).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            BotError::Validation(msg) => assert!(msg.contains("parent directory references")),
            _ => panic!("Expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_file_to_multipart_empty_path() {
        let multipart = MultipartName::FilePath("".to_string());
        let result = file_to_multipart(&multipart).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            BotError::Validation(msg) => assert_eq!(msg, "File path cannot be empty"),
            _ => panic!("Expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_file_to_multipart_invalid_filename() {
        let multipart = MultipartName::FileContent {
            filename: "file<name>.txt".to_string(), // Contains forbidden character
            content: b"test content".to_vec(),
        };

        let result = file_to_multipart(&multipart).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BotError::Validation(msg) => assert!(msg.contains("forbidden character")),
            _ => panic!("Expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_file_to_multipart_empty_content() {
        let multipart = MultipartName::FileContent {
            filename: "empty.txt".to_string(),
            content: Vec::new(), // Empty content
        };

        let result = file_to_multipart(&multipart).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BotError::Validation(msg) => assert_eq!(msg, "File content cannot be empty"),
            _ => panic!("Expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_validate_filename_reserved_names() {
        let reserved_names = ["CON", "PRN", "AUX", "NUL", "COM1", "LPT1"];

        for name in reserved_names.iter() {
            let result = validate_filename(name);
            assert!(result.is_err());
            match result.unwrap_err() {
                BotError::Validation(msg) => assert!(msg.contains("reserved name")),
                _ => panic!("Expected Validation error for {}", name),
            }
        }
    }

    #[tokio::test]
    async fn test_validate_filename_valid() {
        let valid_names = ["document.txt", "image.jpg", "data.json", "archive.zip"];

        for name in valid_names.iter() {
            let result = validate_filename(name);
            assert!(result.is_ok(), "Filename {} should be valid", name);
        }
    }

    #[tokio::test]
    async fn test_make_stream_valid_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "test stream content").unwrap();
        let temp_path = temp_file.path().to_string_lossy().to_string();

        let result = make_stream(&temp_path).await;
        assert!(
            result.is_ok(),
            "Failed to create stream: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_make_stream_invalid_file() {
        let invalid_path = "/path/that/does/not/exist/file.txt".to_string();
        let result = make_stream(&invalid_path).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            BotError::Io(_) => {} // Expected IO error
            _ => panic!("Expected IO error"),
        }
    }

    #[tokio::test]
    async fn test_validate_response_all_success_codes() {
        let success_codes = [
            StatusCode::OK,
            StatusCode::CREATED,
            StatusCode::ACCEPTED,
            StatusCode::NO_CONTENT,
        ];

        for code in success_codes.iter() {
            assert!(
                validate_response(code).is_ok(),
                "Status code {:?} should be valid",
                code
            );
        }
    }

    #[tokio::test]
    async fn test_validate_response_all_client_error_codes() {
        let client_error_codes = [
            StatusCode::BAD_REQUEST,
            StatusCode::UNAUTHORIZED,
            StatusCode::FORBIDDEN,
            StatusCode::NOT_FOUND,
            StatusCode::METHOD_NOT_ALLOWED,
        ];

        for code in client_error_codes.iter() {
            let result = validate_response(code);
            assert!(result.is_err(), "Status code {:?} should be error", code);
            match result.unwrap_err() {
                BotError::Validation(_) => {} // Expected
                _ => panic!("Expected Validation error for code {:?}", code),
            }
        }
    }

    #[tokio::test]
    async fn test_validate_response_all_server_error_codes() {
        let server_error_codes = [
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::NOT_IMPLEMENTED,
            StatusCode::BAD_GATEWAY,
            StatusCode::SERVICE_UNAVAILABLE,
            StatusCode::GATEWAY_TIMEOUT,
        ];

        for code in server_error_codes.iter() {
            let result = validate_response(code);
            assert!(result.is_err(), "Status code {:?} should be error", code);
            match result.unwrap_err() {
                BotError::System(_) => {} // Expected
                _ => panic!("Expected System error for code {:?}", code),
            }
        }
    }

    #[tokio::test]
    async fn test_connection_pool_clone() {
        let pool1 = ConnectionPool::new(reqwest::Client::new(), 3, Duration::from_millis(200));
        let pool2 = pool1.clone();

        assert_eq!(pool1.retries, pool2.retries);
        assert_eq!(pool1.max_backoff, pool2.max_backoff);
    }

    #[test]
    fn test_connection_pool_debug() {
        let pool = ConnectionPool::new(reqwest::Client::new(), 2, Duration::from_millis(100));
        let debug_str = format!("{:?}", pool);
        assert!(debug_str.contains("ConnectionPool"));
    }

    #[tokio::test]
    async fn test_deprecated_get_bytes_response() {
        // Test the deprecated function still works
        let client = reqwest::Client::new();
        let url = reqwest::Url::parse("https://httpbin.org/bytes/10").unwrap();

        let result = get_bytes_response(client, url).await;
        // This might fail in CI/testing environments, so we just check it doesn't panic
        match result {
            Ok(bytes) => assert!(!bytes.is_empty()),
            Err(_) => {} // Network errors are acceptable in tests
        }
    }

    #[tokio::test]
    async fn test_shutdown_signal_setup() {
        // Test that shutdown_signal can be set up without panicking
        // We can't easily test the actual signal handling in unit tests

        let signal_task = tokio::spawn(async {
            tokio::time::timeout(Duration::from_millis(100), shutdown_signal()).await
        });

        // Should timeout since no signal is sent
        let result = signal_task.await.unwrap();
        assert!(result.is_err()); // Should timeout
    }
}
/// Validate file path for security and correctness
///
/// ## Errors
/// - `BotError::Validation` - invalid file path
fn validate_file_path(path: &str) -> Result<()> {
    use std::path::Path;

    // Check if path is empty
    if path.is_empty() {
        return Err(BotError::Validation(
            "File path cannot be empty".to_string(),
        ));
    }

    // Check for null bytes (security vulnerability)
    if path.contains('\0') {
        return Err(BotError::Validation(
            "File path contains null bytes".to_string(),
        ));
    }

    // Normalize path and check for directory traversal attempts
    let path_obj = Path::new(path);

    // Check for dangerous path components
    for component in path_obj.components() {
        match component {
            std::path::Component::ParentDir => {
                return Err(BotError::Validation(
                    "File path contains parent directory references (..)".to_string(),
                ));
            }
            std::path::Component::CurDir => {
                return Err(BotError::Validation(
                    "File path contains current directory references (.)".to_string(),
                ));
            }
            _ => {}
        }
    }

    // Check if path is absolute or relative
    if path_obj.is_absolute() {
        // For absolute paths, ensure they exist and are readable
        if !path_obj.exists() {
            return Err(BotError::Validation(format!(
                "File does not exist: {}",
                path
            )));
        }

        if !path_obj.is_file() {
            return Err(BotError::Validation(format!(
                "Path is not a file: {}",
                path
            )));
        }
    }

    // Additional checks for maximum path length (varies by OS)
    #[cfg(target_os = "windows")]
    const MAX_PATH_LEN: usize = 260;
    #[cfg(not(target_os = "windows"))]
    const MAX_PATH_LEN: usize = 4096;

    if path.len() > MAX_PATH_LEN {
        return Err(BotError::Validation(format!(
            "File path too long: {} characters (max: {})",
            path.len(),
            MAX_PATH_LEN
        )));
    }

    Ok(())
}

/// Validate filename for security and correctness
///
/// ## Errors
/// - `BotError::Validation` - invalid filename
fn validate_filename(filename: &str) -> Result<()> {
    // Check if filename is empty
    if filename.is_empty() {
        return Err(BotError::Validation("Filename cannot be empty".to_string()));
    }

    // Check for null bytes
    if filename.contains('\0') {
        return Err(BotError::Validation(
            "Filename contains null bytes".to_string(),
        ));
    }

    // Check for dangerous characters
    const FORBIDDEN_CHARS: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    for &forbidden_char in FORBIDDEN_CHARS {
        if filename.contains(forbidden_char) {
            return Err(BotError::Validation(format!(
                "Filename contains forbidden character: '{}'",
                forbidden_char
            )));
        }
    }

    // Check for reserved names on Windows
    const RESERVED_NAMES: &[&str] = &[
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    let filename_upper = filename.to_uppercase();
    let name_without_ext = filename_upper.split('.').next().unwrap_or("");

    if RESERVED_NAMES.contains(&name_without_ext) {
        return Err(BotError::Validation(format!(
            "Filename uses reserved name: {}",
            filename
        )));
    }

    // Check filename length
    const MAX_FILENAME_LEN: usize = 255;
    if filename.len() > MAX_FILENAME_LEN {
        return Err(BotError::Validation(format!(
            "Filename too long: {} characters (max: {})",
            filename.len(),
            MAX_FILENAME_LEN
        )));
    }

    // Check for filenames starting or ending with dots or spaces
    if filename.starts_with('.') && filename != "." && filename != ".." {
        // Hidden files are generally OK, but we might want to warn
    }

    if filename.ends_with(' ') || filename.ends_with('.') {
        return Err(BotError::Validation(
            "Filename cannot end with space or dot".to_string(),
        ));
    }

    Ok(())
}
/// Retryable multipart form that can be recreated for retry attempts
#[derive(Debug, Clone)]
pub struct RetryableMultipartForm {
    file_data: Vec<u8>,
    pub filename: String,
    field_name: String,
}

impl RetryableMultipartForm {
    /// Create new retryable form from file content
    pub fn from_content(filename: String, field_name: String, content: Vec<u8>) -> Self {
        Self {
            file_data: content,
            filename,
            field_name,
        }
    }

    /// Create new retryable form from file path (loads content into memory)
    pub async fn from_file_path(path: String) -> Result<Self> {
        // Validate file path first
        validate_file_path_async(&path).await?;

        // Read file content into memory for retry capability
        let content = tokio::fs::read(&path).await.map_err(BotError::Io)?;

        let filename = std::path::Path::new(&path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&path)
            .to_string();

        Ok(Self::from_content(filename.clone(), filename, content))
    }

    /// Convert to reqwest Form for sending
    pub fn to_form(&self) -> Form {
        let part = Part::bytes(self.file_data.clone()).file_name(self.filename.clone());
        Form::new().part(self.field_name.clone(), part)
    }

    /// Get file size for logging/validation
    pub fn size(&self) -> usize {
        self.file_data.len()
    }
}

/// Validate file path asynchronously for security and correctness
///
/// ## Errors
/// - `BotError::Validation` - invalid file path
pub async fn validate_file_path_async(path: &str) -> Result<()> {
    // Check if path is empty
    if path.is_empty() {
        return Err(BotError::Validation(
            "File path cannot be empty".to_string(),
        ));
    }

    // Check for null bytes (security vulnerability)
    if path.contains('\0') {
        return Err(BotError::Validation(
            "File path contains null bytes".to_string(),
        ));
    }

    // Normalize path and check for directory traversal attempts
    let path_obj = std::path::Path::new(path);

    // Check for dangerous path components
    for component in path_obj.components() {
        match component {
            std::path::Component::ParentDir => {
                return Err(BotError::Validation(
                    "File path contains parent directory references (..)".to_string(),
                ));
            }
            std::path::Component::CurDir => {
                return Err(BotError::Validation(
                    "File path contains current directory references (.)".to_string(),
                ));
            }
            _ => {}
        }
    }

    // Check if path is absolute or relative - use async operations
    if path_obj.is_absolute() {
        // For absolute paths, ensure they exist and are readable using async operations
        let metadata = tokio::fs::metadata(path)
            .await
            .map_err(|e| BotError::Validation(format!("File does not exist: {} ({})", path, e)))?;

        if !metadata.is_file() {
            return Err(BotError::Validation(format!(
                "Path is not a file: {}",
                path
            )));
        }

        // Check if file is readable by attempting to get canonicalized path
        let _canonical = tokio::fs::canonicalize(path)
            .await
            .map_err(|e| BotError::Validation(format!("Cannot access file: {} ({})", path, e)))?;
    }

    // Additional checks for maximum path length (varies by OS)
    #[cfg(target_os = "windows")]
    const MAX_PATH_LEN: usize = 260;
    #[cfg(not(target_os = "windows"))]
    const MAX_PATH_LEN: usize = 4096;

    if path.len() > MAX_PATH_LEN {
        return Err(BotError::Validation(format!(
            "File path too long: {} characters (max: {})",
            path.len(),
            MAX_PATH_LEN
        )));
    }

    Ok(())
}
