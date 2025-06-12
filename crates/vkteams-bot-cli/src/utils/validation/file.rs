//! File-related validation functions
//!
//! This module contains validation functions for file paths, file IDs,
//! and file-related operations.

use super::validate_not_empty;
use crate::errors::prelude::{CliError, Result as CliResult};
use std::path::Path;

/// Validate a file path exists and is accessible
///
/// # Arguments
/// * `file_path` - The file path to validate
///
/// # Returns
/// * `Ok(())` if the file path is valid
/// * `Err(CliError::FileError)` if the file doesn't exist or is not accessible
pub fn validate_file_path(file_path: &str) -> CliResult<()> {
    validate_not_empty(file_path, "File path")?;

    let path = Path::new(file_path);
    if !path.exists() {
        return Err(CliError::FileError(format!(
            "File not found: {}",
            file_path
        )));
    }
    if !path.is_file() {
        return Err(CliError::FileError(format!(
            "Path is not a file: {}",
            file_path
        )));
    }

    Ok(())
}

/// Validate a directory path exists and is accessible
///
/// # Arguments
/// * `dir_path` - The directory path to validate
///
/// # Returns
/// * `Ok(())` if the directory path is valid
/// * `Err(CliError::FileError)` if the directory doesn't exist or is not accessible
pub fn validate_directory_path(dir_path: &str) -> CliResult<()> {
    if dir_path.trim().is_empty() {
        return Ok(()); // Empty path is allowed, will use default
    }

    let path = Path::new(dir_path);
    if path.exists() && !path.is_dir() {
        return Err(CliError::FileError(format!(
            "Path exists but is not a directory: {}",
            dir_path
        )));
    }

    Ok(())
}

/// Validate a file ID format
///
/// # Arguments
/// * `file_id` - The file ID to validate
///
/// # Returns
/// * `Ok(())` if the file ID is valid
/// * `Err(CliError::InputError)` if the file ID is invalid
pub fn validate_file_id(file_id: &str) -> CliResult<()> {
    validate_not_empty(file_id, "File ID")?;

    // Basic file ID format validation
    if !file_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(CliError::InputError(
            "File ID contains invalid characters. Only alphanumeric, underscore, and hyphen are allowed".to_string()
        ));
    }

    Ok(())
}

/// Validate a voice file path and format
///
/// # Arguments
/// * `file_path` - The voice file path to validate
///
/// # Returns
/// * `Ok(())` if the voice file is valid
/// * `Err(CliError::InputError)` if the voice file is invalid
pub fn validate_voice_file_path(file_path: &str) -> CliResult<()> {
    validate_file_path(file_path)?;

    // Additional validation for voice files
    let path = Path::new(file_path);
    if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        match ext.as_str() {
            "ogg" | "mp3" | "wav" | "m4a" | "aac" => Ok(()),
            _ => Err(CliError::InputError(format!(
                "Unsupported voice file format: {}. Supported formats: ogg, mp3, wav, m4a, aac",
                ext
            ))),
        }
    } else {
        Err(CliError::InputError(
            "Voice file must have an extension".to_string(),
        ))
    }
}

/// Validate file size is within limits
///
/// # Arguments
/// * `file_path` - The file path to check
/// * `max_size` - Maximum allowed file size in bytes
///
/// # Returns
/// * `Ok(())` if the file size is within limits
/// * `Err(CliError::FileError)` if the file is too large
pub fn validate_file_size(file_path: &str, max_size: usize) -> CliResult<()> {
    let path = Path::new(file_path);
    if let Ok(metadata) = path.metadata() {
        let size = metadata.len() as usize;
        if size > max_size {
            return Err(CliError::FileError(format!(
                "File size ({} bytes) exceeds maximum allowed size ({} bytes)",
                size, max_size
            )));
        }
    }
    Ok(())
}

/// Check if a file extension is supported for voice messages
///
/// # Arguments
/// * `extension` - The file extension to check (without the dot)
///
/// # Returns
/// * `true` if the extension is supported for voice messages
/// * `false` otherwise
pub fn is_supported_voice_format(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "ogg" | "mp3" | "wav" | "m4a" | "aac"
    )
}

/// Check if a file extension is supported for regular file uploads
///
/// # Arguments
/// * `extension` - The file extension to check (without the dot)
///
/// # Returns
/// * `true` if the extension is supported for file uploads
/// * `false` if the extension is blocked or restricted
pub fn is_supported_file_format(extension: &str) -> bool {
    // Most file types are supported, but we can block dangerous ones
    let blocked_extensions = ["exe", "bat", "cmd", "com", "scr", "pif"];
    !blocked_extensions.contains(&extension.to_lowercase().as_str())
}

/// Validate that a path is safe for file operations (no path traversal)
///
/// # Arguments
/// * `path` - The path to validate
///
/// # Returns
/// * `Ok(())` if the path is safe
/// * `Err(CliError::InputError)` if the path contains unsafe elements
pub fn validate_safe_path(path: &str) -> CliResult<()> {
    if path.contains("..") || path.contains("~") {
        return Err(CliError::InputError(
            "Path contains unsafe elements (.. or ~)".to_string(),
        ));
    }

    // Check for absolute paths on Windows that might be problematic
    #[cfg(windows)]
    {
        if path.len() >= 2 && path.chars().nth(1) == Some(':') {
            // Allow C:, D:, etc. but validate they're reasonable
            let drive = path.chars().next().unwrap().to_ascii_uppercase();
            if !('A'..='Z').contains(&drive) {
                return Err(CliError::InputError(
                    "Invalid drive letter in path".to_string(),
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_validate_file_id() {
        assert!(validate_file_id("file123").is_ok());
        assert!(validate_file_id("file_123").is_ok());
        assert!(validate_file_id("file-123").is_ok());
        assert!(validate_file_id("").is_err());
        assert!(validate_file_id("file@123").is_err());
        assert!(validate_file_id("file.123").is_err());
    }

    #[test]
    fn test_is_supported_voice_format() {
        assert!(is_supported_voice_format("ogg"));
        assert!(is_supported_voice_format("mp3"));
        assert!(is_supported_voice_format("wav"));
        assert!(is_supported_voice_format("OGG"));
        assert!(!is_supported_voice_format("txt"));
        assert!(!is_supported_voice_format("exe"));
    }

    #[test]
    fn test_is_supported_file_format() {
        assert!(is_supported_file_format("txt"));
        assert!(is_supported_file_format("pdf"));
        assert!(is_supported_file_format("jpg"));
        assert!(!is_supported_file_format("exe"));
        assert!(!is_supported_file_format("bat"));
    }

    #[test]
    fn test_validate_safe_path() {
        assert!(validate_safe_path("file.txt").is_ok());
        assert!(validate_safe_path("folder/file.txt").is_ok());
        assert!(validate_safe_path("../file.txt").is_err());
        assert!(validate_safe_path("~/file.txt").is_err());
        assert!(validate_safe_path("folder/../file.txt").is_err());
    }

    #[test]
    fn test_validate_file_path_with_real_file() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        assert!(validate_file_path(file_path.to_str().unwrap()).is_ok());
        assert!(validate_file_path("nonexistent_file.txt").is_err());
    }

    #[test]
    fn test_validate_directory_path_empty() {
        assert!(validate_directory_path("").is_ok());
        assert!(validate_directory_path("   ").is_ok());
    }
}

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_validate_file_id_random(s in ".{0,256}") {
            let _ = validate_file_id(&s);
        }

        #[test]
        fn prop_is_supported_voice_format_random(s in ".{0,32}") {
            let _ = is_supported_voice_format(&s);
        }

        #[test]
        fn prop_is_supported_file_format_random(s in ".{0,32}") {
            let _ = is_supported_file_format(&s);
        }

        #[test]
        fn prop_validate_safe_path_random(s in ".{0,256}") {
            let _ = validate_safe_path(&s);
        }
    }
}
