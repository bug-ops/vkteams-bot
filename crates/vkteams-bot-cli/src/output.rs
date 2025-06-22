//! Unified output formatting for CLI

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use colored::Colorize;
use crate::commands::OutputFormat;
use crate::errors::prelude::CliError;
use tabled::{Table, Tabled};
use std::collections::BTreeMap;

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

/// Row representation for table output
#[derive(Debug, Clone, Tabled)]
struct TableRow {
    #[tabled(rename = "Key")]
    pub key: String,
    #[tabled(rename = "Value")]
    pub value: String,
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
                Self::print_table(response)?;
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

    fn print_table<T: Serialize>(response: &CliResponse<T>) -> Result<(), CliError> {
        if response.success {
            if let Some(data) = &response.data {
                let data_json = serde_json::to_value(data)
                    .map_err(|e| CliError::UnexpectedError(format!("JSON serialization error: {}", e)))?;
                
                Self::print_table_data(&data_json)?;
            } else {
                // Success but no data
                let rows = vec![
                    TableRow {
                        key: "Status".to_string(),
                        value: "Success".to_string(),
                    }
                ];
                let table = Table::new(rows);
                println!("{}", table);
            }
        } else {
            // Error case
            let mut rows = vec![
                TableRow {
                    key: "Status".to_string(),
                    value: "Error".to_string(),
                }
            ];
            
            if let Some(error) = &response.error {
                rows.push(TableRow {
                    key: "Error".to_string(),
                    value: error.clone(),
                });
            }
            
            let table = Table::new(rows);
            println!("{}", table);
        }

        // Add metadata
        let metadata_rows = vec![
            TableRow {
                key: "Command".to_string(),
                value: response.command.clone(),
            },
            TableRow {
                key: "Timestamp".to_string(),
                value: response.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            }
        ];
        
        println!();
        let metadata_table = Table::new(metadata_rows);
        println!("{}", metadata_table);
        
        Ok(())
    }

    fn print_table_data(data: &serde_json::Value) -> Result<(), CliError> {
        match data {
            serde_json::Value::Object(map) => {
                // Convert object to key-value table
                let mut rows = Vec::new();
                
                // Use BTreeMap to ensure consistent ordering
                let ordered_map: BTreeMap<String, &serde_json::Value> = map.iter()
                    .map(|(k, v)| (k.clone(), v))
                    .collect();
                
                for (key, value) in ordered_map {
                    rows.push(TableRow {
                        key,
                        value: Self::format_table_value(value),
                    });
                }
                
                let table = Table::new(rows);
                println!("{}", table);
            }
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    println!("No data available");
                    return Ok(());
                }
                
                // Try to create a table from array of objects
                if let Some(first) = arr.first() {
                    if let serde_json::Value::Object(_) = first {
                        Self::print_array_as_table(arr)?;
                    } else {
                        // Array of primitives - show as numbered list in table format
                        let rows: Vec<TableRow> = arr.iter()
                            .enumerate()
                            .map(|(i, item)| TableRow {
                                key: format!("Item {}", i + 1),
                                value: Self::format_table_value(item),
                            })
                            .collect();
                        
                        let table = Table::new(rows);
                        println!("{}", table);
                    }
                }
            }
            _ => {
                // Single value
                let rows = vec![
                    TableRow {
                        key: "Value".to_string(),
                        value: Self::format_table_value(data),
                    }
                ];
                let table = Table::new(rows);
                println!("{}", table);
            }
        }
        
        Ok(())
    }

    fn print_array_as_table(arr: &[serde_json::Value]) -> Result<(), CliError> {
        // Collect all unique keys from objects in the array
        let mut all_keys: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        
        for item in arr {
            if let serde_json::Value::Object(obj) = item {
                for key in obj.keys() {
                    all_keys.insert(key.clone());
                }
            }
        }
        
        if all_keys.is_empty() {
            println!("No data available");
            return Ok(());
        }
        
        // Create dynamic table data
        let mut table_data: Vec<Vec<String>> = Vec::new();
        
        // Add header row
        let header: Vec<String> = all_keys.iter().cloned().collect();
        table_data.push(header);
        
        // Add data rows
        for item in arr {
            if let serde_json::Value::Object(obj) = item {
                let mut row = Vec::new();
                for key in &all_keys {
                    let value = obj.get(key)
                        .map(Self::format_table_value)
                        .unwrap_or_else(|| "-".to_string());
                    row.push(value);
                }
                table_data.push(row);
            }
        }
        
        // Convert to table using the raw data approach
        if !table_data.is_empty() {
            let table = Table::from_iter(table_data);
            println!("{}", table);
        }
        
        Ok(())
    }

    fn format_table_value(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "null".to_string(),
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    "[]".to_string()
                } else if arr.len() <= 3 {
                    // Show small arrays inline
                    let items: Vec<String> = arr.iter()
                        .map(Self::format_table_value)
                        .collect();
                    format!("[{}]", items.join(", "))
                } else {
                    format!("[{} items]", arr.len())
                }
            }
            serde_json::Value::Object(obj) => {
                if obj.is_empty() {
                    "{}".to_string()
                } else if obj.len() <= 2 {
                    // Show small objects inline
                    let items: Vec<String> = obj.iter()
                        .map(|(k, v)| format!("{}: {}", k, Self::format_table_value(v)))
                        .collect();
                    format!("{{{}}}", items.join(", "))
                } else {
                    format!("{{{} fields}}", obj.len())
                }
            }
        }
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

    #[test]
    fn test_table_row_creation() {
        let row = TableRow {
            key: "test_key".to_string(),
            value: "test_value".to_string(),
        };
        
        assert_eq!(row.key, "test_key");
        assert_eq!(row.value, "test_value");
    }

    #[test]
    fn test_format_table_value() {
        // Test string
        let value = serde_json::Value::String("test".to_string());
        assert_eq!(OutputFormatter::format_table_value(&value), "test");
        
        // Test number
        let value = serde_json::Value::Number(serde_json::Number::from(42));
        assert_eq!(OutputFormatter::format_table_value(&value), "42");
        
        // Test boolean
        let value = serde_json::Value::Bool(true);
        assert_eq!(OutputFormatter::format_table_value(&value), "true");
        
        // Test null
        let value = serde_json::Value::Null;
        assert_eq!(OutputFormatter::format_table_value(&value), "null");
        
        // Test empty array
        let value = serde_json::Value::Array(vec![]);
        assert_eq!(OutputFormatter::format_table_value(&value), "[]");
        
        // Test small array
        let value = serde_json::Value::Array(vec![
            serde_json::Value::String("a".to_string()),
            serde_json::Value::String("b".to_string()),
        ]);
        assert_eq!(OutputFormatter::format_table_value(&value), "[a, b]");
        
        // Test large array
        let value = serde_json::Value::Array(vec![
            serde_json::Value::String("a".to_string()),
            serde_json::Value::String("b".to_string()),
            serde_json::Value::String("c".to_string()),
            serde_json::Value::String("d".to_string()),
        ]);
        assert_eq!(OutputFormatter::format_table_value(&value), "[4 items]");
    }

    #[test]
    fn test_table_output_object() {
        use crate::commands::OutputFormat;
        
        let data = json!({
            "name": "test",
            "value": 42,
            "enabled": true
        });
        let response = CliResponse::success("test-table", data);
        
        // This would normally print to stdout, so we can't easily test the output
        // But we can verify it doesn't panic
        let result = OutputFormatter::print(&response, &OutputFormat::Table);
        assert!(result.is_ok());
    }

    #[test] 
    fn test_table_output_array() {
        use crate::commands::OutputFormat;
        
        let data = json!([
            {"name": "item1", "value": 1},
            {"name": "item2", "value": 2}
        ]);
        let response = CliResponse::success("test-table-array", data);
        
        let result = OutputFormatter::print(&response, &OutputFormat::Table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_table_output_error() {
        use crate::commands::OutputFormat;
        
        let response = CliResponse::<serde_json::Value>::error("test-error", "Something went wrong");
        
        let result = OutputFormatter::print(&response, &OutputFormat::Table);
        assert!(result.is_ok());
    }
}