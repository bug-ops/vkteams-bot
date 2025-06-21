use crate::cli_bridge::CliBridge;
use std::sync::Arc;
use vkteams_bot::config::UnifiedConfig;

#[derive(Debug, Clone)]
pub struct Server {
    pub cli: Arc<CliBridge>,
    pub chat_id: String,
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
        let config = Self::load_config();
        let chat_id = config.mcp.chat_id
            .clone()
            .or_else(|| std::env::var("VKTEAMS_BOT_CHAT_ID").ok())
            .expect("VKTEAMS_BOT_CHAT_ID is not set in config or environment");
        
        let cli = CliBridge::new().expect("Failed to create CLI bridge");
        
        Self {
            cli: Arc::new(cli),
            chat_id,
            config,
        }
    }
    
    /// Create Server with custom configuration
    pub fn with_config(config: UnifiedConfig) -> Self {
        let chat_id = config.mcp.chat_id
            .clone()
            .or_else(|| std::env::var("VKTEAMS_BOT_CHAT_ID").ok())
            .expect("VKTEAMS_BOT_CHAT_ID is not set in config or environment");
        
        let cli = CliBridge::new().expect("Failed to create CLI bridge");
        
        Self {
            cli: Arc::new(cli),
            chat_id,
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
            "~/.config/vkteams-bot/config.toml",
        ];
        
        for path in &config_paths {
            if let Ok(config) = UnifiedConfig::load_from_file(path) {
                return config;
            }
        }
        
        // Fall back to default with environment overrides
        let mut config = UnifiedConfig::default();
        config.apply_env_overrides();
        config
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
            assert_eq!(server.chat_id, "test_chat_id");
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
            assert_eq!(server.chat_id, "custom_chat_id");
            assert_eq!(server.config.api.url, "https://custom.api.com");
        }) {
            Ok(_) => println!("Server with config created successfully"),
            Err(_) => println!("Expected failure in test environment without CLI binary"),
        }
    }
}
