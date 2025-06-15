//! File utilities for VKTeams Bot MCP server
//!
//! This module provides helper functions for file handling, validation,
//! and content processing.

use crate::errors::McpError;
use std::collections::HashMap;

/// Maximum file size in bytes (10MB)
pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Supported file extensions and their MIME types
pub fn get_mime_types() -> HashMap<&'static str, &'static str> {
    let mut mime_types = HashMap::new();

    // Text files
    mime_types.insert("txt", "text/plain");
    mime_types.insert("md", "text/markdown");
    mime_types.insert("json", "application/json");
    mime_types.insert("xml", "application/xml");
    mime_types.insert("csv", "text/csv");
    mime_types.insert("log", "text/plain");

    // Code files
    mime_types.insert("rs", "text/x-rust");
    mime_types.insert("py", "text/x-python");
    mime_types.insert("js", "application/javascript");
    mime_types.insert("ts", "application/typescript");
    mime_types.insert("html", "text/html");
    mime_types.insert("css", "text/css");
    mime_types.insert("sql", "application/sql");
    mime_types.insert("sh", "application/x-sh");
    mime_types.insert("yml", "application/x-yaml");
    mime_types.insert("yaml", "application/x-yaml");
    mime_types.insert("toml", "application/toml");

    // Images
    mime_types.insert("png", "image/png");
    mime_types.insert("jpg", "image/jpeg");
    mime_types.insert("jpeg", "image/jpeg");
    mime_types.insert("gif", "image/gif");
    mime_types.insert("svg", "image/svg+xml");
    mime_types.insert("webp", "image/webp");

    // Documents
    mime_types.insert("pdf", "application/pdf");
    mime_types.insert("doc", "application/msword");
    mime_types.insert(
        "docx",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    );
    mime_types.insert("xls", "application/vnd.ms-excel");
    mime_types.insert(
        "xlsx",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    );

    // Archives
    mime_types.insert("zip", "application/zip");
    mime_types.insert("tar", "application/x-tar");
    mime_types.insert("gz", "application/gzip");
    mime_types.insert("7z", "application/x-7z-compressed");

    // Audio/Video
    mime_types.insert("mp3", "audio/mpeg");
    mime_types.insert("wav", "audio/wav");
    mime_types.insert("ogg", "audio/ogg");
    mime_types.insert("mp4", "video/mp4");
    mime_types.insert("avi", "video/x-msvideo");
    mime_types.insert("mov", "video/quicktime");

    mime_types
}

/// Extract file extension from filename
pub fn get_file_extension(filename: &str) -> Option<&str> {
    filename
        .rfind('.')
        .map(|i| &filename[i + 1..])
        .filter(|ext| !ext.is_empty())
}

/// Get MIME type for a file based on its extension
pub fn get_mime_type(filename: &str) -> &'static str {
    let mime_types = get_mime_types();

    if let Some(extension) = get_file_extension(filename) {
        mime_types
            .get(extension.to_lowercase().as_str())
            .unwrap_or(&"application/octet-stream")
    } else {
        "application/octet-stream"
    }
}

/// Validate file size
pub fn validate_file_size(content: &[u8]) -> Result<(), McpError> {
    if content.len() > MAX_FILE_SIZE {
        return Err(McpError::Other(format!(
            "File size {} bytes exceeds maximum allowed size {} bytes",
            content.len(),
            MAX_FILE_SIZE
        )));
    }
    Ok(())
}

/// Validate filename for safety
pub fn validate_filename(filename: &str) -> Result<(), McpError> {
    if filename.is_empty() {
        return Err(McpError::Other("Filename cannot be empty".to_string()));
    }

    if filename.len() > 255 {
        return Err(McpError::Other(
            "Filename too long (max 255 characters)".to_string(),
        ));
    }

    // Check for invalid characters
    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];
    if filename.chars().any(|c| invalid_chars.contains(&c)) {
        return Err(McpError::Other(
            "Filename contains invalid characters".to_string(),
        ));
    }

    // Check for reserved names (Windows)
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    let name_without_ext = filename.split('.').next().unwrap_or("");
    if reserved_names
        .iter()
        .any(|&reserved| reserved.eq_ignore_ascii_case(name_without_ext))
    {
        return Err(McpError::Other("Filename uses reserved name".to_string()));
    }

    Ok(())
}

/// Check if file is text-based
pub fn is_text_file(filename: &str) -> bool {
    let text_extensions = [
        "txt",
        "md",
        "json",
        "xml",
        "csv",
        "log",
        "rs",
        "py",
        "js",
        "ts",
        "html",
        "css",
        "sql",
        "sh",
        "yml",
        "yaml",
        "toml",
        "ini",
        "cfg",
        "conf",
        "properties",
        "gitignore",
        "dockerfile",
        "makefile",
    ];

    if let Some(ext) = get_file_extension(filename) {
        text_extensions
            .iter()
            .any(|&text_ext| text_ext.eq_ignore_ascii_case(ext))
    } else {
        false
    }
}

/// Check if file is an image
pub fn is_image_file(filename: &str) -> bool {
    let image_extensions = ["png", "jpg", "jpeg", "gif", "svg", "webp", "bmp", "ico"];

    if let Some(ext) = get_file_extension(filename) {
        image_extensions
            .iter()
            .any(|&img_ext| img_ext.eq_ignore_ascii_case(ext))
    } else {
        false
    }
}

/// Check if file is an audio file
pub fn is_audio_file(filename: &str) -> bool {
    let audio_extensions = ["mp3", "wav", "ogg", "m4a", "aac", "flac", "wma"];

    if let Some(ext) = get_file_extension(filename) {
        audio_extensions
            .iter()
            .any(|&audio_ext| audio_ext.eq_ignore_ascii_case(ext))
    } else {
        false
    }
}

/// Format file size in human-readable format
pub fn format_file_size(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Sanitize filename by replacing invalid characters
pub fn sanitize_filename(filename: &str) -> String {
    let mut sanitized = filename.to_string();

    // Replace invalid characters with underscores
    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];
    for &invalid_char in &invalid_chars {
        sanitized = sanitized.replace(invalid_char, "_");
    }

    // Trim whitespace and dots from start/end
    sanitized = sanitized.trim().trim_matches('.').to_string();

    // Ensure it's not empty
    if sanitized.is_empty() {
        sanitized = "file".to_string();
    }

    // Truncate if too long
    if sanitized.len() > 255 {
        if let Some(ext_pos) = sanitized.rfind('.') {
            let extension = &sanitized[ext_pos..];
            let name_part = &sanitized[..ext_pos];
            let max_name_len = 255 - extension.len();
            sanitized = format!(
                "{}{}",
                &name_part[..max_name_len.min(name_part.len())],
                extension
            );
        } else {
            sanitized.truncate(255);
        }
    }

    sanitized
}

/// Detect content type from file content (magic bytes)
pub fn detect_content_type(content: &[u8]) -> &'static str {
    if content.is_empty() {
        return "application/octet-stream";
    }

    // Check magic bytes for common file types
    match content {
        // PNG
        [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, ..] => "image/png",
        // JPEG
        [0xFF, 0xD8, 0xFF, ..] => "image/jpeg",
        // GIF
        [0x47, 0x49, 0x46, 0x38, ..] => "image/gif",
        // PDF
        [0x25, 0x50, 0x44, 0x46, ..] => "application/pdf",
        // ZIP
        [0x50, 0x4B, 0x03, 0x04, ..]
        | [0x50, 0x4B, 0x05, 0x06, ..]
        | [0x50, 0x4B, 0x07, 0x08, ..] => "application/zip",
        // MP3
        [0xFF, 0xFB, ..] | [0xFF, 0xF3, ..] | [0xFF, 0xF2, ..] => "audio/mpeg",
        // Check if it's likely text
        _ => {
            if is_likely_text(content) {
                "text/plain"
            } else {
                "application/octet-stream"
            }
        }
    }
}

/// Check if content is likely text (UTF-8)
pub fn is_likely_text(content: &[u8]) -> bool {
    // Try to decode as UTF-8
    if let Ok(text) = std::str::from_utf8(content) {
        // Check if it contains mostly printable characters
        let printable_count = text
            .chars()
            .filter(|&c| c.is_ascii_graphic() || c.is_ascii_whitespace())
            .count();
        let total_chars = text.chars().count();

        if total_chars == 0 {
            return false;
        }

        // If more than 90% are printable, consider it text
        (printable_count as f64 / total_chars as f64) > 0.9
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension("test.txt"), Some("txt"));
        assert_eq!(get_file_extension("test.tar.gz"), Some("gz"));
        assert_eq!(get_file_extension("test"), None);
        assert_eq!(get_file_extension("test."), None);
        assert_eq!(get_file_extension(".hidden"), Some("hidden"));
    }

    #[test]
    fn test_get_mime_type() {
        assert_eq!(get_mime_type("test.txt"), "text/plain");
        assert_eq!(get_mime_type("test.json"), "application/json");
        assert_eq!(get_mime_type("test.unknown"), "application/octet-stream");
        assert_eq!(get_mime_type("test"), "application/octet-stream");
    }

    #[test]
    fn test_validate_filename() {
        assert!(validate_filename("test.txt").is_ok());
        assert!(validate_filename("").is_err());
        assert!(validate_filename("test/file.txt").is_err());
        assert!(validate_filename("CON.txt").is_err());
        assert!(validate_filename(&"a".repeat(256)).is_err());
    }

    #[test]
    fn test_is_text_file() {
        assert!(is_text_file("test.txt"));
        assert!(is_text_file("test.rs"));
        assert!(is_text_file("test.json"));
        assert!(!is_text_file("test.jpg"));
        assert!(!is_text_file("test.mp3"));
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(100), "100 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test/file.txt"), "test_file.txt");
        assert_eq!(sanitize_filename("test:file.txt"), "test_file.txt");
        assert_eq!(sanitize_filename(""), "file");
        assert_eq!(sanitize_filename("..."), "file");
    }

    #[test]
    fn test_detect_content_type() {
        // PNG magic bytes
        let png_bytes = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(detect_content_type(&png_bytes), "image/png");

        // Text content
        let text_bytes = b"Hello, world!";
        assert_eq!(detect_content_type(text_bytes), "text/plain");

        // Empty content
        assert_eq!(detect_content_type(&[]), "application/octet-stream");
    }

    #[test]
    fn test_validate_file_size() {
        let small_content = vec![0u8; 100];
        assert!(validate_file_size(&small_content).is_ok());

        let large_content = vec![0u8; MAX_FILE_SIZE + 1];
        assert!(validate_file_size(&large_content).is_err());
    }
}
