use std::fmt;
use vkteams_bot::error::BotError;

// Static error message prefixes to avoid repeated allocations
pub static FILE_NOT_FOUND: &str = "File not found: ";
pub static DIR_NOT_FOUND: &str = "Directory not found: ";
pub static NOT_A_FILE: &str = "Path is not a file: ";
pub static NOT_A_DIR: &str = "Path is not a directory: ";
pub static WRITE_ERROR: &str = "Failed to write file: ";
pub static READ_ERROR: &str = "Failed to read file: ";
pub static DOWNLOAD_ERROR: &str = "Failed to download file: ";
pub static API_ERROR: &str = "API Error: ";
pub static INPUT_ERROR: &str = "Input Error: ";
pub static UNEXPECTED_ERROR: &str = "Unexpected Error: ";

/// Enum representing different types of CLI errors
#[derive(Debug)]
pub enum CliError {
    /// API error from the underlying vkteams-bot crate
    ApiError(BotError),
    /// Error that occurs when file operations fail
    FileError(String),
    /// Error that occurs with invalid CLI arguments
    InputError(String),
    /// Unexpected or general error
    UnexpectedError(String),
}

/// Result type for CLI operations
pub type Result<T> = std::result::Result<T, CliError>;

impl From<BotError> for CliError {
    fn from(error: BotError) -> Self {
        CliError::ApiError(error)
    }
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        CliError::FileError(error.to_string())
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        CliError::UnexpectedError(format!("JSON error: {error}"))
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::ApiError(err) => write!(f, "{API_ERROR}{err}"),
            CliError::FileError(err) => write!(f, "File Error: {err}"),
            CliError::InputError(err) => write!(f, "{INPUT_ERROR}{err}"),
            CliError::UnexpectedError(err) => write!(f, "{UNEXPECTED_ERROR}{err}"),
        }
    }
}

impl std::error::Error for CliError {}

impl CliError {
    /// Returns the appropriate exit code for this error
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::ApiError(_) => exitcode::UNAVAILABLE,
            CliError::FileError(_) => exitcode::IOERR,
            CliError::InputError(_) => exitcode::USAGE,
            CliError::UnexpectedError(_) => exitcode::SOFTWARE,
        }
    }

    // TODO: Enable this method when we need direct error exit functionality
    // /// Prints the error message and exits with appropriate code
    // pub fn exit_with_error(self) -> ! {
    //     // Avoid unnecessary string allocation for commonly used error types
    //     let error_message = match &self {
    //         CliError::ApiError(err) => format!("{API_ERROR}{err}"),
    //         CliError::FileError(msg) => format!("File Error: {msg}"),
    //         CliError::InputError(msg) => format!("{INPUT_ERROR}{msg}"),
    //         CliError::UnexpectedError(msg) => format!("{UNEXPECTED_ERROR}{msg}"),
    //     };
    //
    //     eprintln!("{}", error_message.red());
    //     exit(self.exit_code());
    // }
}

/// A module to re-export all error types and constants
pub mod prelude {
    pub use super::{
        API_ERROR, DIR_NOT_FOUND, DOWNLOAD_ERROR, FILE_NOT_FOUND, INPUT_ERROR, NOT_A_DIR,
        NOT_A_FILE, READ_ERROR, UNEXPECTED_ERROR, WRITE_ERROR,
    };
    pub use super::{CliError, Result};
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::io;
    use vkteams_bot::error::BotError;

    #[test]
    fn test_cli_error_display_and_exit_code() {
        let api_err = CliError::ApiError(BotError::Api(vkteams_bot::error::ApiError {
            description: "api fail".to_string(),
        }));
        assert!(format!("{}", api_err).contains("API Error:"));
        assert_eq!(api_err.exit_code(), exitcode::UNAVAILABLE);

        let file_err = CliError::FileError("file fail".to_string());
        assert!(format!("{}", file_err).contains("File Error:"));
        assert_eq!(file_err.exit_code(), exitcode::IOERR);

        let input_err = CliError::InputError("bad arg".to_string());
        assert!(format!("{}", input_err).contains("Input Error:"));
        assert_eq!(input_err.exit_code(), exitcode::USAGE);

        let unexp_err = CliError::UnexpectedError("boom".to_string());
        assert!(format!("{}", unexp_err).contains("Unexpected Error:"));
        assert_eq!(unexp_err.exit_code(), exitcode::SOFTWARE);
    }

    #[test]
    fn test_from_bot_error() {
        let bot_err = BotError::Api(vkteams_bot::error::ApiError {
            description: "api fail".to_string(),
        });
        let cli_err: CliError = bot_err.into();
        match cli_err {
            CliError::ApiError(_) => {}
            _ => panic!("Expected ApiError variant"),
        }
    }

    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::Other, "io fail");
        let cli_err: CliError = io_err.into();
        match cli_err {
            CliError::FileError(msg) => assert!(msg.contains("io fail")),
            _ => panic!("Expected FileError variant"),
        }
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<u32>("not a number").unwrap_err();
        let cli_err: CliError = json_err.into();
        match cli_err {
            CliError::UnexpectedError(msg) => assert!(msg.contains("JSON error")),
            _ => panic!("Expected UnexpectedError variant"),
        }
    }
}
