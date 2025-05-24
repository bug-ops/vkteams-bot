//! Path utilities for VK Teams Bot CLI
//!
//! This module provides path manipulation and file system utilities
//! used throughout the CLI application.

use crate::errors::prelude::{CliError, Result as CliResult};
use std::fs;
use std::path::{Path, PathBuf};

/// Ensure that a directory exists, creating it if necessary
///
/// # Arguments
/// * `path` - The directory path to ensure exists
///
/// # Returns
/// * `Ok(())` if the directory exists or was created successfully
/// * `Err(CliError::FileError)` if directory creation fails
pub fn ensure_directory_exists(path: &Path) -> CliResult<()> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| {
            CliError::FileError(format!(
                "Failed to create directory {}: {}",
                path.display(),
                e
            ))
        })?;
    } else if !path.is_dir() {
        return Err(CliError::FileError(format!(
            "Path exists but is not a directory: {}",
            path.display()
        )));
    }
    Ok(())
}

/// Get the file name from a path string
///
/// # Arguments
/// * `path` - The path string to extract the file name from
///
/// # Returns
/// * The file name as a string, or the full path if no file name can be extracted
pub fn get_file_name_from_path(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string()
}

/// Get the file extension from a path string
///
/// # Arguments
/// * `path` - The path string to extract the extension from
///
/// # Returns
/// * `Some(String)` containing the extension (without the dot)
/// * `None` if there is no extension
pub fn get_file_extension(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
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
    let blocked_extensions = ["exe", "bat", "cmd", "com", "scr", "pif", "msi"];
    !blocked_extensions.contains(&extension.to_lowercase().as_str())
}

/// Sanitize a file name by removing or replacing unsafe characters
///
/// # Arguments
/// * `filename` - The filename to sanitize
///
/// # Returns
/// * A sanitized filename safe for file system operations
pub fn sanitize_filename(filename: &str) -> String {
    let unsafe_chars = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];
    let mut sanitized = filename.to_string();

    for unsafe_char in &unsafe_chars {
        sanitized = sanitized.replace(*unsafe_char, "_");
    }

    // Remove leading/trailing dots and spaces
    sanitized = sanitized.trim_matches(|c| c == '.' || c == ' ').to_string();

    // Ensure the filename is not empty
    if sanitized.is_empty() {
        sanitized = "unnamed_file".to_string();
    }

    // Limit filename length to reasonable bounds
    if sanitized.len() > 255 {
        sanitized = sanitized[..255].to_string();
    }

    sanitized
}

/// Get the size of a file in bytes
///
/// # Arguments
/// * `path` - The path to the file
///
/// # Returns
/// * `Ok(u64)` containing the file size in bytes
/// * `Err(CliError::FileError)` if the file cannot be accessed
pub fn get_file_size(path: &Path) -> CliResult<u64> {
    let metadata = fs::metadata(path).map_err(|e| {
        CliError::FileError(format!(
            "Failed to get file metadata for {}: {}",
            path.display(),
            e
        ))
    })?;
    Ok(metadata.len())
}

/// Normalize a path string to use consistent separators
///
/// # Arguments
/// * `path` - The path string to normalize
///
/// # Returns
/// * A normalized path string
pub fn normalize_path(path: &str) -> String {
    Path::new(path).to_string_lossy().replace('\\', "/")
}

/// Get the parent directory of a path
///
/// # Arguments
/// * `path` - The path to get the parent of
///
/// # Returns
/// * `Some(PathBuf)` containing the parent directory
/// * `None` if there is no parent directory
pub fn get_parent_dir(path: &Path) -> Option<PathBuf> {
    path.parent().map(|p| p.to_path_buf())
}

/// Join multiple path components safely
///
/// # Arguments
/// * `base` - The base path
/// * `components` - Path components to join
///
/// # Returns
/// * A PathBuf with all components joined
pub fn join_path_components(base: &Path, components: &[&str]) -> PathBuf {
    let mut path = base.to_path_buf();
    for component in components {
        path.push(component);
    }
    path
}

/// Check if a file exists and is readable
///
/// # Arguments
/// * `path` - The path to check
///
/// # Returns
/// * `true` if the file exists and is readable
/// * `false` otherwise
pub fn is_file_readable(path: &Path) -> bool {
    path.exists() && path.is_file() && fs::metadata(path).is_ok()
}

/// Check if a directory exists and is writable
///
/// # Arguments
/// * `path` - The path to check
///
/// # Returns
/// * `true` if the directory exists and is writable
/// * `false` otherwise
pub fn is_directory_writable(path: &Path) -> bool {
    if !path.exists() || !path.is_dir() {
        return false;
    }

    // Try to create a temporary file to test writability
    let test_file = path.join(".write_test_temp");
    match fs::write(&test_file, b"test") {
        Ok(_) => {
            // Clean up the test file
            let _ = fs::remove_file(&test_file);
            true
        }
        Err(_) => false,
    }
}

/// Get a unique filename by appending a number if the file already exists
///
/// # Arguments
/// * `base_path` - The base file path
///
/// # Returns
/// * A unique file path that doesn't exist
pub fn get_unique_filename(base_path: &Path) -> PathBuf {
    if !base_path.exists() {
        return base_path.to_path_buf();
    }

    let parent = base_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = base_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let extension = base_path.extension().and_then(|s| s.to_str()).unwrap_or("");

    for i in 1..1000 {
        let new_filename = if extension.is_empty() {
            format!("{}_{}", stem, i)
        } else {
            format!("{}_{}.{}", stem, i, extension)
        };

        let new_path = parent.join(new_filename);
        if !new_path.exists() {
            return new_path;
        }
    }

    // Fallback if we can't find a unique name
    base_path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_get_file_name_from_path() {
        assert_eq!(get_file_name_from_path("/path/to/file.txt"), "file.txt");
        assert_eq!(get_file_name_from_path("file.txt"), "file.txt");
        assert_eq!(get_file_name_from_path("/path/to/"), "to");
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension("file.txt"), Some("txt".to_string()));
        assert_eq!(get_file_extension("file.TAR.GZ"), Some("gz".to_string()));
        assert_eq!(get_file_extension("file"), None);
        assert_eq!(get_file_extension(".hidden"), None);
    }

    #[test]
    fn test_is_supported_voice_format() {
        assert!(is_supported_voice_format("ogg"));
        assert!(is_supported_voice_format("mp3"));
        assert!(is_supported_voice_format("OGG"));
        assert!(!is_supported_voice_format("txt"));
        assert!(!is_supported_voice_format("exe"));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("normal_file.txt"), "normal_file.txt");
        assert_eq!(sanitize_filename("file<>name.txt"), "file__name.txt");
        assert_eq!(sanitize_filename("   file.txt   "), "file.txt");
        assert_eq!(sanitize_filename(""), "unnamed_file");
    }

    #[test]
    fn test_validate_safe_path() {
        use crate::utils::validation::validate_safe_path;
        assert!(validate_safe_path("safe/path/file.txt").is_ok());
        assert!(validate_safe_path("../unsafe/path").is_err());
        assert!(validate_safe_path("~/home/path").is_err());
        assert!(validate_safe_path("path/../file").is_err());
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("path\\to\\file"), "path/to/file");
        assert_eq!(normalize_path("path/to/file"), "path/to/file");
    }

    #[test]
    fn test_ensure_directory_exists() {
        let temp_dir = tempdir().unwrap();
        let new_dir = temp_dir.path().join("new_directory");

        assert!(ensure_directory_exists(&new_dir).is_ok());
        assert!(new_dir.exists() && new_dir.is_dir());
    }

    #[test]
    fn test_get_unique_filename() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // First call should return the original path
        let unique1 = get_unique_filename(&file_path);
        assert_eq!(unique1, file_path);

        // Create the file and test again
        fs::write(&file_path, "test").unwrap();
        let unique2 = get_unique_filename(&file_path);
        assert_ne!(unique2, file_path);
        assert!(!unique2.exists());
    }
}
