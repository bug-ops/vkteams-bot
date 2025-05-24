use crate::errors::prelude::{CliError, Result as CliResult};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;
use toml;

// Use constants from the constants module
use crate::constants::config::{CONFIG_FILE_NAME, DEFAULT_CONFIG_DIR, ENV_PREFIX};

/// Configuration structure for VK Teams Bot CLI
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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

    /// UI Configuration including progress bars
    #[serde(default)]
    pub ui: UiConfig,

    /// Proxy configuration
    #[serde(default)]
    pub proxy: Option<ProxyConfig>,

    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
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

/// UI and progress indicator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Enable or disable progress bars
    #[serde(default = "default_show_progress")]
    pub show_progress: bool,

    /// Progress bar style (default, unicode, ascii)
    #[serde(default = "default_progress_style")]
    pub progress_style: String,

    /// Progress bar refresh rate in milliseconds
    #[serde(default = "default_progress_refresh_rate")]
    pub progress_refresh_rate: u64,
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

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_progress: default_show_progress(),
            progress_style: default_progress_style(),
            progress_refresh_rate: default_progress_refresh_rate(),
        }
    }
}

// Default values functions
pub fn default_timeout() -> u64 {
    30
}

pub fn default_retries() -> u32 {
    3
}

pub fn default_max_file_size() -> usize {
    100 * 1024 * 1024 // 100MB
}

pub fn default_buffer_size() -> usize {
    64 * 1024 // 64KB
}

pub fn default_log_level() -> String {
    "info".to_string()
}

pub fn default_log_format() -> String {
    "text".to_string()
}

pub fn default_log_colors() -> bool {
    true
}

// Default values for UI configuration
pub fn default_show_progress() -> bool {
    true
}

pub fn default_progress_style() -> String {
    "unicode".to_string()
}

pub fn default_progress_refresh_rate() -> u64 {
    100
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable or disable rate limiting
    #[serde(default = "default_rate_limit_enabled")]
    pub enabled: bool,

    /// Maximum requests per time window
    #[serde(default = "default_rate_limit_limit")]
    pub limit: usize,

    /// Time window duration in seconds
    #[serde(default = "default_rate_limit_duration")]
    pub duration: u64,

    /// Delay between retry attempts in milliseconds
    #[serde(default = "default_rate_limit_retry_delay")]
    pub retry_delay: u64,

    /// Maximum number of retry attempts
    #[serde(default = "default_rate_limit_retry_attempts")]
    pub retry_attempts: u16,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: default_rate_limit_enabled(),
            limit: default_rate_limit_limit(),
            duration: default_rate_limit_duration(),
            retry_delay: default_rate_limit_retry_delay(),
            retry_attempts: default_rate_limit_retry_attempts(),
        }
    }
}

/// Default values for rate limiting configuration
pub fn default_rate_limit_enabled() -> bool {
    false // Disabled by default for CLI usage
}

pub fn default_rate_limit_limit() -> usize {
    1000 // More generous limit for CLI
}

pub fn default_rate_limit_duration() -> u64 {
    60
}

pub fn default_rate_limit_retry_delay() -> u64 {
    500 // Shorter delay for CLI
}

pub fn default_rate_limit_retry_attempts() -> u16 {
    3
}

impl Config {
    /// Load configuration from all available sources
    ///
    /// # Errors
    /// - Returns `CliError::FileError` if there is an error reading the config file
    /// - Returns `CliError::UnexpectedError` if there is an error parsing the config
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
    ///
    /// # Errors
    /// - Returns `CliError::FileError` if there is an error reading the config file
    /// - Returns `CliError::UnexpectedError` if there is an error parsing the config
    pub fn from_file() -> CliResult<Self> {
        let config_paths = crate::utils::config_helpers::get_config_paths();

        for path in config_paths {
            if path.exists() {
                return Self::from_path(&path);
            }
        }

        // Return default config if no file found
        Ok(Config::default())
    }

    /// Load configuration from a specific path
    ///
    /// # Errors
    /// - Returns `CliError::FileError` if there is an error reading the config file
    /// - Returns `CliError::UnexpectedError` if there is an error parsing the config
    pub fn from_path(path: &Path) -> CliResult<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| CliError::FileError(format!("Could not read config file: {e}")))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| CliError::UnexpectedError(format!("Could not parse config file: {e}")))?;

        Ok(config)
    }



    /// Save configuration to file
    ///
    /// # Errors
    /// - Returns `CliError::FileError` if there is an error creating directories or writing the file
    /// - Returns `CliError::UnexpectedError` if there is an error serializing the config
    pub fn save(&self, path: Option<&Path>) -> CliResult<()> {
        let path = if let Some(p) = path {
            p.to_owned()
        } else {
            let mut p = dirs::home_dir().ok_or_else(|| {
                CliError::FileError("Could not determine home directory".to_string())
            })?;
            p.push(DEFAULT_CONFIG_DIR);
            fs::create_dir_all(&p).map_err(|e| {
                CliError::FileError(format!("Could not create config directory: {e}"))
            })?;
            p.push(CONFIG_FILE_NAME);
            p
        };

        let content = toml::to_string_pretty(self)
            .map_err(|e| CliError::UnexpectedError(format!("Could not serialize config: {e}")))?;

        fs::write(&path, content)
            .map_err(|e| CliError::FileError(format!("Could not write config file: {e}")))?;

        Ok(())
    }

    /// Load configuration from environment variables
    ///
    /// # Errors
    /// - Returns `CliError::FileError` if there is an error with file operations
    /// - Returns `CliError::UnexpectedError` for unexpected errors
    pub fn from_env() -> CliResult<Self> {
        let mut config = Config::default();

        // API config
        if let Ok(token) = env::var(format!("{ENV_PREFIX}BOT_API_TOKEN")) {
            config.api.token = Some(token);
        }

        if let Ok(url) = env::var(format!("{ENV_PREFIX}BOT_API_URL")) {
            config.api.url = Some(url);
        }

        if let Ok(timeout_str) = env::var(format!("{ENV_PREFIX}TIMEOUT")) {
            if let Ok(timeout_val) = timeout_str.parse::<u64>() {
                config.api.timeout = timeout_val;
            }
        }

        // File config
        if let Ok(download_dir) = env::var(format!("{ENV_PREFIX}DOWNLOAD_DIR")) {
            config.files.download_dir = Some(download_dir);
        }

        if let Ok(upload_dir) = env::var(format!("{ENV_PREFIX}UPLOAD_DIR")) {
            config.files.upload_dir = Some(upload_dir);
        }

        if let Ok(max_file_size_str) = env::var(format!("{ENV_PREFIX}MAX_FILE_SIZE")) {
            if let Ok(max_file_size_val) = max_file_size_str.parse::<usize>() {
                config.files.max_file_size = max_file_size_val;
            }
        }

        // Logging config
        if let Ok(level) = env::var(format!("{ENV_PREFIX}LOG_LEVEL")) {
            config.logging.level = level;
        }

        if let Ok(format) = env::var(format!("{ENV_PREFIX}LOG_FORMAT")) {
            config.logging.format = format;
        }

        if let Ok(colors_str) = env::var(format!("{ENV_PREFIX}LOG_COLORS")) {
            if let Ok(colors_val) = colors_str.parse::<bool>() {
                config.logging.colors = colors_val;
            }
        }

        // UI config
        if let Ok(show_progress_str) = env::var(format!("{ENV_PREFIX}SHOW_PROGRESS")) {
            if let Ok(show_progress_val) = show_progress_str.parse::<bool>() {
                config.ui.show_progress = show_progress_val;
            }
        }

        if let Ok(progress_style) = env::var(format!("{ENV_PREFIX}PROGRESS_STYLE")) {
            config.ui.progress_style = progress_style;
        }

        if let Ok(refresh_rate_str) = env::var(format!("{ENV_PREFIX}PROGRESS_REFRESH_RATE")) {
            if let Ok(refresh_rate_val) = refresh_rate_str.parse::<u64>() {
                config.ui.progress_refresh_rate = refresh_rate_val;
            }
        }

        // Proxy config
        if let Ok(proxy_url) = env::var(format!("{ENV_PREFIX}PROXY")) {
            config.proxy = Some(ProxyConfig {
                url: proxy_url,
                user: env::var(format!("{ENV_PREFIX}PROXY_USER")).ok(),
                password: env::var(format!("{ENV_PREFIX}PROXY_PASSWORD")).ok(),
            });
        }

        // Rate limiting config
        if let Ok(enabled_str) = env::var(format!("{ENV_PREFIX}RATE_LIMIT_ENABLED")) {
            if let Ok(enabled_val) = enabled_str.parse::<bool>() {
                config.rate_limit.enabled = enabled_val;
            }
        }

        if let Ok(limit_str) = env::var(format!("{ENV_PREFIX}RATE_LIMIT_LIMIT")) {
            if let Ok(limit_val) = limit_str.parse::<usize>() {
                config.rate_limit.limit = limit_val;
            }
        }

        if let Ok(duration_str) = env::var(format!("{ENV_PREFIX}RATE_LIMIT_DURATION")) {
            if let Ok(duration_val) = duration_str.parse::<u64>() {
                config.rate_limit.duration = duration_val;
            }
        }

        Ok(config)
    }

    /// Merge two configurations, with the second taking precedence
    fn merge_configs(base: Self, overlay: Self) -> Self {
        Self {
            api: ApiConfig {
                token: overlay.api.token.or(base.api.token),
                url: overlay.api.url.or(base.api.url),
                timeout: if overlay.api.timeout == default_timeout() {
                    base.api.timeout
                } else {
                    overlay.api.timeout
                },
                max_retries: if overlay.api.max_retries == default_retries() {
                    base.api.max_retries
                } else {
                    overlay.api.max_retries
                },
            },
            files: FileConfig {
                download_dir: overlay.files.download_dir.or(base.files.download_dir),
                upload_dir: overlay.files.upload_dir.or(base.files.upload_dir),
                max_file_size: if overlay.files.max_file_size == default_max_file_size() {
                    base.files.max_file_size
                } else {
                    overlay.files.max_file_size
                },
                buffer_size: if overlay.files.buffer_size == default_buffer_size() {
                    base.files.buffer_size
                } else {
                    overlay.files.buffer_size
                },
            },
            logging: LoggingConfig {
                level: if overlay.logging.level == default_log_level() {
                    base.logging.level
                } else {
                    overlay.logging.level
                },
                format: if overlay.logging.format == default_log_format() {
                    base.logging.format
                } else {
                    overlay.logging.format
                },
                colors: if overlay.logging.colors == default_log_colors() {
                    base.logging.colors
                } else {
                    overlay.logging.colors
                },
            },
            ui: UiConfig {
                show_progress: if overlay.ui.show_progress == default_show_progress() {
                    base.ui.show_progress
                } else {
                    overlay.ui.show_progress
                },
                progress_style: if overlay.ui.progress_style == default_progress_style() {
                    base.ui.progress_style
                } else {
                    overlay.ui.progress_style
                },
                progress_refresh_rate: if overlay.ui.progress_refresh_rate
                    == default_progress_refresh_rate()
                {
                    base.ui.progress_refresh_rate
                } else {
                    overlay.ui.progress_refresh_rate
                },
            },
            proxy: overlay.proxy.or(base.proxy),
            rate_limit: crate::utils::config_helpers::merge_rate_limit_configs(base.rate_limit, overlay.rate_limit),
        }
    }


}
