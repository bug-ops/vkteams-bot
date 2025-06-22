//! Unified configuration for CLI and MCP components
//!
//! This module provides shared configuration structures that can be used
//! by both the CLI and MCP server components, ensuring consistency.

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::path::PathBuf;

/// Unified configuration structure for both CLI and MCP
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct UnifiedConfig {
    /// API configuration shared by CLI and MCP
    #[serde(default)]
    pub api: ApiConfig,

    /// Storage configuration (database and vector search)
    #[cfg(feature = "storage")]
    #[serde(default)]
    pub storage: StorageConfig,

    /// MCP server specific configuration
    #[serde(default)]
    pub mcp: McpConfig,

    /// CLI specific configuration
    #[serde(default)]
    pub cli: CliConfig,

    /// Network configuration shared by both components
    #[serde(default)]
    pub network: NetworkConfig,

    /// OpenTelemetry configuration
    #[cfg(feature = "otlp")]
    #[serde(default)]
    pub otlp: OtlpConfig,
}

/// API configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ApiConfig {
    /// VK Teams Bot API token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// API base URL
    #[serde(default = "default_api_url")]
    pub url: String,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum retry attempts
    #[serde(default = "default_retries")]
    pub max_retries: u32,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            token: None,
            url: default_api_url(),
            timeout: default_timeout(),
            max_retries: default_retries(),
        }
    }
}

/// Storage configuration
#[cfg(feature = "storage")]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub struct StorageConfig {
    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,

    /// Vector embedding configuration
    #[cfg(feature = "vector-search")]
    #[serde(default)]
    pub embedding: EmbeddingConfig,
}

/// Database configuration
#[cfg(feature = "storage")]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct DatabaseConfig {
    /// Database connection URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Maximum number of connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Whether to auto-migrate on startup
    #[serde(default = "default_auto_migrate")]
    pub auto_migrate: bool,
}

#[cfg(feature = "storage")]
impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: None,
            max_connections: default_max_connections(),
            auto_migrate: default_auto_migrate(),
        }
    }
}

/// Vector embedding configuration
#[cfg(all(feature = "storage", feature = "vector-search"))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct EmbeddingConfig {
    /// Embedding provider (ollama, openai)
    #[serde(default = "default_embedding_provider")]
    pub provider: String,

    /// Model name
    #[serde(default = "default_embedding_model")]
    pub model: String,

    /// API endpoint (for ollama or custom endpoints)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,

    /// API key (for OpenAI and similar services)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

#[cfg(all(feature = "storage", feature = "vector-search"))]
impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: default_embedding_provider(),
            model: default_embedding_model(),
            endpoint: None,
            api_key: None,
        }
    }
}

/// MCP server configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct McpConfig {
    /// Default chat ID for MCP operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<String>,

    /// Path to CLI binary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli_path: Option<PathBuf>,

    /// Enable storage tools in MCP
    #[serde(default = "default_enable_storage_tools")]
    pub enable_storage_tools: bool,

    /// Enable file tools in MCP
    #[serde(default = "default_enable_file_tools")]
    pub enable_file_tools: bool,

    /// Maximum retry attempts for CLI calls
    #[serde(default = "default_cli_retries")]
    pub cli_retries: usize,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            chat_id: None,
            cli_path: None,
            enable_storage_tools: default_enable_storage_tools(),
            enable_file_tools: default_enable_file_tools(),
            cli_retries: default_cli_retries(),
        }
    }
}

/// CLI specific configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub struct CliConfig {
    /// File handling configuration
    #[serde(default)]
    pub files: FileConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,

    /// UI configuration
    #[serde(default)]
    pub ui: UiConfig,
}

/// File handling configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct FileConfig {
    /// Download directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_dir: Option<PathBuf>,

    /// Upload directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload_dir: Option<PathBuf>,

    /// Maximum file size in bytes
    #[serde(default = "default_max_file_size")]
    pub max_file_size: usize,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            download_dir: None,
            upload_dir: None,
            max_file_size: default_max_file_size(),
        }
    }
}

/// Logging configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct LoggingConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log format
    #[serde(default = "default_log_format")]
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}

/// UI configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct UiConfig {
    /// Show progress bars
    #[serde(default = "default_show_progress")]
    pub show_progress: bool,

    /// Use colored output
    #[serde(default = "default_colored_output")]
    pub colored_output: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_progress: default_show_progress(),
            colored_output: default_colored_output(),
        }
    }
}

/// Network configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct NetworkConfig {
    /// Number of retry attempts
    #[serde(default = "default_retries_usize")]
    pub retries: usize,

    /// Request timeout in seconds
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,

    /// Connection timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            retries: default_retries_usize(),
            request_timeout_secs: default_request_timeout(),
            connect_timeout_secs: default_connect_timeout(),
        }
    }
}

/// OpenTelemetry configuration
#[cfg(feature = "otlp")]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct OtlpConfig {
    /// Instance ID
    #[serde(default = "default_instance_id")]
    pub instance_id: Cow<'static, str>,

    /// Deployment environment
    #[serde(default = "default_environment")]
    pub deployment_environment_name: Cow<'static, str>,

    /// OTLP exporter endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exporter_endpoint: Option<String>,
}

#[cfg(feature = "otlp")]
impl Default for OtlpConfig {
    fn default() -> Self {
        Self {
            instance_id: default_instance_id(),
            deployment_environment_name: default_environment(),
            exporter_endpoint: None,
        }
    }
}

// Default value functions
fn default_api_url() -> String {
    "https://api.vk.com".to_string()
}

fn default_timeout() -> u64 {
    30
}

fn default_retries() -> u32 {
    3
}

fn default_retries_usize() -> usize {
    3
}

#[cfg(feature = "storage")]
fn default_max_connections() -> u32 {
    20
}

#[cfg(feature = "storage")]
fn default_auto_migrate() -> bool {
    true
}

#[cfg(all(feature = "storage", feature = "vector-search"))]
fn default_embedding_provider() -> String {
    "ollama".to_string()
}

#[cfg(all(feature = "storage", feature = "vector-search"))]
fn default_embedding_model() -> String {
    "nomic-embed-text".to_string()
}

fn default_enable_storage_tools() -> bool {
    true
}

fn default_enable_file_tools() -> bool {
    true
}

fn default_cli_retries() -> usize {
    3
}

fn default_max_file_size() -> usize {
    104_857_600 // 100MB
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "pretty".to_string()
}

fn default_show_progress() -> bool {
    true
}

fn default_colored_output() -> bool {
    true
}

fn default_request_timeout() -> u64 {
    30
}

fn default_connect_timeout() -> u64 {
    10
}

#[cfg(feature = "otlp")]
fn default_instance_id() -> Cow<'static, str> {
    Cow::Borrowed("bot")
}

#[cfg(feature = "otlp")]
fn default_environment() -> Cow<'static, str> {
    Cow::Borrowed("production")
}

impl UnifiedConfig {
    /// Load configuration from file or environment variables
    pub fn load_from_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let mut config: UnifiedConfig = toml::from_str(&content)?;

        // Override with environment variables
        config.apply_env_overrides();

        Ok(config)
    }

    /// Apply environment variable overrides
    pub fn apply_env_overrides(&mut self) {
        // API configuration
        if let Ok(token) = std::env::var("VKTEAMS_BOT_API_TOKEN") {
            self.api.token = Some(token);
        }
        if let Ok(url) = std::env::var("VKTEAMS_BOT_API_URL") {
            self.api.url = url;
        }

        // MCP configuration
        if let Ok(chat_id) = std::env::var("VKTEAMS_BOT_CHAT_ID") {
            self.mcp.chat_id = Some(chat_id);
        }
        if let Ok(cli_path) = std::env::var("VKTEAMS_BOT_CLI_PATH") {
            self.mcp.cli_path = Some(PathBuf::from(cli_path));
        }

        // Storage configuration
        #[cfg(feature = "storage")]
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            self.storage.database.url = Some(db_url);
        }

        // Embedding configuration
        #[cfg(all(feature = "storage", feature = "vector-search"))]
        {
            if let Ok(provider) = std::env::var("EMBEDDING_PROVIDER") {
                self.storage.embedding.provider = provider;
            }
            if let Ok(model) = std::env::var("EMBEDDING_MODEL") {
                self.storage.embedding.model = model;
            }
            if let Ok(endpoint) = std::env::var("EMBEDDING_ENDPOINT") {
                self.storage.embedding.endpoint = Some(endpoint);
            }
            if let Ok(api_key) = std::env::var("EMBEDDING_API_KEY") {
                self.storage.embedding.api_key = Some(api_key);
            }
        }
    }

    /// Create a default configuration file
    pub fn create_default_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config = UnifiedConfig::default();
        let content = toml::to_string_pretty(&config)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = UnifiedConfig::default();
        assert_eq!(config.api.url, "https://api.vk.com");
        assert_eq!(config.api.timeout, 30);
        assert_eq!(config.network.retries, 3);
        assert!(config.mcp.enable_storage_tools);
        assert!(config.cli.ui.show_progress);
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = UnifiedConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: UnifiedConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_env_overrides() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_API_TOKEN", "test_token");
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat");
        }

        let mut config = UnifiedConfig::default();
        config.apply_env_overrides();

        assert_eq!(config.api.token, Some("test_token".to_string()));
        assert_eq!(config.mcp.chat_id, Some("test_chat".to_string()));

        // Cleanup
        unsafe {
            std::env::remove_var("VKTEAMS_BOT_API_TOKEN");
            std::env::remove_var("VKTEAMS_BOT_CHAT_ID");
        }
    }

    #[test]
    fn test_api_config_default() {
        let api_config = ApiConfig::default();
        assert_eq!(api_config.url, "https://api.vk.com");
        assert_eq!(api_config.timeout, 30);
        assert!(api_config.token.is_none());
        assert_eq!(api_config.max_retries, 3);
    }

    #[test]
    fn test_network_config_default() {
        let network_config = NetworkConfig::default();
        assert_eq!(network_config.retries, 3);
        assert_eq!(network_config.request_timeout_secs, 30);
        assert_eq!(network_config.connect_timeout_secs, 10);
    }

    #[test]
    fn test_mcp_config_default() {
        let mcp_config = McpConfig::default();
        assert!(mcp_config.chat_id.is_none());
        assert!(mcp_config.cli_path.is_none());
        assert!(mcp_config.enable_storage_tools);
        assert!(mcp_config.enable_file_tools);
        assert_eq!(mcp_config.cli_retries, 3);
    }

    #[test]
    fn test_cli_config_default() {
        let cli_config = CliConfig::default();
        assert!(cli_config.ui.show_progress);
    }

    #[test]
    fn test_api_config_custom() {
        let api_config = ApiConfig {
            url: "https://custom.api.com".to_string(),
            token: Some("custom_token".to_string()),
            timeout: 60,
            max_retries: 5,
        };

        assert_eq!(api_config.url, "https://custom.api.com");
        assert_eq!(api_config.token, Some("custom_token".to_string()));
        assert_eq!(api_config.timeout, 60);
        assert_eq!(api_config.max_retries, 5);
    }

    #[test]
    fn test_mcp_config_custom() {
        let mcp_config = McpConfig {
            chat_id: Some("custom_chat".to_string()),
            cli_path: Some(PathBuf::from("/usr/bin/cli")),
            enable_storage_tools: false,
            enable_file_tools: false,
            cli_retries: 5,
        };

        assert_eq!(mcp_config.chat_id, Some("custom_chat".to_string()));
        assert_eq!(mcp_config.cli_path, Some(PathBuf::from("/usr/bin/cli")));
        assert!(!mcp_config.enable_storage_tools);
        assert!(!mcp_config.enable_file_tools);
        assert_eq!(mcp_config.cli_retries, 5);
    }

    #[test]
    fn test_config_serialization_with_custom_values() {
        let mut config = UnifiedConfig::default();
        config.api.url = "https://custom.com".to_string();
        config.api.token = Some("secret".to_string());
        config.mcp.chat_id = Some("chat123".to_string());
        config.network.retries = 5;

        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("https://custom.com"));
        assert!(serialized.contains("secret"));
        assert!(serialized.contains("chat123"));
        assert!(serialized.contains("retries = 5"));

        let deserialized: UnifiedConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_env_override_partial() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_API_TOKEN", "env_token");
            // Don't set VKTEAMS_BOT_CHAT_ID to test partial override
        }

        let mut config = UnifiedConfig::default();
        config.mcp.chat_id = Some("original_chat".to_string());
        config.apply_env_overrides();

        // Token should be overridden
        assert_eq!(config.api.token, Some("env_token".to_string()));
        // Chat ID should remain original since env var not set
        assert_eq!(config.mcp.chat_id, Some("original_chat".to_string()));

        // Cleanup
        unsafe {
            std::env::remove_var("VKTEAMS_BOT_API_TOKEN");
        }
    }

    #[test]
    fn test_default_functions() {
        // Test the key default functions
        assert_eq!(default_api_url(), "https://api.vk.com");
        assert_eq!(default_timeout(), 30);
        assert_eq!(default_retries(), 3);
        assert!(default_enable_storage_tools());
        assert!(default_enable_file_tools());
        assert_eq!(default_cli_retries(), 3);
    }

    #[test]
    fn test_file_config_default() {
        let file_config = FileConfig::default();
        assert_eq!(file_config.max_file_size, 104_857_600); // 100MB
    }

    #[test]
    fn test_logging_config_default() {
        let logging_config = LoggingConfig::default();
        assert_eq!(logging_config.level, "info");
        assert_eq!(logging_config.format, "pretty");
    }

    #[test]
    fn test_ui_config_default() {
        let ui_config = UiConfig::default();
        assert!(ui_config.show_progress);
    }

    #[test]
    fn test_config_equality() {
        let config1 = UnifiedConfig::default();
        let config2 = UnifiedConfig::default();
        assert_eq!(config1, config2);

        let mut config3 = UnifiedConfig::default();
        config3.api.timeout = 60;
        assert_ne!(config1, config3);
    }

    #[test]
    fn test_config_cloning() {
        let mut original = UnifiedConfig::default();
        original.api.token = Some("test_token".to_string());
        original.mcp.chat_id = Some("test_chat".to_string());

        let cloned = original.clone();
        assert_eq!(original, cloned);
        assert_eq!(cloned.api.token, Some("test_token".to_string()));
        assert_eq!(cloned.mcp.chat_id, Some("test_chat".to_string()));
    }

    #[test]
    fn test_config_debug_format() {
        let config = UnifiedConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("UnifiedConfig"));
        assert!(debug_str.contains("ApiConfig"));
        assert!(debug_str.contains("NetworkConfig"));
        assert!(debug_str.contains("McpConfig"));
        assert!(debug_str.contains("CliConfig"));
    }
}
