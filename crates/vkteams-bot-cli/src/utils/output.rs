//! Output formatting utilities for VK Teams Bot CLI
//!
//! This module provides consistent output formatting functions for different
//! output formats and display modes.

use crate::commands::OutputFormat;
use crate::constants::ui::emoji;
use crate::errors::prelude::{CliError, Result as CliResult};
use colored::Colorize;
use serde_json;
use std::collections::HashMap;

/// Print a successful API result in the specified format
///
/// # Arguments
/// * `result` - The result to print (must be serializable)
/// * `format` - The output format to use
///
/// # Returns
/// * `Ok(())` if printing succeeds
/// * `Err(CliError)` if serialization or printing fails
pub fn print_success_result<T>(result: &T, format: &OutputFormat) -> CliResult<()>
where
    T: serde::Serialize,
{
    match format {
        OutputFormat::Pretty => print_pretty_result(result),
        OutputFormat::Json => print_json_result(result),
        OutputFormat::Table => print_table_result(result),
        OutputFormat::Quiet => Ok(()), // No output in quiet mode
    }
}

/// Print result in pretty colored format
fn print_pretty_result<T>(result: &T) -> CliResult<()>
where
    T: serde::Serialize,
{
    let json_str = serde_json::to_string_pretty(result)
        .map_err(|e| CliError::UnexpectedError(format!("Failed to serialize response: {e}")))?;

    println!("{}", json_str.green());
    Ok(())
}

/// Print result in JSON format
fn print_json_result<T>(result: &T) -> CliResult<()>
where
    T: serde::Serialize,
{
    let json_str = serde_json::to_string(result)
        .map_err(|e| CliError::UnexpectedError(format!("Failed to serialize response: {e}")))?;

    println!("{json_str}");
    Ok(())
}

/// Print result in table format using tabled crate
fn print_table_result<T>(result: &T) -> CliResult<()>
where
    T: serde::Serialize,
{
    use std::collections::BTreeMap;
    use tabled::{Table, Tabled};

    // Convert to JSON first for processing
    let json_value = serde_json::to_value(result)
        .map_err(|e| CliError::UnexpectedError(format!("Failed to serialize for table: {e}")))?;

    #[derive(Tabled)]
    struct TableRow {
        #[tabled(rename = "Key")]
        key: String,
        #[tabled(rename = "Value")]
        value: String,
    }

    match json_value {
        serde_json::Value::Object(map) => {
            // Convert object to key-value table
            let ordered_map: BTreeMap<String, &serde_json::Value> =
                map.iter().map(|(k, v)| (k.clone(), v)).collect();

            let rows: Vec<TableRow> = ordered_map
                .into_iter()
                .map(|(key, value)| TableRow {
                    key,
                    value: format_table_value(value),
                })
                .collect();

            let table = Table::new(rows);
            println!("{table}");
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                println!("No data available");
                return Ok(());
            }

            // Try to create a table from array of objects
            if let Some(first) = arr.first() {
                if let serde_json::Value::Object(_) = first {
                    print_array_as_table(&arr)?;
                } else {
                    // Array of primitives - show as numbered list
                    let rows: Vec<TableRow> = arr
                        .iter()
                        .enumerate()
                        .map(|(i, item)| TableRow {
                            key: format!("Item {}", i + 1),
                            value: format_table_value(item),
                        })
                        .collect();

                    let table = Table::new(rows);
                    println!("{table}");
                }
            }
        }
        _ => {
            // Single value
            let rows = vec![TableRow {
                key: "Value".to_string(),
                value: format_table_value(&json_value),
            }];
            let table = Table::new(rows);
            println!("{table}");
        }
    }

    Ok(())
}

/// Print array of objects as table
fn print_array_as_table(arr: &[serde_json::Value]) -> CliResult<()> {
    use std::collections::BTreeSet;

    // Collect all unique keys from objects in the array
    let mut all_keys: BTreeSet<String> = BTreeSet::new();

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
                let value = obj
                    .get(key)
                    .map(format_table_value)
                    .unwrap_or_else(|| "-".to_string());
                row.push(value);
            }
            table_data.push(row);
        }
    }

    // Convert to table using the raw data approach
    if !table_data.is_empty() {
        use tabled::Table;
        let table = Table::from_iter(table_data);
        println!("{table}");
    }

    Ok(())
}

/// Format JSON value for table display
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
                let items: Vec<String> = arr.iter().map(format_table_value).collect();
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
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, format_table_value(v)))
                    .collect();
                format!("{{{}}}", items.join(", "))
            } else {
                format!("{{{} fields}}", obj.len())
            }
        }
    }
}

/// Print a simple success message
pub fn print_success_message(message: &str, format: &OutputFormat) {
    match format {
        OutputFormat::Pretty => println!("{} {}", emoji::CHECK, message.green()),
        OutputFormat::Json => {
            let json = serde_json::json!({
                "success": true,
                "message": message
            });
            println!("{json}");
        }
        OutputFormat::Table => println!("{} {}", emoji::CHECK, message),
        OutputFormat::Quiet => {} // No output in quiet mode
    }
}

/// Print an error message
pub fn print_error_message(message: &str, format: &OutputFormat) {
    match format {
        OutputFormat::Pretty => eprintln!("{} {}", emoji::CROSS, message.red()),
        OutputFormat::Json => {
            let json = serde_json::json!({
                "success": false,
                "error": message
            });
            eprintln!("{json}");
        }
        OutputFormat::Table => eprintln!("{} {}", emoji::CROSS, message),
        OutputFormat::Quiet => {} // No output in quiet mode
    }
}

/// Print a warning message
pub fn print_warning_message(message: &str, format: &OutputFormat) {
    match format {
        OutputFormat::Pretty => println!("{} {}", emoji::WARNING, message.yellow()),
        OutputFormat::Json => {
            let json = serde_json::json!({
                "warning": message
            });
            println!("{json}");
        }
        OutputFormat::Table => println!("{} {}", emoji::WARNING, message),
        OutputFormat::Quiet => {} // No output in quiet mode
    }
}

/// Print an info message
pub fn print_info_message(message: &str, format: &OutputFormat) {
    match format {
        OutputFormat::Pretty => println!("{} {}", emoji::INFO, message.blue()),
        OutputFormat::Json => {
            let json = serde_json::json!({
                "info": message
            });
            println!("{json}");
        }
        OutputFormat::Table => println!("{} {}", emoji::INFO, message),
        OutputFormat::Quiet => {} // No output in quiet mode
    }
}

/// Print a list of items in the specified format
pub fn print_list<T>(items: &[T], title: &str, format: &OutputFormat) -> CliResult<()>
where
    T: serde::Serialize + std::fmt::Display,
{
    match format {
        OutputFormat::Pretty => {
            if !title.is_empty() {
                println!("{}", title.bold().blue());
            }
            for (i, item) in items.iter().enumerate() {
                println!("  {}. {}", i + 1, item);
            }
        }
        OutputFormat::Json => {
            let json = serde_json::json!({
                "title": title,
                "items": items
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&json).map_err(|e| CliError::UnexpectedError(
                    format!("Failed to serialize list: {e}")
                ))?
            );
        }
        OutputFormat::Table => {
            use tabled::{Table, Tabled};

            #[derive(Tabled)]
            struct ListItem {
                #[tabled(rename = "#")]
                index: usize,
                #[tabled(rename = "Item")]
                item: String,
            }

            if !title.is_empty() {
                println!("{title}");
                println!("{}", "=".repeat(title.len()));
            }

            let rows: Vec<ListItem> = items
                .iter()
                .enumerate()
                .map(|(i, item)| ListItem {
                    index: i + 1,
                    item: item.to_string(),
                })
                .collect();

            let table = Table::new(rows);
            println!("{table}");
        }
        OutputFormat::Quiet => {} // No output in quiet mode
    }
    Ok(())
}

/// Print key-value pairs in the specified format
pub fn print_key_value_pairs(
    pairs: &HashMap<String, String>,
    title: &str,
    format: &OutputFormat,
) -> CliResult<()> {
    match format {
        OutputFormat::Pretty => {
            if !title.is_empty() {
                println!("{}", title.bold().blue());
            }
            for (key, value) in pairs {
                println!("  {}: {}", key.cyan(), value);
            }
        }
        OutputFormat::Json => {
            let json = serde_json::json!({
                "title": title,
                "data": pairs
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&json).map_err(|e| CliError::UnexpectedError(
                    format!("Failed to serialize key-value pairs: {e}")
                ))?
            );
        }
        OutputFormat::Table => {
            use std::collections::BTreeMap;
            use tabled::{Table, Tabled};

            #[derive(Tabled)]
            struct KeyValue {
                #[tabled(rename = "Key")]
                key: String,
                #[tabled(rename = "Value")]
                value: String,
            }

            if !title.is_empty() {
                println!("{title}");
                println!("{}", "=".repeat(title.len()));
            }

            // Sort keys for consistent output
            let ordered_pairs: BTreeMap<String, String> =
                pairs.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

            let rows: Vec<KeyValue> = ordered_pairs
                .into_iter()
                .map(|(key, value)| KeyValue { key, value })
                .collect();

            let table = Table::new(rows);
            println!("{table}");
        }
        OutputFormat::Quiet => {} // No output in quiet mode
    }
    Ok(())
}

/// Print a progress message (used during operations)
pub fn print_progress_message(message: &str, format: &OutputFormat) {
    match format {
        OutputFormat::Pretty => println!("{} {}", emoji::GEAR, message.blue()),
        OutputFormat::Json => {
            let json = serde_json::json!({
                "progress": message
            });
            println!("{json}");
        }
        OutputFormat::Table | OutputFormat::Quiet => {} // No progress output in these modes
    }
}

/// Print a section header
pub fn print_section_header(title: &str, format: &OutputFormat) {
    match format {
        OutputFormat::Pretty => {
            println!();
            println!("{}", title.bold().blue());
            println!("{}", "─".repeat(title.len()).blue());
        }
        OutputFormat::Table => {
            println!();
            println!("{title}");
            println!("{}", "=".repeat(title.len()));
        }
        OutputFormat::Json | OutputFormat::Quiet => {} // No headers in JSON or quiet mode
    }
}

/// Print statistics or summary information
pub fn print_statistics(
    stats: &HashMap<String, u64>,
    title: &str,
    format: &OutputFormat,
) -> CliResult<()> {
    match format {
        OutputFormat::Pretty => {
            print_section_header(title, format);
            for (key, value) in stats {
                println!("  {}: {}", key.cyan(), value.to_string().bold());
            }
        }
        OutputFormat::Json => {
            let json = serde_json::json!({
                "title": title,
                "statistics": stats
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&json).map_err(|e| CliError::UnexpectedError(
                    format!("Failed to serialize statistics: {e}")
                ))?
            );
        }
        OutputFormat::Table => {
            use std::collections::BTreeMap;
            use tabled::{Table, Tabled};

            #[derive(Tabled)]
            struct Statistic {
                #[tabled(rename = "Metric")]
                metric: String,
                #[tabled(rename = "Value")]
                value: u64,
            }

            println!("{title}");
            println!("{}", "=".repeat(title.len()));

            // Sort keys for consistent output
            let ordered_stats: BTreeMap<String, u64> =
                stats.iter().map(|(k, v)| (k.clone(), *v)).collect();

            let rows: Vec<Statistic> = ordered_stats
                .into_iter()
                .map(|(metric, value)| Statistic { metric, value })
                .collect();

            let table = Table::new(rows);
            println!("{table}");
        }
        OutputFormat::Quiet => {} // No output in quiet mode
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::OutputFormat;
    use std::collections::HashMap;

    #[derive(serde::Serialize)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    #[test]
    fn test_print_json_result() {
        let test_data = TestStruct {
            name: "test".to_string(),
            value: 42,
        };

        // This would normally print to stdout, so we can't easily test the output
        // But we can verify it doesn't panic
        assert!(print_json_result(&test_data).is_ok());
    }

    #[test]
    fn test_print_pretty_result() {
        let test_data = TestStruct {
            name: "test".to_string(),
            value: 42,
        };

        assert!(print_pretty_result(&test_data).is_ok());
    }

    #[test]
    fn test_print_success_message_all_formats() {
        let msg = "Success!";
        print_success_message(msg, &OutputFormat::Pretty);
        print_success_message(msg, &OutputFormat::Json);
        print_success_message(msg, &OutputFormat::Table);
        print_success_message(msg, &OutputFormat::Quiet);
    }

    #[test]
    fn test_print_error_message_all_formats() {
        let msg = "Error!";
        print_error_message(msg, &OutputFormat::Pretty);
        print_error_message(msg, &OutputFormat::Json);
        print_error_message(msg, &OutputFormat::Table);
        print_error_message(msg, &OutputFormat::Quiet);
    }

    #[test]
    fn test_print_warning_message_all_formats() {
        let msg = "Warning!";
        print_warning_message(msg, &OutputFormat::Pretty);
        print_warning_message(msg, &OutputFormat::Json);
        print_warning_message(msg, &OutputFormat::Table);
        print_warning_message(msg, &OutputFormat::Quiet);
    }

    #[test]
    fn test_print_info_message_all_formats() {
        let msg = "info message";
        for format in [
            OutputFormat::Pretty,
            OutputFormat::Json,
            OutputFormat::Table,
            OutputFormat::Quiet,
        ] {
            print_info_message(msg, &format);
        }
    }

    #[test]
    fn test_print_list_all_formats() {
        let items = vec!["item1", "item2"];
        for format in [
            OutputFormat::Pretty,
            OutputFormat::Json,
            OutputFormat::Table,
            OutputFormat::Quiet,
        ] {
            assert!(print_list(&items, "Title", &format).is_ok());
            assert!(print_list::<String>(&[], "", &format).is_ok());
        }
    }

    #[test]
    fn test_print_key_value_pairs_all_formats() {
        let mut pairs = HashMap::new();
        pairs.insert("key1".to_string(), "val1".to_string());
        pairs.insert("key2".to_string(), "val2".to_string());
        for format in [
            OutputFormat::Pretty,
            OutputFormat::Json,
            OutputFormat::Table,
            OutputFormat::Quiet,
        ] {
            assert!(print_key_value_pairs(&pairs, "Title", &format).is_ok());
            assert!(print_key_value_pairs(&HashMap::new(), "", &format).is_ok());
        }
    }

    #[test]
    fn test_print_progress_message_all_formats() {
        let msg = "progress...";
        for format in [
            OutputFormat::Pretty,
            OutputFormat::Json,
            OutputFormat::Table,
            OutputFormat::Quiet,
        ] {
            print_progress_message(msg, &format);
        }
    }

    #[test]
    fn test_print_section_header_all_formats() {
        for format in [
            OutputFormat::Pretty,
            OutputFormat::Table,
            OutputFormat::Json,
            OutputFormat::Quiet,
        ] {
            print_section_header("Section", &format);
            print_section_header("", &format);
        }
    }

    #[test]
    fn test_print_statistics_all_formats() {
        let mut stats = HashMap::new();
        stats.insert("stat1".to_string(), 42u64);
        stats.insert("stat2".to_string(), 0u64);
        for format in [
            OutputFormat::Pretty,
            OutputFormat::Json,
            OutputFormat::Table,
            OutputFormat::Quiet,
        ] {
            assert!(print_statistics(&stats, "Stats", &format).is_ok());
            assert!(print_statistics(&HashMap::new(), "", &format).is_ok());
        }
    }
}
