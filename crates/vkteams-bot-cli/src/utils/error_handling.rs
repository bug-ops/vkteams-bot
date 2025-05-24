//! Error handling utilities for VK Teams Bot CLI
//!
//! This module provides utilities for error handling, logging, and
//! error formatting throughout the CLI application.

use crate::errors::prelude::{CliError, Result as CliResult};
use crate::constants::ui::emoji;
use colored::Colorize;
use tracing::{debug, error, info};
use vkteams_bot::error::BotError;

/// Convert a BotError to a CliError with additional context
///
/// # Arguments
/// * `error` - The BotError to convert
/// * `context` - Optional context string to add to the error
///
/// # Returns
/// * A CliError with appropriate error type and message
pub fn handle_api_error(error: BotError, context: Option<&str>) -> CliError {
    let error_msg = if let Some(ctx) = context {
        format!("{}: {}", ctx, error)
    } else {
        error.to_string()
    };
    
    debug!("API error occurred: {}", error_msg);
    CliError::ApiError(error)
}

/// Log command execution start
///
/// # Arguments
/// * `command_name` - The name of the command being executed
/// * `args` - Optional command arguments for logging
pub fn log_command_start(command_name: &str, args: Option<&str>) {
    if let Some(args) = args {
        info!("Executing command '{}' with args: {}", command_name, args);
    } else {
        info!("Executing command '{}'", command_name);
    }
}

/// Log command execution result
///
/// # Arguments
/// * `command_name` - The name of the command that was executed
/// * `success` - Whether the command succeeded
/// * `duration` - Optional execution duration
pub fn log_command_execution(command_name: &str, success: bool, duration: Option<std::time::Duration>) {
    let duration_str = if let Some(d) = duration {
        format!(" (took {:.2}s)", d.as_secs_f64())
    } else {
        String::new()
    };

    if success {
        info!("Command '{}' completed successfully{}", command_name, duration_str);
    } else {
        error!("Command '{}' failed{}", command_name, duration_str);
    }
}

/// Setup logging based on configuration
///
/// # Arguments
/// * `verbose` - Whether to enable verbose (debug) logging
/// * `log_level` - Optional specific log level to use
pub fn setup_logging(verbose: bool, log_level: Option<&str>) {
    let level = if verbose {
        "debug"
    } else {
        log_level.unwrap_or("info")
    };

    unsafe {
        std::env::set_var("RUST_LOG", level);
    }

    // Initialize pretty_env_logger if not already initialized
    if std::env::var("RUST_LOG_INITIALIZED").is_err() {
        pretty_env_logger::init();
        unsafe {
            std::env::set_var("RUST_LOG_INITIALIZED", "1");
        }
    }
}

/// Print a formatted error message to stderr
///
/// # Arguments
/// * `error` - The error to print
/// * `show_details` - Whether to show detailed error information
pub fn print_error(error: &CliError, show_details: bool) {
    match error {
        CliError::ApiError(api_err) => {
            eprintln!("{} API Error: {}", emoji::CROSS.red(), api_err.to_string().red());
            if show_details {
                eprintln!("  {}", "This is likely a network or authentication issue.".dimmed());
                eprintln!("  {}", "Try running 'vkteams-bot-cli validate' to test your configuration.".dimmed());
            }
        }
        CliError::FileError(msg) => {
            eprintln!("{} File Error: {}", emoji::CROSS.red(), msg.red());
            if show_details {
                eprintln!("  {}", "Check that the file path is correct and you have the necessary permissions.".dimmed());
            }
        }
        CliError::InputError(msg) => {
            eprintln!("{} Input Error: {}", emoji::CROSS.red(), msg.red());
            if show_details {
                eprintln!("  {}", "Check your command arguments and try again.".dimmed());
                eprintln!("  {}", "Use --help for usage information.".dimmed());
            }
        }
        CliError::UnexpectedError(msg) => {
            eprintln!("{} Unexpected Error: {}", emoji::CROSS.red(), msg.red());
            if show_details {
                eprintln!("  {}", "This may be a bug. Please report it if the issue persists.".dimmed());
            }
        }
    }
}

/// Print a warning message
///
/// # Arguments
/// * `message` - The warning message to print
pub fn print_warning(message: &str) {
    println!("{} {}", emoji::WARNING.yellow(), message.yellow());
}

/// Print an info message
///
/// # Arguments
/// * `message` - The info message to print
pub fn print_info(message: &str) {
    println!("{} {}", emoji::INFO.blue(), message.blue());
}

/// Print a success message
///
/// # Arguments
/// * `message` - The success message to print
pub fn print_success(message: &str) {
    println!("{} {}", emoji::CHECK.green(), message.green());
}

/// Wrap a function call with error handling and logging
///
/// # Arguments
/// * `operation_name` - Name of the operation for logging
/// * `operation` - The operation to execute
///
/// # Returns
/// * The result of the operation with enhanced error handling
pub async fn with_error_handling<F, T, E>(operation_name: &str, operation: F) -> CliResult<T>
where
    F: std::future::Future<Output = Result<T, E>>,
    E: Into<CliError>,
{
    let start_time = std::time::Instant::now();
    log_command_start(operation_name, None);

    let result = operation.await;
    let duration = start_time.elapsed();

    match result {
        Ok(value) => {
            log_command_execution(operation_name, true, Some(duration));
            Ok(value)
        }
        Err(error) => {
            let cli_error = error.into();
            log_command_execution(operation_name, false, Some(duration));
            error!("Operation '{}' failed: {}", operation_name, cli_error);
            Err(cli_error)
        }
    }
}

/// Create a user-friendly error message for common error scenarios
///
/// # Arguments
/// * `error` - The error to create a friendly message for
///
/// # Returns
/// * A user-friendly error message with suggestions
pub fn create_friendly_error_message(error: &CliError) -> String {
    match error {
        CliError::ApiError(_) => {
            format!(
                "{}Network or API error occurred. This could be due to:\n\
                 • Invalid API token or URL\n\
                 • Network connectivity issues\n\
                 • VK Teams service unavailable\n\
                 \n\
                 Try: vkteams-bot-cli validate",
                emoji::CROSS
            )
        }
        CliError::FileError(msg) if msg.contains("not found") => {
            format!(
                "{}File not found. Make sure:\n\
                 • The file path is correct\n\
                 • You have permission to access the file\n\
                 • The file hasn't been moved or deleted",
                emoji::CROSS
            )
        }
        CliError::FileError(msg) if msg.contains("permission") => {
            format!(
                "{}Permission denied. Try:\n\
                 • Running with appropriate permissions\n\
                 • Checking file/directory ownership\n\
                 • Ensuring the path is writable",
                emoji::CROSS
            )
        }
        CliError::FileError(_) => {
            format!(
                "{}File operation failed. Please:\n\
                 • Check that the file path is correct\n\
                 • Verify you have the necessary permissions\n\
                 • Ensure the file is not locked by another process",
                emoji::CROSS
            )
        }
        CliError::InputError(_) => {
            format!(
                "{}Invalid input provided. Please:\n\
                 • Check your command arguments\n\
                 • Use --help for usage information\n\
                 • Refer to examples with: vkteams-bot-cli examples",
                emoji::CROSS
            )
        }
        CliError::UnexpectedError(_) => {
            format!(
                "{}An unexpected error occurred. You can:\n\
                 • Try running the command again\n\
                 • Check the logs for more details\n\
                 • Report this issue if it persists",
                emoji::CROSS
            )
        }
    }
}

/// Handle and format validation errors
///
/// # Arguments
/// * `errors` - A vector of validation errors
///
/// # Returns
/// * A formatted string with all validation errors
pub fn format_validation_errors(errors: &[String]) -> String {
    if errors.is_empty() {
        return String::new();
    }

    let mut formatted = format!("{} Validation failed:\n", emoji::CROSS.red());
    for (i, error) in errors.iter().enumerate() {
        formatted.push_str(&format!("  {}. {}\n", i + 1, error));
    }
    formatted
}

/// Check if an error suggests a configuration problem
///
/// # Arguments
/// * `error` - The error to check
///
/// # Returns
/// * `true` if the error suggests a configuration issue
pub fn is_config_error(error: &CliError) -> bool {
    match error {
        CliError::InputError(msg) => {
            msg.contains("token") || msg.contains("URL") || msg.contains("configure")
        }
        CliError::ApiError(_) => true, // API errors often indicate config issues
        _ => false,
    }
}

/// Suggest next steps based on an error
///
/// # Arguments
/// * `error` - The error to analyze
///
/// # Returns
/// * A vector of suggested next steps
pub fn suggest_next_steps(error: &CliError) -> Vec<String> {
    let mut suggestions = Vec::new();

    if is_config_error(error) {
        suggestions.push("Run 'vkteams-bot-cli setup' to configure the CLI".to_string());
        suggestions.push("Check your API token and URL with 'vkteams-bot-cli validate'".to_string());
    }

    match error {
        CliError::FileError(_) => {
            suggestions.push("Verify the file path is correct".to_string());
            suggestions.push("Check file permissions".to_string());
        }
        CliError::InputError(_) => {
            suggestions.push("Use --help for command usage information".to_string());
            suggestions.push("See examples with 'vkteams-bot-cli examples'".to_string());
        }
        CliError::UnexpectedError(_) => {
            suggestions.push("Try running the command again".to_string());
            suggestions.push("Enable verbose logging with --verbose".to_string());
        }
        _ => {}
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_config_error() {
        let token_error = CliError::InputError("API token is required".to_string());
        assert!(is_config_error(&token_error));

        let file_error = CliError::FileError("File not found".to_string());
        assert!(!is_config_error(&file_error));
    }

    #[test]
    fn test_suggest_next_steps() {
        let config_error = CliError::InputError("API token is required".to_string());
        let suggestions = suggest_next_steps(&config_error);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("setup")));
    }

    #[test]
    fn test_format_validation_errors() {
        let errors = vec![
            "First error".to_string(),
            "Second error".to_string(),
        ];
        let formatted = format_validation_errors(&errors);
        assert!(formatted.contains("1. First error"));
        assert!(formatted.contains("2. Second error"));
    }

    #[test]
    fn test_create_friendly_error_message() {
        let input_error = CliError::InputError("Invalid command".to_string());
        let message = create_friendly_error_message(&input_error);
        assert!(message.contains("Invalid input"));
        assert!(message.contains("--help"));
    }
}