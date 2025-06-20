use reqwest::StatusCode;
use std::time::Duration;
use tempfile::tempdir;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use vkteams_bot::bot::net::{
    RetryableMultipartForm, calculate_backoff_duration, should_retry_status,
    validate_file_path_async,
};

#[tokio::test]
async fn test_retryable_multipart_form_integration() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test_file.txt");
    let test_content = b"Hello, World!";

    // Create test file
    let mut file = File::create(&file_path).await.unwrap();
    file.write_all(test_content).await.unwrap();
    file.flush().await.unwrap();

    // Test creating retryable form from file
    let retryable_form =
        RetryableMultipartForm::from_file_path(file_path.to_string_lossy().to_string())
            .await
            .unwrap();

    assert_eq!(retryable_form.size(), test_content.len());
    assert_eq!(retryable_form.filename, "test_file.txt");

    // Test that we can create multiple forms (simulate retry)
    let _form1 = retryable_form.to_form();
    let _form2 = retryable_form.to_form();
    // Forms should be independently usable
}

#[tokio::test]
async fn test_async_file_validation_integration() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("valid_file.txt");

    // Create test file
    let mut file = File::create(&file_path).await.unwrap();
    file.write_all(b"test content").await.unwrap();

    // Should not return error for valid file
    let result = validate_file_path_async(&file_path.to_string_lossy()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_file_validation_security_integration() {
    // Test path traversal protection
    let result = validate_file_path_async("../../../etc/passwd").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("parent directory"));

    // Test nonexistent file
    let result = validate_file_path_async("/nonexistent/file.txt").await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("File does not exist")
    );
}

#[test]
fn test_retry_logic_integration() {
    let max_backoff = Duration::from_secs(10);

    // Test backoff calculation
    let backoff1 = calculate_backoff_duration(1, max_backoff);
    let backoff2 = calculate_backoff_duration(2, max_backoff);

    assert!(backoff1 <= max_backoff);
    assert!(backoff2 <= max_backoff);

    // Test retry status decisions
    assert!(should_retry_status(&StatusCode::INTERNAL_SERVER_ERROR));
    assert!(should_retry_status(&StatusCode::TOO_MANY_REQUESTS));
    assert!(!should_retry_status(&StatusCode::BAD_REQUEST));
    assert!(!should_retry_status(&StatusCode::OK));
}
