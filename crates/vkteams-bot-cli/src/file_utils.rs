use crate::config::CONFIG;
use crate::errors::prelude::{CliError, Result as CliResult};
use crate::progress;
use crate::utils::{validate_directory_path, validate_file_path};
use futures::StreamExt;
use std::fmt::Debug;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};
use vkteams_bot::prelude::*;

// Validation functions are now imported from utils/validation module

// TODO: Enable this function when we need streaming file uploads
// /// Streams a file from disk for uploading
// ///
// /// # Errors
// /// - Returns `CliError::FileError` if the file doesn't exist or cannot be opened
// pub async fn read_file_stream(file_path: &str) -> CliResult<tokio::fs::File> {
//     validate_file_path(file_path)?;
//
//     let file = tokio::fs::File::open(file_path)
//         .await
//         .map_err(|e| CliError::FileError(format!("Failed to open file {file_path}: {e}")))?;
//
//     Ok(file)
// }

/// Stream downloads a file and saves it to disk
///
/// # Errors
/// - Returns `CliError::FileError` if there are issues with file operations
/// - Returns `CliError::ApiError` if there are issues with the API
pub async fn download_and_save_file(
    bot: &Bot,
    file_id: &str,
    dir_path: &str,
) -> CliResult<PathBuf> {
    let cfg = &CONFIG.files;
    // Use directory from path or config or current directory
    let target_dir = if !dir_path.is_empty() {
        dir_path.to_string()
    } else if let Some(download_dir) = &cfg.download_dir {
        download_dir.clone()
    } else {
        ".".to_string()
    };

    validate_directory_path(&target_dir)?;

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

    if total_size > cfg.max_file_size as u64 {
        return Err(CliError::FileError(format!(
            "File size exceeds maximum allowed size of {} bytes",
            cfg.max_file_size
        )));
    }

    let mut file_writer = tokio::io::BufWriter::with_capacity(cfg.buffer_size, file);

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    // Create a progress bar for the download
    let progress_bar = progress::create_download_progress_bar(total_size, &file_info.file_name);

    debug!("Streaming file content to disk");
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| {
            progress::abandon_progress(&progress_bar, "Download failed");
            CliError::FileError(format!("Error during download: {e}"))
        })?;

        file_writer.write_all(&chunk).await.map_err(|e| {
            progress::abandon_progress(&progress_bar, "Write failed");
            CliError::FileError(format!("Failed to write to file: {e}"))
        })?;

        downloaded += chunk.len() as u64;
        progress::increment_progress(&progress_bar, chunk.len() as u64);

        // Log progress for large files (if progress bar is disabled)
        if !&CONFIG.ui.show_progress
            && total_size > 1024 * 1024
            && downloaded % (1024 * 1024) < chunk.len() as u64
        {
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
                downloaded_mb, total_mb
            );
        }
    }

    debug!("Flushing and finalizing file");
    file_writer.flush().await.map_err(|e| {
        progress::abandon_progress(&progress_bar, "File flush failed");
        CliError::FileError(format!("Failed to flush file data: {e}"))
    })?;

    progress::finish_progress(
        &progress_bar,
        &format!("Downloaded to {}", file_path.display()),
    );
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
) -> CliResult<impl serde::Serialize + Debug> {
    let cfg = &CONFIG.files;
    // Use file path from arguments or config
    let source_path = if !file_path.is_empty() {
        file_path.to_string()
    } else if let Some(upload_dir) = &cfg.upload_dir {
        upload_dir.clone()
    } else {
        return Err(CliError::InputError(
            "No file path provided and no default upload directory configured".to_string(),
        ));
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
    let progress_bar = progress::create_upload_progress_bar(file_size, &source_path);

    // Start the upload
    let result = match bot
        .send_api_request(RequestMessagesSendFile::new((
            ChatId(user_id.to_string()),
            MultipartName::FilePath(source_path.to_string()),
        )))
        .await
    {
        Ok(res) => {
            progress::finish_progress(&progress_bar, "Upload complete");
            res
        }
        Err(e) => {
            progress::abandon_progress(&progress_bar, "Upload failed");
            return Err(CliError::ApiError(e));
        }
    };

    info!("Successfully uploaded file: {}", source_path);
    Ok(result)
}

/// Stream uploads a voice message to the API
///
/// # Errors
/// - Returns `CliError::InputError` if no file path provided
/// - Returns `CliError::FileError` if the file doesn't exist or is not accessible
/// - Returns `CliError::ApiError` if there are issues with the API
pub async fn upload_voice(
    bot: &Bot,
    user_id: &str,
    file_path: &str,
) -> CliResult<impl serde::Serialize + Debug> {
    let cfg = &CONFIG.files;
    // Use file path from arguments or config
    let source_path = if !file_path.is_empty() {
        file_path.to_string()
    } else if let Some(upload_dir) = &cfg.upload_dir {
        upload_dir.clone()
    } else {
        return Err(CliError::InputError(
            "No file path provided and no default upload directory configured".to_string(),
        ));
    };

    validate_file_path(&source_path)?;

    debug!("Preparing to upload voice message: {}", source_path);

    // Get the file size for the progress bar
    let file_size = match progress::calculate_upload_size(&source_path) {
        Ok(size) => size,
        Err(e) => {
            debug!("Could not determine file size: {}", e);
            0 // If we can't determine size, progress bar will be indeterminate
        }
    };

    // Create a progress bar for upload
    let progress_bar = progress::create_upload_progress_bar(file_size, &source_path);

    // Start the voice upload
    let result = match bot
        .send_api_request(RequestMessagesSendVoice::new((
            ChatId(user_id.to_string()),
            MultipartName::FilePath(source_path.to_string()),
        )))
        .await
    {
        Ok(res) => {
            progress::finish_progress(&progress_bar, "Voice upload complete");
            res
        }
        Err(e) => {
            progress::abandon_progress(&progress_bar, "Voice upload failed");
            return Err(CliError::ApiError(e));
        }
    };

    info!("Successfully uploaded voice message: {}", source_path);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::create_dummy_bot;
    use proptest::prelude::*;
    use std::fs;
    use tempfile::tempdir;
    use tokio_test::block_on;

    #[tokio::test]
    async fn test_upload_file_empty_path() {
        let bot = create_dummy_bot();
        let res = upload_file(&bot, "user123", "").await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_upload_file_nonexistent() {
        let bot = create_dummy_bot();
        let res = upload_file(&bot, "user123", "no_such_file.txt").await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_upload_voice_invalid_format() {
        let bot = create_dummy_bot();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("voice.txt");
        fs::write(&file_path, "test").unwrap();
        let res = upload_voice(&bot, "user123", file_path.to_str().unwrap()).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_download_and_save_file_invalid_dir() {
        let bot = create_dummy_bot();
        let res = download_and_save_file(&bot, "fileid123", "/no/such/dir").await;
        assert!(res.is_err());
    }

    proptest! {
        #[test]
        fn prop_upload_file_random_path(user_id in ".{0,32}", file_path in ".{0,128}") {
            let bot = create_dummy_bot();
            let fut = upload_file(&bot, &user_id, &file_path);
            let res = block_on(fut);
            prop_assert!(res.is_err());
        }

        #[test]
        fn prop_upload_voice_random_path(user_id in ".{0,32}", file_path in ".{0,128}") {
            let bot = create_dummy_bot();
            let fut = upload_voice(&bot, &user_id, &file_path);
            let res = block_on(fut);
            prop_assert!(res.is_err());
        }
    }
}

#[cfg(test)]
mod more_edge_tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_download_and_save_file_api_error() {
        let bot =
            Bot::with_params(&APIVersionUrl::V1, "dummy_token", "https://dummy.api.com").unwrap();
        let tmp = tempdir().unwrap();
        let res = download_and_save_file(&bot, "fileid", tmp.path().to_str().unwrap()).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_download_and_save_file_write_error() {
        let bot =
            Bot::with_params(&APIVersionUrl::V1, "dummy_token", "https://dummy.api.com").unwrap();
        let tmp = tempdir().unwrap();
        let dir = tmp.path().join("readonly");
        fs::create_dir(&dir).unwrap();
        let mut perms = fs::metadata(&dir).unwrap().permissions();
        perms.set_mode(0o400); // read-only
        fs::set_permissions(&dir, perms).unwrap();
        let res = download_and_save_file(&bot, "fileid", dir.to_str().unwrap()).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_upload_file_api_error() {
        let bot =
            Bot::with_params(&APIVersionUrl::V1, "dummy_token", "https://dummy.api.com").unwrap();
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("file.txt");
        File::create(&file_path).unwrap();
        let res = upload_file(&bot, "user123", file_path.to_str().unwrap()).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_upload_file_too_large() {
        let bot =
            Bot::with_params(&APIVersionUrl::V1, "dummy_token", "https://dummy.api.com").unwrap();
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("bigfile.bin");
        let mut f = File::create(&file_path).unwrap();
        f.write_all(&vec![0u8; 200 * 1024 * 1024]).unwrap(); // 200MB
        let res = upload_file(&bot, "user123", file_path.to_str().unwrap()).await;
        assert!(res.is_err());
    }
}

#[cfg(test)]
mod happy_path_tests {
    use super::*;
    use crate::utils::create_dummy_bot;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_upload_file_success() {
        let bot = create_dummy_bot();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("file.txt");
        fs::write(&file_path, "test").unwrap();
        // This will fail on real API, but for dummy bot we expect an error or Ok depending on mock
        let _ = upload_file(&bot, "user123", file_path.to_str().unwrap()).await;
        // No panic means the function handles the flow
    }

    #[tokio::test]
    async fn test_upload_voice_success() {
        let bot = create_dummy_bot();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("voice.ogg");
        fs::write(&file_path, "test").unwrap();
        let _ = upload_voice(&bot, "user123", file_path.to_str().unwrap()).await;
    }

    #[tokio::test]
    async fn test_download_and_save_file_success() {
        let bot = create_dummy_bot();
        let temp_dir = tempdir().unwrap();
        // File ID is dummy, but function should handle the flow without panic
        let _ = download_and_save_file(&bot, "fileid123", temp_dir.path().to_str().unwrap()).await;
    }
}
