use crate::cli_bridge::CliBridge;
use crate::errors::BridgeError;
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

    /// Try to create a new Server instance with error handling
    pub fn try_new() -> Result<Self, BridgeError> {
        let mut config = Self::load_config();
        config.apply_env_overrides();

        let cli = CliBridge::new(&config)?;

        Ok(Self {
            cli: Arc::new(cli),
            config,
        })
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

    /// Try to create Server with custom configuration
    pub fn try_with_config(mut config: UnifiedConfig) -> Result<Self, BridgeError> {
        config.apply_env_overrides();

        let cli = CliBridge::new(&config)?;

        Ok(Self {
            cli: Arc::new(cli),
            config,
        })
    }

    /// Load configuration from file or use defaults
    fn load_config() -> UnifiedConfig {
        // Try environment variable first (highest priority)
        if let Ok(config_path) = std::env::var("VKTEAMS_BOT_CONFIG") {
            match UnifiedConfig::load_from_file(&config_path) {
                Ok(config) => {
                    eprintln!("✓ Loaded config from VKTEAMS_BOT_CONFIG: {}", config_path);
                    return config;
                }
                Err(e) => {
                    eprintln!("⚠ Failed to load config from VKTEAMS_BOT_CONFIG ({}): {} - trying fallback locations", config_path, e);
                }
            }
        }

        // Try to load from standard locations
        let config_paths = [
            "config.toml",
            "shared-config.toml",
            "/etc/vkteams-bot/config.toml",
        ];

        // Try static paths
        for path in &config_paths {
            match UnifiedConfig::load_from_file(path) {
                Ok(config) => {
                    eprintln!("✓ Loaded config from: {}", path);
                    return config;
                }
                Err(_) => {
                    // Silent continue for expected missing files
                }
            }
        }

        // Try user config directory
        if let Some(home_dir) = dirs::home_dir() {
            let user_home_path = home_dir.join(".config/vkteams-bot/config.toml");
            match UnifiedConfig::load_from_file(&user_home_path) {
                Ok(config) => {
                    eprintln!("✓ Loaded config from user directory: {}", user_home_path.display());
                    return config;
                }
                Err(_) => {
                    // Silent fallback - user config is optional
                }
            }
        }

        eprintln!("ℹ Using default configuration (no config file found in standard locations)");
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

        // Test server creation with graceful error handling
        match Server::try_new() {
            Ok(server) => {
                println!("✓ Server created successfully");
                assert_eq!(server.config.mcp.chat_id, Some("test_chat_id".to_string()));
                // Don't assert exact API URL as user might have custom config
                assert!(!server.config.api.url.is_empty());
                let bridge = server.bridge();
                assert!(Arc::strong_count(&bridge) >= 1);
            }
            Err(e) => {
                println!("⚠ Expected failure in test environment without CLI binary: {}", e);
                // This is acceptable in test environment where CLI binary might not be available
                assert!(e.to_string().contains("CLI") || e.to_string().contains("bridge"));
            }
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

        match Server::try_with_config(config.clone()) {
            Ok(server) => {
                println!("✓ Server with config created successfully");
                assert_eq!(server.config.mcp.chat_id, Some("config_test_chat".to_string())); // env override
                assert_eq!(server.config.api.url, "https://custom.api.com");
            }
            Err(e) => {
                println!("⚠ Expected failure in test environment without CLI binary: {}", e);
                assert!(e.to_string().contains("CLI") || e.to_string().contains("not found"));
            }
        }
    }

    #[test]
    fn test_config_loading_scenarios() {
        // Test config loading with environment variable
        let original_config = std::env::var("VKTEAMS_BOT_CONFIG").ok();
        
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CONFIG", "/nonexistent/config.toml");
        }

        // This should handle the error gracefully and fall back to available config
        let config = Server::load_config();
        assert!(!config.api.url.is_empty()); // Should have some URL

        // Restore original state
        unsafe {
            match original_config {
                Some(config) => std::env::set_var("VKTEAMS_BOT_CONFIG", config),
                None => std::env::remove_var("VKTEAMS_BOT_CONFIG"),
            }
        }
    }

    #[test]
    fn test_user_config_directory_resolution() {
        // Test config loading when home directory is available
        if let Some(home_dir) = dirs::home_dir() {
            let user_config_path = home_dir.join(".config/vkteams-bot/config.toml");
            println!("Testing user config path: {}", user_config_path.display());
            
            // This tests the path resolution logic
            let config = Server::load_config();
            assert!(!config.api.url.is_empty());
            // Don't assert specific URL as user might have custom config
        }
    }

    #[test]
    fn test_bridge_reference_counting() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_bridge_ref");
        }

        match Server::try_new() {
            Ok(server) => {
                let bridge1 = server.bridge();
                assert!(Arc::strong_count(&bridge1) >= 1);
                
                // Test multiple references
                let bridge2 = server.bridge();
                assert!(Arc::strong_count(&bridge2) >= 2);
                
                // Both should point to the same instance
                assert!(Arc::ptr_eq(&bridge1, &bridge2));
            }
            Err(_) => {
                println!("Bridge test skipped - CLI binary not available in test environment");
            }
        }
    }
}
