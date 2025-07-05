//! Bot utilities for VK Teams Bot CLI
//!
//! This module provides utilities for creating and managing Bot instances
//! used throughout the CLI application.

use crate::commands::Commands;
use crate::config::Config;
use crate::errors::prelude::{CliError, Result as CliResult};
use vkteams_bot::prelude::*;

/// Create a bot instance from configuration
///
/// # Arguments
/// * `config` - The configuration containing API credentials
///
/// # Returns
/// * `Ok(Bot)` if the bot instance is created successfully
/// * `Err(CliError)` if required configuration is missing or bot creation fails
pub fn create_bot_instance(config: &Config) -> CliResult<Bot> {
    let token = config.api.token.as_ref()
        .ok_or_else(|| CliError::InputError(
            "API token is required. Set VKTEAMS_BOT_API_TOKEN or configure via 'vkteams-bot-cli setup'".to_string()
        ))?;

    let url = config.api.url.as_ref().ok_or_else(|| {
        CliError::InputError(
            "API URL is required. Set VKTEAMS_BOT_API_URL or configure via 'vkteams-bot-cli setup'"
                .to_string(),
        )
    })?;

    // Set environment variables for bot initialization
    setup_bot_environment(config);

    Bot::with_params(&APIVersionUrl::V1, token.as_str(), url.as_str()).map_err(CliError::ApiError)
}

/// Create a dummy bot instance for commands that don't need real API access
///
/// # Returns
/// * A dummy Bot instance (should not be used for actual API calls)
pub fn create_dummy_bot() -> Bot {
    // Create a dummy bot for commands that don't need real API access
    // This is safe because those commands won't actually use the bot
    Bot::with_params(&APIVersionUrl::V1, "dummy_token", "https://dummy.api.com").unwrap_or_else(
        |_| {
            // If even dummy bot creation fails, we'll handle it in the command execution
            panic!("Failed to create dummy bot - this should not happen")
        },
    )
}

/// Check if a command needs a real bot instance for execution
///
/// # Arguments
/// * `command` - The command to check
///
/// # Returns
/// * `true` if the command needs a real bot instance
/// * `false` if the command can work with a dummy bot or no bot at all
pub fn needs_bot_instance(command: &Commands) -> bool {
    match command {
        Commands::Config(_) => false,
        Commands::Diagnostic(crate::commands::diagnostic::DiagnosticCommands::SystemInfo) => false,
        Commands::Diagnostic(_) => true,
        Commands::Files(_) => true, // File operations need real bot for API calls
        Commands::Storage(storage_cmd) => {
            match storage_cmd {
                // Database operations don't need bot
                crate::commands::storage::StorageCommands::Database { .. } => false,
                // Search operations don't need bot (they use storage directly)
                crate::commands::storage::StorageCommands::Search { .. } => false,
                // Context operations don't need bot
                crate::commands::storage::StorageCommands::Context { .. } => false,
            }
        }
        _ => true,
    }
}

/// Setup environment variables for bot initialization
///
/// # Arguments
/// * `config` - The configuration to use for setting up environment
pub fn setup_bot_environment(config: &Config) {
    if let Some(token) = &config.api.token {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_API_TOKEN", token);
        }
    }

    if let Some(url) = &config.api.url {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_API_URL", url);
        }
    }

    if let Some(proxy) = &config.proxy {
        unsafe {
            std::env::set_var("VKTEAMS_PROXY", &proxy.url);
        }

        if let Some(user) = &proxy.user {
            unsafe {
                std::env::set_var("VKTEAMS_PROXY_USER", user);
            }
        }

        if let Some(password) = &proxy.password {
            unsafe {
                std::env::set_var("VKTEAMS_PROXY_PASSWORD", password);
            }
        }
    }
}

/// Test bot connectivity with a simple API call
///
/// # Arguments
/// * `bot` - The bot instance to test
///
/// # Returns
/// * `Ok(())` if the bot can successfully make API calls
/// * `Err(CliError)` if the bot connectivity test fails
pub async fn test_bot_connectivity(bot: &Bot) -> CliResult<()> {
    let request = RequestSelfGet::new(());
    bot.send_api_request(request)
        .await
        .map_err(CliError::ApiError)
        .map(|_| ())
}

/// Create bot instance with retry logic
///
/// # Arguments
/// * `config` - The configuration to use
/// * `max_retries` - Maximum number of retry attempts
///
/// # Returns
/// * `Ok(Bot)` if bot creation succeeds
/// * `Err(CliError)` if bot creation fails after all retries
pub fn create_bot_instance_with_retry(config: &Config, max_retries: u32) -> CliResult<Bot> {
    let mut last_error = None;

    for attempt in 0..=max_retries {
        match create_bot_instance(config) {
            Ok(bot) => return Ok(bot),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries {
                    // Add a small delay between retries
                    std::thread::sleep(std::time::Duration::from_millis(
                        100 * (attempt + 1) as u64,
                    ));
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        CliError::UnexpectedError("Failed to create bot instance after retries".to_string())
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio::runtime::Runtime;

    #[test]
    fn test_needs_bot_instance() {
        // Config commands should not need bot
        let config_cmd = Commands::Config(crate::commands::config::ConfigCommands::Setup);
        assert!(!needs_bot_instance(&config_cmd));

        // System info should not need bot
        let system_info_cmd =
            Commands::Diagnostic(crate::commands::diagnostic::DiagnosticCommands::SystemInfo);
        assert!(!needs_bot_instance(&system_info_cmd));
    }

    #[test]
    fn test_validate_config() {
        let mut config = toml::from_str("").unwrap();
        println!("{config:?}");

        // Empty config should fail
        assert!(crate::utils::config_helpers::validate_config(&config).is_err());

        // Config with only token should fail
        config.api.token = Some("test_token_12345".to_string());
        assert!(crate::utils::config_helpers::validate_config(&config).is_err());

        // Config with token and URL should pass
        config.api.url = Some("https://example.com".to_string());
        assert!(
            crate::utils::config_helpers::validate_config(&config)
                .map_err(|e| eprintln!("{e}"))
                .is_ok()
        );

        // Invalid URL should fail
        config.api.url = Some("invalid-url".to_string());
        assert!(crate::utils::config_helpers::validate_config(&config).is_err());

        // Short token should fail
        config.api.token = Some("short".to_string());
        config.api.url = Some("https://example.com".to_string());
        assert!(crate::utils::config_helpers::validate_config(&config).is_err());
    }

    #[test]
    fn test_create_dummy_bot() {
        let _dummy_bot = create_dummy_bot();
        // We can't test much about the dummy bot without making API calls
        // But we can verify it was created without panicking
        // This test passes if create_dummy_bot() doesn't panic
    }

    #[test]
    fn test_setup_bot_environment_sets_vars() {
        let mut config: crate::config::Config = toml::from_str("").unwrap();
        config.api.token = Some("test_token_env".to_string());
        config.api.url = Some("https://api.example.com".to_string());
        config.proxy = Some(crate::config::ProxyConfig {
            url: "http://proxy.example.com".to_string(),
            user: Some("user1".to_string()),
            password: Some("pass1".to_string()),
        });
        setup_bot_environment(&config);
        assert_eq!(env::var("VKTEAMS_BOT_API_TOKEN").unwrap(), "test_token_env");
        assert_eq!(
            env::var("VKTEAMS_BOT_API_URL").unwrap(),
            "https://api.example.com"
        );
        assert_eq!(
            env::var("VKTEAMS_PROXY").unwrap(),
            "http://proxy.example.com"
        );
        assert_eq!(env::var("VKTEAMS_PROXY_USER").unwrap(), "user1");
        assert_eq!(env::var("VKTEAMS_PROXY_PASSWORD").unwrap(), "pass1");
    }

    #[test]
    fn test_create_bot_instance_with_retry_error() {
        let config: crate::config::Config = toml::from_str("").unwrap();
        // Нет токена и url — всегда ошибка
        let res = create_bot_instance_with_retry(&config, 2);
        assert!(res.is_err());
    }

    #[test]
    fn test_create_bot_instance_with_retry_success() {
        let mut config: crate::config::Config = toml::from_str("").unwrap();
        config.api.token = Some("test_token_12345".to_string());
        config.api.url = Some("https://example.com".to_string());
        let res = create_bot_instance_with_retry(&config, 0);
        // Может быть Ok или Err (если url невалидный), главное — не паникует
        let _ = res;
    }

    #[test]
    fn test_create_bot_instance_with_retry_max_retries() {
        let config: crate::config::Config = toml::from_str("").unwrap();
        let res = create_bot_instance_with_retry(&config, 3);
        assert!(res.is_err());
    }

    #[test]
    fn test_test_bot_connectivity_api_error() {
        let bot =
            Bot::with_params(&APIVersionUrl::V1, "dummy_token", "https://dummy.api.com").unwrap();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(test_bot_connectivity(&bot));
        assert!(res.is_err());
    }
}
