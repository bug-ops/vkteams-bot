//! Integration tests for JSON output functionality

use serde_json::Value;
use std::process::Command;

#[test]
#[ignore] // Requires configured bot token
fn test_json_output_diagnostic_health_check() {
    let output = Command::new("cargo")
        .args(["run", "--", "--output", "json", "health-check"])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).expect("Failed to parse JSON output");

    assert!(json["success"].is_boolean());
    assert!(json["command"].is_string());
    assert!(json["timestamp"].is_string());
}

#[test]
#[ignore] // Requires configured bot token
fn test_json_output_scheduler_list() {
    let output = Command::new("cargo")
        .args(["run", "--", "--output", "json", "scheduler", "list"])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).expect("Failed to parse JSON output");

    assert!(json["success"].is_boolean());
    assert!(json["data"]["tasks"].is_array());
    assert!(json["data"]["total"].is_number());
}

#[test]
fn test_json_output_config_examples() {
    let output = Command::new("cargo")
        .args(["run", "--", "--output", "json", "examples"])
        .output()
        .expect("Failed to execute command");

    // This command doesn't require bot token
    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).expect("Failed to parse JSON output");

    assert!(json["success"].is_boolean());
    assert!(json["command"].is_string());
    assert!(json["data"]["examples"].is_array());
}

#[test]
fn test_json_output_pretty_format_default() {
    let output_pretty = Command::new("cargo")
        .args(["run", "--", "examples"])
        .output()
        .expect("Failed to execute command");

    let output_json = Command::new("cargo")
        .args(["run", "--", "--output", "json", "examples"])
        .output()
        .expect("Failed to execute command");

    assert!(output_pretty.status.success());
    assert!(output_json.status.success());

    // Output should be different
    assert_ne!(output_pretty.stdout, output_json.stdout);

    // JSON output should be parseable
    let stdout = String::from_utf8_lossy(&output_json.stdout);
    let _: Value = serde_json::from_str(&stdout).expect("Failed to parse JSON output");
}
