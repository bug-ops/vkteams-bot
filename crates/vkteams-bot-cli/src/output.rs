//! Unified output formatting for CLI

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use colored::Colorize;
use crate::commands::OutputFormat;
use crate::errors::prelude::CliError;

/// Unified CLI response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub command: String,
}

impl<T: Serialize> CliResponse<T> {
    /// Create successful response with data
    pub fn success(command: impl Into<String>, data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
            command: command.into(),
        }
    }

    /// Create successful response without data
    pub fn success_empty(command: impl Into<String>) -> Self {
        Self {
            success: true,
            data: None,
            error: None,
            timestamp: Utc::now(),
            command: command.into(),
        }
    }

    /// Create error response
    pub fn error(command: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
            timestamp: Utc::now(),
            command: command.into(),
        }
    }
}

impl<T> From<Result<T, CliError>> for CliResponse<T> 
where T: Serialize
{
    fn from(result: Result<T, CliError>) -> Self {
        match result {
            Ok(data) => CliResponse::success("unknown", data),
            Err(err) => CliResponse::error("unknown", err.to_string()),
        }
    }
}

/// Output formatting utility
pub struct OutputFormatter;

impl OutputFormatter {
    /// Format and print response according to output format
    pub fn print<T: Serialize>(
        response: &CliResponse<T>,
        format: &OutputFormat,
    ) -> Result<(), CliError> {
        match format {
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(response)
                    .map_err(|e| CliError::UnexpectedError(format!("JSON serialization error: {}", e)))?;
                println!("{}", json);
            }
            OutputFormat::Pretty => {
                Self::print_pretty(response)?;
            }
            OutputFormat::Table => {
                // For now, fall back to pretty format
                Self::print_pretty(response)?;
            }
            OutputFormat::Quiet => {
                // Only print errors in quiet mode
                if !response.success {
                    if let Some(error) = &response.error {
                        eprintln!("{}", error.red());
                    }
                }
            }
        }
        Ok(())
    }

    fn print_pretty<T: Serialize>(response: &CliResponse<T>) -> Result<(), CliError> {
        if response.success {
            println!("{} {}", "✓".green(), "Success".green().bold());
            
            if let Some(data) = &response.data {
                let data_json = serde_json::to_value(data)
                    .map_err(|e| CliError::UnexpectedError(format!("JSON serialization error: {}", e)))?;
                
                match data_json {
                    serde_json::Value::Object(map) => {
                        for (key, value) in map {
                            println!("  {}: {}", key.cyan(), Self::format_value(&value));
                        }
                    }
                    serde_json::Value::Array(arr) => {
                        for (i, item) in arr.iter().enumerate() {
                            println!("  [{}]: {}", i.to_string().yellow(), Self::format_value(item));
                        }
                    }
                    _ => {
                        println!("  {}", Self::format_value(&data_json));
                    }
                }
            }
        } else {
            println!("{} {}", "✗".red(), "Error".red().bold());
            if let Some(error) = &response.error {
                println!("  {}", error.red());
            }
        }

        // Print metadata in quiet colors
        println!();
        println!("{}: {}", "Command".dimmed(), response.command.dimmed());
        println!("{}: {}", "Timestamp".dimmed(), response.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string().dimmed());
        
        Ok(())
    }

    fn format_value(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "null".dimmed().to_string(),
            serde_json::Value::Array(arr) => {
                format!("[{} items]", arr.len())
            }
            serde_json::Value::Object(obj) => {
                format!("{{{} fields}}", obj.len())
            }
        }
    }
}

/// Macro for easy CLI response creation
#[macro_export]
macro_rules! cli_response {
    (success, $command:expr, $data:expr) => {
        $crate::output::CliResponse::success($command, $data)
    };
    (success, $command:expr) => {
        $crate::output::CliResponse::success_empty($command)
    };
    (error, $command:expr, $error:expr) => {
        $crate::output::CliResponse::<serde_json::Value>::error($command, $error)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_success_response() {
        let data = json!({"message": "Hello", "count": 42});
        let response = CliResponse::success("test-command", data.clone());
        
        assert!(response.success);
        assert_eq!(response.data, Some(data));
        assert!(response.error.is_none());
        assert_eq!(response.command, "test-command");
    }

    #[test]
    fn test_error_response() {
        let response = CliResponse::<serde_json::Value>::error("test-command", "Something went wrong");
        
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_json_serialization() {
        let data = json!({"key": "value"});
        let response = CliResponse::success("test", data);
        
        let json_str = serde_json::to_string(&response).unwrap();
        assert!(json_str.contains("\"success\":true"));
        assert!(json_str.contains("\"command\":\"test\""));
    }
}