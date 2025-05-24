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
    pub use super::{CliError, Result};
    pub use super::{
        API_ERROR, DIR_NOT_FOUND, DOWNLOAD_ERROR, FILE_NOT_FOUND,
        INPUT_ERROR, NOT_A_DIR, NOT_A_FILE, READ_ERROR, 
        UNEXPECTED_ERROR, WRITE_ERROR,
    };
}