use crate::errors::prelude::{CliError, Result as CliResult};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use toml;

// Constants for configuration
pub const CONFIG_FILE_NAME: &str = "vkteams_bot_config.toml";
pub const DEFAULT_CONFIG_DIR: &str = ".config/vkteams-bot";
pub const ENV_PREFIX: &str = "VKTEAMS_";

/// Configuration structure for VK Teams Bot CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// API Configuration
    #[serde(default)]
    pub api: ApiConfig,

    /// File handling configuration
    #[serde(default)]
    pub files: FileConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Proxy configuration
    #[serde(default)]
    pub proxy: Option<ProxyConfig>,
}

/// API Configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API token for VK Teams Bot
    pub token: Option<String>,

    /// Base URL for API requests
    pub url: Option<String>,

    /// Timeout for API requests in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum number of retries for API requests
    #[serde(default = "default_retries")]
    pub max_retries: u32,
}

/// File handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    /// Default directory for downloads
    pub download_dir: Option<String>,

    /// Default directory for uploads
    pub upload_dir: Option<String>,

    /// Maximum file size in bytes for uploads and downloads
    #[serde(default = "default_max_file_size")]
    pub max_file_size: usize,

    /// Buffer size in bytes for file streaming
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (error, warn, info, debug, trace)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Output format (json, text)
    #[serde(default = "default_log_format")]
    pub format: String,

    /// Enable or disable color output
    #[serde(default = "default_log_colors")]
    pub colors: bool,
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Proxy URL
    pub url: String,

    /// Proxy user (if authentication is required)
    pub user: Option<String>,

    /// Proxy password (if authentication is required)
    pub password: Option<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            token: None,
            url: None,
            timeout: default_timeout(),
            max_retries: default_retries(),
        }
    }
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            download_dir: None,
            upload_dir: None,
            max_file_size: default_max_file_size(),
            buffer_size: default_buffer_size(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            colors: default_log_colors(),
        }
    }
}

// Default values functions
fn default_timeout() -> u64 {
    30
}

fn default_retries() -> u32 {
    3
}

fn default_max_file_size() -> usize {
    100 * 1024 * 1024 // 100MB
}

fn default_buffer_size() -> usize {
    64 * 1024 // 64KB
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "text".to_string()
}

fn default_log_colors() -> bool {
    true
}

impl Config {
    /// Load configuration from all available sources
    pub fn load() -> CliResult<Self> {
        let mut config = Config::default();

        // Try to load from config file
        if let Ok(file_config) = Self::from_file() {
            config = Self::merge_configs(config, file_config);
        }

        // Overlay with environment variables
        let env_config = Self::from_env()?;
        config = Self::merge_configs(config, env_config);

        Ok(config)
    }

    /// Load configuration from default locations
    pub fn from_file() -> CliResult<Self> {
        let config_paths = Self::get_config_paths();

        for path in config_paths {
            if path.exists() {
                return Self::from_path(&path);
            }
        }

        // Return default config if no file found
        Ok(Config::default())
    }

    /// Load configuration from a specific path
    pub fn from_path(path: &Path) -> CliResult<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| CliError::FileError(format!("Could not read config file: {}", e)))?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            CliError::UnexpectedError(format!("Could not parse config file: {}", e))
        })?;

        Ok(config)
    }

    /// Get a list of possible config file paths in order of preference
    fn get_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Current directory
        paths.push(PathBuf::from(CONFIG_FILE_NAME));

        // User config directory
        if let Some(home_dir) = dirs::home_dir() {
            let mut user_config = home_dir;
            user_config.push(DEFAULT_CONFIG_DIR);
            user_config.push(CONFIG_FILE_NAME);
            paths.push(user_config);
        }

        // System config directory
        #[cfg(unix)]
        {
            let mut system_config = PathBuf::from("/etc");
            system_config.push("vkteams-bot");
            system_config.push(CONFIG_FILE_NAME);
            paths.push(system_config);
        }

        paths
    }

    /// Save configuration to file
    pub fn save(&self, path: Option<&Path>) -> CliResult<()> {
        let path = match path {
            Some(p) => p.to_owned(),
            None => {
                let mut p = dirs::home_dir().ok_or_else(|| {
                    CliError::FileError("Could not determine home directory".to_string())
                })?;
                p.push(DEFAULT_CONFIG_DIR);
                fs::create_dir_all(&p).map_err(|e| {
                    CliError::FileError(format!("Could not create config directory: {}", e))
                })?;
                p.push(CONFIG_FILE_NAME);
                p
            }
        };

        let content = toml::to_string_pretty(self)
            .map_err(|e| CliError::UnexpectedError(format!("Could not serialize config: {}", e)))?;

        fs::write(&path, content)
            .map_err(|e| CliError::FileError(format!("Could not write config file: {}", e)))?;

        Ok(())
    }

    /// Load configuration from environment variables
    pub fn from_env() -> CliResult<Self> {
        let mut config = Config::default();

        // API config
        if let Ok(token) = env::var(format!("{}BOT_API_TOKEN", ENV_PREFIX)) {
            config.api.token = Some(token);
        }

        if let Ok(url) = env::var(format!("{}BOT_API_URL", ENV_PREFIX)) {
            config.api.url = Some(url);
        }

        if let Ok(timeout) = env::var(format!("{}TIMEOUT", ENV_PREFIX)).map(|v| v.parse::<u64>()) {
            if let Ok(timeout) = timeout {
                config.api.timeout = timeout;
            }
        }

        // File config
        if let Ok(download_dir) = env::var(format!("{}DOWNLOAD_DIR", ENV_PREFIX)) {
            config.files.download_dir = Some(download_dir);
        }

        if let Ok(upload_dir) = env::var(format!("{}UPLOAD_DIR", ENV_PREFIX)) {
            config.files.upload_dir = Some(upload_dir);
        }

        if let Ok(max_file_size) =
            env::var(format!("{}MAX_FILE_SIZE", ENV_PREFIX)).map(|v| v.parse::<usize>())
        {
            if let Ok(max_file_size) = max_file_size {
                config.files.max_file_size = max_file_size;
            }
        }

        // Logging config
        if let Ok(level) = env::var(format!("{}LOG_LEVEL", ENV_PREFIX)) {
            config.logging.level = level;
        }

        if let Ok(format) = env::var(format!("{}LOG_FORMAT", ENV_PREFIX)) {
            config.logging.format = format;
        }

        if let Ok(colors) = env::var(format!("{}LOG_COLORS", ENV_PREFIX)).map(|v| v.parse::<bool>())
        {
            if let Ok(colors) = colors {
                config.logging.colors = colors;
            }
        }

        // Proxy config
        if let Ok(proxy_url) = env::var(format!("{}PROXY", ENV_PREFIX)) {
            config.proxy = Some(ProxyConfig {
                url: proxy_url,
                user: env::var(format!("{}PROXY_USER", ENV_PREFIX)).ok(),
                password: env::var(format!("{}PROXY_PASSWORD", ENV_PREFIX)).ok(),
            });
        }

        Ok(config)
    }

    /// Merge two configurations, with the second taking precedence
    fn merge_configs(base: Self, overlay: Self) -> Self {
        Self {
            api: ApiConfig {
                token: overlay.api.token.or(base.api.token),
                url: overlay.api.url.or(base.api.url),
                timeout: if overlay.api.timeout != default_timeout() {
                    overlay.api.timeout
                } else {
                    base.api.timeout
                },
                max_retries: if overlay.api.max_retries != default_retries() {
                    overlay.api.max_retries
                } else {
                    base.api.max_retries
                },
            },
            files: FileConfig {
                download_dir: overlay.files.download_dir.or(base.files.download_dir),
                upload_dir: overlay.files.upload_dir.or(base.files.upload_dir),
                max_file_size: if overlay.files.max_file_size != default_max_file_size() {
                    overlay.files.max_file_size
                } else {
                    base.files.max_file_size
                },
                buffer_size: if overlay.files.buffer_size != default_buffer_size() {
                    overlay.files.buffer_size
                } else {
                    base.files.buffer_size
                },
            },
            logging: LoggingConfig {
                level: if overlay.logging.level != default_log_level() {
                    overlay.logging.level
                } else {
                    base.logging.level
                },
                format: if overlay.logging.format != default_log_format() {
                    overlay.logging.format
                } else {
                    base.logging.format
                },
                colors: if overlay.logging.colors != default_log_colors() {
                    overlay.logging.colors
                } else {
                    base.logging.colors
                },
            },
            proxy: overlay.proxy.or(base.proxy),
        }
    }
}
