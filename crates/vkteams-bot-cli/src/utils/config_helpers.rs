//! Configuration helper utilities for VK Teams Bot CLI
//!
//! This module provides helper functions for configuration management,
//! merging, and validation.

use crate::config::Config;
use crate::errors::prelude::{CliError, Result as CliResult};
use std::path::PathBuf;

/// Get a list of possible config file paths in order of preference
///
/// # Returns
/// * A vector of PathBuf objects representing possible config file locations
pub fn get_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Current directory
    paths.push(PathBuf::from(crate::constants::config::CONFIG_FILE_NAME));

    // User config directory
    if let Some(home_dir) = dirs::home_dir() {
        let mut user_config = home_dir;
        user_config.push(crate::constants::config::DEFAULT_CONFIG_DIR);
        user_config.push(crate::constants::config::CONFIG_FILE_NAME);
        paths.push(user_config);
    }

    // System config directory
    #[cfg(unix)]
    {
        let mut system_config = PathBuf::from("/etc");
        system_config.push("vkteams-bot");
        system_config.push(crate::constants::config::CONFIG_FILE_NAME);
        paths.push(system_config);
    }

    paths
}

/// Merge two configurations, with the overlay taking precedence
///
/// # Arguments
/// * `base` - The base configuration
/// * `overlay` - The overlay configuration (takes precedence)
///
/// # Returns
/// * A merged configuration
pub fn merge_configs(base: Config, overlay: Config) -> Config {
    Config {
        api: merge_api_configs(base.api, overlay.api),
        files: merge_file_configs(base.files, overlay.files),
        logging: merge_logging_configs(base.logging, overlay.logging),
        ui: merge_ui_configs(base.ui, overlay.ui),
        proxy: overlay.proxy.or(base.proxy),
        rate_limit: merge_rate_limit_configs(base.rate_limit, overlay.rate_limit),
    }
}

/// Merge API configurations
fn merge_api_configs(
    base: crate::config::ApiConfig,
    overlay: crate::config::ApiConfig,
) -> crate::config::ApiConfig {
    crate::config::ApiConfig {
        token: overlay.token.or(base.token),
        url: overlay.url.or(base.url),
        timeout: if overlay.timeout == crate::config::default_timeout() {
            base.timeout
        } else {
            overlay.timeout
        },
        max_retries: if overlay.max_retries == crate::config::default_retries() {
            base.max_retries
        } else {
            overlay.max_retries
        },
    }
}

/// Merge file configurations
fn merge_file_configs(
    base: crate::config::FileConfig,
    overlay: crate::config::FileConfig,
) -> crate::config::FileConfig {
    crate::config::FileConfig {
        download_dir: overlay.download_dir.or(base.download_dir),
        upload_dir: overlay.upload_dir.or(base.upload_dir),
        max_file_size: if overlay.max_file_size == crate::config::default_max_file_size() {
            base.max_file_size
        } else {
            overlay.max_file_size
        },
        buffer_size: if overlay.buffer_size == crate::config::default_buffer_size() {
            base.buffer_size
        } else {
            overlay.buffer_size
        },
    }
}

/// Merge logging configurations
fn merge_logging_configs(
    base: crate::config::LoggingConfig,
    overlay: crate::config::LoggingConfig,
) -> crate::config::LoggingConfig {
    crate::config::LoggingConfig {
        level: if overlay.level == crate::config::default_log_level() {
            base.level
        } else {
            overlay.level
        },
        format: if overlay.format == crate::config::default_log_format() {
            base.format
        } else {
            overlay.format
        },
        colors: if overlay.colors == crate::config::default_log_colors() {
            base.colors
        } else {
            overlay.colors
        },
    }
}

/// Merge UI configurations
fn merge_ui_configs(
    base: crate::config::UiConfig,
    overlay: crate::config::UiConfig,
) -> crate::config::UiConfig {
    crate::config::UiConfig {
        show_progress: if overlay.show_progress == crate::config::default_show_progress() {
            base.show_progress
        } else {
            overlay.show_progress
        },
        progress_style: if overlay.progress_style == crate::config::default_progress_style() {
            base.progress_style
        } else {
            overlay.progress_style
        },
        progress_refresh_rate: if overlay.progress_refresh_rate
            == crate::config::default_progress_refresh_rate()
        {
            base.progress_refresh_rate
        } else {
            overlay.progress_refresh_rate
        },
    }
}

/// Merge rate limiting configurations
pub fn merge_rate_limit_configs(
    base: crate::config::RateLimitConfig,
    overlay: crate::config::RateLimitConfig,
) -> crate::config::RateLimitConfig {
    crate::config::RateLimitConfig {
        enabled: if overlay.enabled == crate::config::default_rate_limit_enabled() {
            base.enabled
        } else {
            overlay.enabled
        },
        limit: if overlay.limit == crate::config::default_rate_limit_limit() {
            base.limit
        } else {
            overlay.limit
        },
        duration: if overlay.duration == crate::config::default_rate_limit_duration() {
            base.duration
        } else {
            overlay.duration
        },
        retry_delay: if overlay.retry_delay == crate::config::default_rate_limit_retry_delay() {
            base.retry_delay
        } else {
            overlay.retry_delay
        },
        retry_attempts: if overlay.retry_attempts
            == crate::config::default_rate_limit_retry_attempts()
        {
            base.retry_attempts
        } else {
            overlay.retry_attempts
        },
    }
}

/// Validate a configuration for completeness and correctness
///
/// # Arguments
/// * `config` - The configuration to validate
///
/// # Returns
/// * `Ok(())` if the configuration is valid
/// * `Err(CliError)` if the configuration has issues
pub fn validate_config(config: &Config) -> CliResult<()> {
    // Validate API configuration
    if config.api.token.is_none() {
        return Err(CliError::InputError(
            "API token is required. Set VKTEAMS_BOT_API_TOKEN or configure via setup".to_string(),
        ));
    }

    if config.api.url.is_none() {
        return Err(CliError::InputError(
            "API URL is required. Set VKTEAMS_BOT_API_URL or configure via setup".to_string(),
        ));
    }

    // Validate API URL format
    if let Some(url) = &config.api.url {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(CliError::InputError(
                "API URL must start with http:// or https://".to_string(),
            ));
        }
    }

    // Validate token format (basic check)
    if let Some(token) = &config.api.token {
        if token.trim().is_empty() {
            return Err(CliError::InputError(
                "API token cannot be empty".to_string(),
            ));
        }

        if token.len() < 10 {
            return Err(CliError::InputError(
                "API token appears to be too short".to_string(),
            ));
        }
    }

    // Validate timeout and retries
    if config.api.timeout == 0 {
        return Err(CliError::InputError(
            "API timeout must be greater than 0".to_string(),
        ));
    }

    if config.api.timeout > 300 {
        return Err(CliError::InputError(
            "API timeout should not exceed 300 seconds".to_string(),
        ));
    }

    if config.api.max_retries > 10 {
        return Err(CliError::InputError(
            "Maximum retries should not exceed 10".to_string(),
        ));
    }

    // Validate file configuration
    if config.files.max_file_size == 0 {
        return Err(CliError::InputError(
            "Maximum file size must be greater than 0".to_string(),
        ));
    }

    if config.files.buffer_size == 0 {
        return Err(CliError::InputError(
            "Buffer size must be greater than 0".to_string(),
        ));
    }

    // Validate directories exist if specified
    if let Some(download_dir) = &config.files.download_dir {
        crate::utils::validation::validate_directory_path(download_dir)?;
    }

    if let Some(upload_dir) = &config.files.upload_dir {
        crate::utils::validation::validate_directory_path(upload_dir)?;
    }

    // Validate logging configuration
    let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
    if !valid_log_levels.contains(&config.logging.level.as_str()) {
        return Err(CliError::InputError(format!(
            "Invalid log level: {}. Valid levels: {}",
            config.logging.level,
            valid_log_levels.join(", ")
        )));
    }

    let valid_log_formats = ["json", "text"];
    if !valid_log_formats.contains(&config.logging.format.as_str()) {
        return Err(CliError::InputError(format!(
            "Invalid log format: {}. Valid formats: {}",
            config.logging.format,
            valid_log_formats.join(", ")
        )));
    }

    // Validate UI configuration
    let valid_progress_styles = ["default", "unicode", "ascii"];
    if !valid_progress_styles.contains(&config.ui.progress_style.as_str()) {
        return Err(CliError::InputError(format!(
            "Invalid progress style: {}. Valid styles: {}",
            config.ui.progress_style,
            valid_progress_styles.join(", ")
        )));
    }

    if config.ui.progress_refresh_rate == 0 {
        return Err(CliError::InputError(
            "Progress refresh rate must be greater than 0".to_string(),
        ));
    }

    if config.ui.progress_refresh_rate > 1000 {
        return Err(CliError::InputError(
            "Progress refresh rate should not exceed 1000ms".to_string(),
        ));
    }

    // Validate proxy configuration if present
    if let Some(proxy) = &config.proxy {
        if proxy.url.trim().is_empty() {
            return Err(CliError::InputError(
                "Proxy URL cannot be empty".to_string(),
            ));
        }

        if !proxy.url.starts_with("http://") && !proxy.url.starts_with("https://") {
            return Err(CliError::InputError(
                "Proxy URL must start with http:// or https://".to_string(),
            ));
        }
    }

    // Validate rate limiting configuration
    if config.rate_limit.limit == 0 {
        return Err(CliError::InputError(
            "Rate limit count must be greater than 0".to_string(),
        ));
    }

    if config.rate_limit.duration == 0 {
        return Err(CliError::InputError(
            "Rate limit duration must be greater than 0".to_string(),
        ));
    }

    if config.rate_limit.retry_delay > 60000 {
        return Err(CliError::InputError(
            "Rate limit retry delay should not exceed 60 seconds".to_string(),
        ));
    }

    if config.rate_limit.retry_attempts > 20 {
        return Err(CliError::InputError(
            "Rate limit retry attempts should not exceed 20".to_string(),
        ));
    }

    Ok(())
}

/// Load configuration with environment variable overrides
///
/// # Returns
/// * `Ok(Config)` with the loaded and merged configuration
/// * `Err(CliError)` if loading fails
pub fn load_config_with_env_overrides() -> CliResult<Config> {
    let mut config = toml::from_str::<Config>("").unwrap();

    // Try to load from config file
    if let Ok(file_config) = Config::from_file() {
        config = merge_configs(config, file_config);
    }

    // Overlay with environment variables
    let env_config = Config::from_env()?;
    config = merge_configs(config, env_config);

    Ok(config)
}

/// Check if any config file exists in the standard locations
///
/// # Returns
/// * `true` if at least one config file exists
/// * `false` if no config files are found
pub fn config_file_exists() -> bool {
    get_config_paths().iter().any(|path| path.exists())
}

/// Get the path of the first existing config file
///
/// # Returns
/// * `Some(PathBuf)` if a config file exists
/// * `None` if no config files are found
pub fn get_existing_config_path() -> Option<PathBuf> {
    get_config_paths().into_iter().find(|path| path.exists())
}

/// Create default config directories
///
/// # Returns
/// * `Ok(())` if directories are created successfully
/// * `Err(CliError)` if directory creation fails
pub fn create_default_config_dirs() -> CliResult<()> {
    if let Some(home_dir) = dirs::home_dir() {
        let config_dir = home_dir.join(crate::constants::config::DEFAULT_CONFIG_DIR);
        std::fs::create_dir_all(&config_dir).map_err(|e| {
            CliError::FileError(format!("Failed to create config directory: {}", e))
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_configs() {
        let mut base =
            toml::from_str::<Config>("[api]\n[files]\n[logging]\n[ui]\n[proxy]\n[rate_limit]")
                .unwrap();
        base.api.token = Some("base_token".to_string());
        base.api.timeout = 30;
        base.rate_limit.enabled = true;
        base.rate_limit.limit = 500;

        let mut overlay = toml::from_str::<Config>("").unwrap();
        overlay.api.url = Some("https://api.example.com".to_string());
        overlay.api.timeout = 60;
        overlay.rate_limit.limit = 2000; // Use a non-default value

        let merged = merge_configs(base, overlay);

        assert_eq!(merged.api.token, Some("base_token".to_string()));
        assert_eq!(merged.api.url, Some("https://api.example.com".to_string()));
        assert_eq!(merged.api.timeout, 60);
        assert!(merged.rate_limit.enabled);
        assert_eq!(merged.rate_limit.limit, 2000);
    }

    #[test]
    fn test_validate_config() {
        let mut config = toml::from_str::<Config>("").unwrap();

        // Invalid config should fail
        assert!(
            validate_config(&config)
                .map_err(|e| eprintln!("{}", e))
                .is_err()
        );

        // Valid config should pass
        config.api.token = Some("valid_token_123".to_string());
        config.api.url = Some("https://example.com".to_string());
        assert!(
            validate_config(&config)
                .map_err(|e| eprintln!("{}", e))
                .is_ok()
        );

        // Invalid URL should fail
        config.api.url = Some("invalid-url".to_string());
        assert!(
            validate_config(&config)
                .map_err(|e| eprintln!("{}", e))
                .is_err()
        );
    }

    #[test]
    fn test_get_config_paths() {
        let paths = get_config_paths();
        assert!(!paths.is_empty());
        assert!(paths[0].ends_with(crate::constants::config::CONFIG_FILE_NAME));
    }

    #[test]
    fn test_config_file_exists() {
        // This test depends on the current environment
        // We can't guarantee config files exist, so just test it doesn't panic
        let _exists = config_file_exists();
    }

    #[test]
    fn test_merge_rate_limit_configs() {
        let base = crate::config::RateLimitConfig {
            enabled: true,
            limit: 100,
            duration: 30,      // Non-default value
            retry_delay: 250,  // Non-default value
            retry_attempts: 2, // Non-default value
        };

        let overlay = crate::config::RateLimitConfig {
            enabled: true, // Use true instead of false to avoid default
            limit: 500,
            duration: 120,
            retry_delay: 800, // Non-default value
            retry_attempts: 5,
        };

        let merged = merge_rate_limit_configs(base, overlay);

        assert!(merged.enabled);
        assert_eq!(merged.limit, 500);
        assert_eq!(merged.duration, 120);
        assert_eq!(merged.retry_delay, 800);
        assert_eq!(merged.retry_attempts, 5);
    }
}
