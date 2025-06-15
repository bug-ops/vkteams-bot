#![allow(unsafe_code)]

//! Tests for config module that require unsafe operations
//! These tests are separated to avoid conflicts with the forbid(unsafe_code) directive

use serial_test::serial;
use std::io::Write;
use tempfile::NamedTempFile;
use vkteams_bot::config::{Config, types::APP_FOLDER};

// Helper functions for environment variable manipulation in tests
fn remove_env_var(key: &str) {
    unsafe {
        std::env::remove_var(key);
    }
}

fn set_env_var(key: &str, value: &str) {
    unsafe {
        std::env::set_var(key, value);
    }
}

#[test]
#[serial]
fn test_config_new_fallback_to_default() {
    // Remove environment variable to test fallback
    remove_env_var(APP_FOLDER);

    let config = Config::new();

    // Should use default values
    assert_eq!(config.network.retries, 3);
    assert_eq!(config.network.max_backoff_ms, 5000);

    #[cfg(feature = "otlp")]
    {
        assert_eq!(config.otlp.instance_id, "bot");
        assert_eq!(config.otlp.deployment_environment_name, "dev");
    }
}

#[test]
#[serial]
fn test_get_config_missing_env_var() {
    remove_env_var(APP_FOLDER);

    let result = vkteams_bot::config::get_config();
    assert!(result.is_err());

    match result.unwrap_err() {
        vkteams_bot::error::BotError::Environment(_) => {} // Expected
        _ => panic!("Expected Environment error"),
    }
}

#[test]
#[serial]
fn test_get_config_missing_file() {
    set_env_var(APP_FOLDER, "/path/that/does/not/exist/config.toml");

    let result = vkteams_bot::config::get_config();
    assert!(result.is_err());

    // Clean up
    remove_env_var(APP_FOLDER);
}

#[test]
#[serial]
fn test_get_config_invalid_toml() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "invalid toml content [[[").unwrap();

    set_env_var(APP_FOLDER, &temp_file.path().to_string_lossy().to_string());

    let result = vkteams_bot::config::get_config();
    assert!(result.is_err());

    // Clean up
    remove_env_var(APP_FOLDER);
}

#[test]
#[serial]
fn test_get_config_valid_toml() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[network]
retries = 5
max_backoff_ms = 60000
request_timeout_secs = 45

[listener]
empty_backoff_ms = 2000
timeout_s = 15

[otlp]
instance_id = "test-bot"
deployment_environment_name = "test"
exporter_endpoint = "https://test.example.com/v1/traces"
exporter_timeout = 60
exporter_metric_interval = 90
ratio = 0.5
otel_filter_default = "warn"
fmt_filter_default = "info"
fmt_filter_self_directive = "debug"
log_format = "json"
fmt_ansi = false
"#
    )
    .unwrap();

    set_env_var(APP_FOLDER, &temp_file.path().to_string_lossy().to_string());

    let result = vkteams_bot::config::get_config();
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.network.retries, 5);
    assert_eq!(config.network.max_backoff_ms, 60000);
    assert_eq!(config.network.request_timeout_secs, 45);
    assert_eq!(config.listener.empty_backoff_ms, 2000);

    #[cfg(feature = "otlp")]
    {
        assert_eq!(config.otlp.instance_id, "test-bot");
        assert_eq!(config.otlp.deployment_environment_name, "test");
        assert_eq!(
            config.otlp.exporter_endpoint,
            Some("https://test.example.com/v1/traces".into())
        );
        assert_eq!(config.otlp.exporter_timeout, 60);
        assert_eq!(config.otlp.exporter_metric_interval, 90);
        assert_eq!(config.otlp.ratio, 0.5);
        assert_eq!(config.otlp.otel_filter_default, "warn");
        assert_eq!(config.otlp.fmt_filter_default, "info");
        assert_eq!(config.otlp.fmt_filter_self_directive, "debug");
        assert_eq!(config.otlp.log_format, vkteams_bot::config::LogFormat::Json);
        assert!(!config.otlp.fmt_ansi);
    }

    // Clean up
    remove_env_var(APP_FOLDER);
}

#[cfg(all(feature = "otlp", not(target_os = "windows")))]
#[test]
#[serial]
fn test_get_config_permission_denied() {
    set_env_var(APP_FOLDER, "/non/existent/directory/config.toml");

    let result = vkteams_bot::config::get_config();
    assert!(result.is_err());

    // Try with likely permission denied path
    if std::path::Path::new("/root").exists() {
        set_env_var(APP_FOLDER, "/root/config.toml");
        let result = vkteams_bot::config::get_config();
        assert!(result.is_err());
    }

    // Clean up
    remove_env_var(APP_FOLDER);
}

#[cfg(feature = "otlp")]
#[test]
#[serial]
fn test_get_config_valid_otlp_config() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[otlp]
instance_id = "integration-test-bot"
deployment_environment_name = "integration"
exporter_endpoint = "https://integration.example.com/v1/traces"
exporter_timeout = 120
exporter_metric_interval = 120
ratio = 0.1
otel_filter_default = "info"
fmt_filter_default = "debug"
fmt_filter_self_directive = "trace"
log_format = "pretty"
fmt_ansi = true
"#
    )
    .unwrap();

    set_env_var(APP_FOLDER, &temp_file.path().to_string_lossy().to_string());

    let result = vkteams_bot::config::get_config();
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.otlp.instance_id, "integration-test-bot");
    assert_eq!(config.otlp.deployment_environment_name, "integration");
    assert_eq!(
        config.otlp.exporter_endpoint,
        Some("https://integration.example.com/v1/traces".into())
    );
    assert_eq!(config.otlp.exporter_timeout, 120);
    assert_eq!(config.otlp.exporter_metric_interval, 120);
    assert_eq!(config.otlp.ratio, 0.1);
    assert_eq!(config.otlp.otel_filter_default, "info");
    assert_eq!(config.otlp.fmt_filter_default, "debug");
    assert_eq!(config.otlp.fmt_filter_self_directive, "trace");
    assert_eq!(
        config.otlp.log_format,
        vkteams_bot::config::LogFormat::Pretty
    );
    assert!(config.otlp.fmt_ansi);

    // Clean up
    remove_env_var(APP_FOLDER);
}
