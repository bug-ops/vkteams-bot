use crate::errors::prelude::{CliError, Result as CliResult};
use crate::config::Config;
use crate::progress;
use futures::StreamExt;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};
use vkteams_bot::prelude::*;

/// Validates a file path exists
///
/// # Errors
/// - Returns `CliError::FileError` if file doesn't exist or is not a regular file
pub fn validate_file_path(file_path: &str) -> CliResult<()> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(CliError::FileError(format!(
            "File not found: {file_path}"
        )));
    }

    if !path.is_file() {
        return Err(CliError::FileError(format!(
            "Path is not a file: {file_path}"
        )));
    }

    Ok(())
}

/// Validates a directory path exists
///
/// # Errors
/// - Returns `CliError::FileError` if directory doesn't exist or is not a directory
pub fn validate_directory(dir_path: &str) -> CliResult<()> {
    let path = Path::new(dir_path);
    if !path.exists() {
        return Err(CliError::FileError(format!(
            "Directory not found: {dir_path}"
        )));
    }

    if !path.is_dir() {
        return Err(CliError::FileError(format!(
            "Path is not a directory: {dir_path}"
        )));
    }

    Ok(())
}

/// Streams a file from disk for uploading
///
/// # Errors
/// - Returns `CliError::FileError` if the file doesn't exist or cannot be opened
pub async fn read_file_stream(file_path: &str) -> CliResult<tokio::fs::File> {
    validate_file_path(file_path)?;
    
    let file = tokio::fs::File::open(file_path)
        .await
        .map_err(|e| CliError::FileError(format!("Failed to open file {file_path}: {e}")))?;
    
    Ok(file)
}

/// Stream downloads a file and saves it to disk
///
/// # Errors
/// - Returns `CliError::FileError` if there are issues with file operations
/// - Returns `CliError::ApiError` if there are issues with the API
pub async fn download_and_save_file(
    bot: &Bot,
    file_id: &str,
    dir_path: &str,
    config: &Config,
) -> CliResult<PathBuf> {
    // Use directory from path or config or current directory
    let target_dir = if !dir_path.is_empty() {
        dir_path.to_string()
    } else if let Some(download_dir) = &config.files.download_dir {
        download_dir.clone()
    } else {
        ".".to_string()
    };
    
    validate_directory(&target_dir)?;

    debug!("Getting file info for file ID: {}", file_id);
    let file_info = bot
        .send_api_request(RequestFilesGetInfo::new(FileId(file_id.to_string())))
        .await
        .map_err(CliError::ApiError)?;

    let mut file_path = PathBuf::from(&target_dir);
    file_path.push(&file_info.file_name);

    debug!("Creating file at path: {}", file_path.display());
    let file = tokio::fs::File::create(&file_path).await.map_err(|e| {
        CliError::FileError(format!(
            "Failed to create file {}: {}",
            file_path.display(),
            e
        ))
    })?;

    debug!("Starting file download stream");
    let client = reqwest::Client::new();
    let url = file_info.url.clone();

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| CliError::FileError(format!("Failed to initiate download: {e}")))?;

    if !response.status().is_success() {
        return Err(CliError::FileError(format!(
            "Failed to download file, status code: {}",
            response.status()
        )));
    }

    let total_size = response.content_length().unwrap_or(0);

    if total_size > config.files.max_file_size as u64 {
        return Err(CliError::FileError(format!(
            "File size exceeds maximum allowed size of {} bytes",
            config.files.max_file_size
        )));
    }

    let mut file_writer = tokio::io::BufWriter::with_capacity(config.files.buffer_size, file);

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    // Create a progress bar for the download
    let progress_bar = progress::create_download_progress_bar(
        config, 
        total_size, 
        &file_info.file_name
    );

    debug!("Streaming file content to disk");
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result
            .map_err(|e| {
                progress::abandon_progress(&progress_bar, "Download failed");
                CliError::FileError(format!("Error during download: {e}"))
            })?;

        file_writer
            .write_all(&chunk)
            .await
            .map_err(|e| {
                progress::abandon_progress(&progress_bar, "Write failed");
                CliError::FileError(format!("Failed to write to file: {e}"))
            })?;

        downloaded += chunk.len() as u64;
        progress::increment_progress(&progress_bar, chunk.len() as u64);

        // Log progress for large files (if progress bar is disabled)
        if !config.ui.show_progress && total_size > 1024 * 1024 && downloaded % (1024 * 1024) < chunk.len() as u64 {
            let downloaded_mb = {
                #[allow(clippy::cast_precision_loss)]
                let val = (downloaded / 1_048_576) as f64;
                val
            };
            let total_mb = {
                #[allow(clippy::cast_precision_loss)]
                let val = (total_size / 1_048_576) as f64;
                val
            };
            info!(
                "Download progress: {:.1}MB / {:.1}MB",
                downloaded_mb,
                total_mb
            );
        }
    }

    debug!("Flushing and finalizing file");
    file_writer
        .flush()
        .await
        .map_err(|e| {
            progress::abandon_progress(&progress_bar, "File flush failed");
            CliError::FileError(format!("Failed to flush file data: {e}"))
        })?;

    progress::finish_progress(&progress_bar, &format!("Downloaded to {}", file_path.display()));
    info!("Successfully downloaded file to: {}", file_path.display());
    Ok(file_path)
}

/// Stream uploads a file to the API
///
/// # Errors
/// - Returns `CliError::InputError` if no file path provided
/// - Returns `CliError::FileError` if the file doesn't exist or is not accessible
/// - Returns `CliError::ApiError` if there are issues with the API
pub async fn upload_file(
    bot: &Bot,
    user_id: &str,
    file_path: &str,
    config: &Config,
) -> CliResult<impl serde::Serialize + Debug> {
    // Use file path from arguments or config
    let source_path = if !file_path.is_empty() {
        file_path.to_string()
    } else if let Some(upload_dir) = &config.files.upload_dir {
        upload_dir.clone()
    } else {
        return Err(CliError::InputError("No file path provided and no default upload directory configured".to_string()));
    };
    
    validate_file_path(&source_path)?;

    debug!("Preparing to upload file: {}", source_path);
    
    // Get the file size for the progress bar
    let file_size = match progress::calculate_upload_size(&source_path) {
        Ok(size) => size,
        Err(e) => {
            debug!("Could not determine file size: {}", e);
            0 // If we can't determine size, progress bar will be indeterminate
        }
    };
    
    // Create a progress bar for upload
    let progress_bar = progress::create_upload_progress_bar(config, file_size, &source_path);
    
    // Start the upload
    let result = match bot
        .send_api_request(RequestMessagesSendFile::new((
            ChatId(user_id.to_string()),
            MultipartName::File(source_path.to_string()),
        )))
        .await {
            Ok(res) => {
                progress::finish_progress(&progress_bar, "Upload complete");
                res
            },
            Err(e) => {
                progress::abandon_progress(&progress_bar, "Upload failed");
                return Err(CliError::ApiError(e));
            }
        };

    info!("Successfully uploaded file: {}", source_path);
    Ok(result)
}
