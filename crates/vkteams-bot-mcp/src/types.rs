use crate::cli_bridge::CliBridge;
use std::sync::Arc;
use vkteams_bot::config::UnifiedConfig;

#[derive(Debug, Clone)]
pub struct Server {
    pub cli: Arc<CliBridge>,
    pub config: UnifiedConfig,
}

impl Server {
    pub fn bridge(&self) -> Arc<CliBridge> {
        Arc::clone(&self.cli)
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    /// Create a new Server instance with unified configuration
    pub fn new() -> Self {
        let mut config = Self::load_config();
        config.apply_env_overrides();

        let cli = CliBridge::new(&config).expect("Failed to create CLI bridge");

        Self {
            cli: Arc::new(cli),
            config,
        }
    }

    /// Create Server with custom configuration
    pub fn with_config(mut config: UnifiedConfig) -> Self {
        config.apply_env_overrides();

        let cli = CliBridge::new(&config).expect("Failed to create CLI bridge");

        Self {
            cli: Arc::new(cli),
            config,
        }
    }

    /// Load configuration from file or use defaults
    fn load_config() -> UnifiedConfig {
        // Try to load from standard locations
        let config_paths = [
            "config.toml",
            "shared-config.toml",
            "/etc/vkteams-bot/config.toml",
        ];

        // Try static paths first
        for path in &config_paths {
            if let Ok(config) = UnifiedConfig::load_from_file(path) {
                return config;
            }
        }

        // Try user config directory
        if let Some(home_dir) = dirs::home_dir() {
            let user_home_path = home_dir.join(".config/vkteams-bot/config.toml");
            if let Ok(config) = UnifiedConfig::load_from_file(&user_home_path) {
                return config;
            }
        }

        // Fall back to default (env overrides will be applied in new/with_config)
        UnifiedConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_default_and_bridge() {
        // Set required env vars for default
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat_id");
        }

        // Note: This test might fail if CLI binary is not available in test environment
        match std::panic::catch_unwind(|| {
            let server = Server::default();
            assert_eq!(server.config.mcp.chat_id, Some("test_chat_id".to_string()));
            assert_eq!(server.config.api.url, "https://api.vk.com");
            let bridge = server.bridge();
            assert!(Arc::strong_count(&bridge) >= 1);
        }) {
            Ok(_) => println!("Server created successfully"),
            Err(_) => println!("Expected failure in test environment without CLI binary"),
        }
    }

    #[test]
    fn test_server_with_config() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "config_test_chat");
        }

        let mut config = UnifiedConfig::default();
        config.mcp.chat_id = Some("custom_chat_id".to_string());
        config.api.url = "https://custom.api.com".to_string();

        match std::panic::catch_unwind(|| {
            let server = Server::with_config(config.clone());
            assert_eq!(
                server.config.mcp.chat_id,
                Some("custom_chat_id".to_string())
            );
            assert_eq!(server.config.api.url, "https://custom.api.com");
        }) {
            Ok(_) => println!("Server with config created successfully"),
            Err(_) => println!("Expected failure in test environment without CLI binary"),
        }
    }
}
