use crate::errors::prelude::{CliError, Result as CliResult};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tokio::time::Instant;
use toml;

// Use constants from the constants module
use crate::constants::config::{CONFIG_FILE_NAME, DEFAULT_CONFIG_DIR, ENV_PREFIX};
pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::load().expect("Failed to load config"));

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyConfig>,

    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
}

/// API Configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API token for VK Teams Bot
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// Base URL for API requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Timeout for API requests in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum number of retries for API requests
    #[serde(default = "default_retries")]
    pub max_retries: u32,
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

/// File handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    /// Default directory for downloads
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_dir: Option<String>,

    /// Default directory for uploads
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload_dir: Option<String>,

    /// Maximum file size in bytes for uploads and downloads
    #[serde(default = "default_max_file_size")]
    pub max_file_size: usize,

    /// Buffer size in bytes for file streaming
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
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

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            colors: default_log_colors(),
        }
    }
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

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_progress: default_show_progress(),
            progress_style: default_progress_style(),
            progress_refresh_rate: default_progress_refresh_rate(),
        }
    }
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Proxy URL
    #[serde(default)]
    pub url: String,

    /// Proxy user (if authentication is required)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Proxy password (if authentication is required)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            url: "".to_string(),
            user: None,
            password: None,
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

    /// Load configuration from all available sources asynchronously (preferred method)
    ///
    /// # Errors
    /// - Returns `CliError::FileError` if there is an error reading the config file
    /// - Returns `CliError::UnexpectedError` if there is an error parsing the config
    pub async fn load_async() -> CliResult<Self> {
        let manager = AsyncConfigManager::default();
        manager.load_config().await
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

    /// Save configuration to file asynchronously (preferred method)
    ///
    /// # Errors
    /// - Returns `CliError::FileError` if there is an error creating directories or writing the file
    /// - Returns `CliError::UnexpectedError` if there is an error serializing the config
    pub async fn save_async(&self, path: Option<&Path>) -> CliResult<()> {
        let path = if let Some(p) = path {
            p.to_owned()
        } else {
            let mut p = dirs::home_dir().ok_or_else(|| {
                CliError::FileError("Could not determine home directory".to_string())
            })?;
            p.push(DEFAULT_CONFIG_DIR);
            tokio::fs::create_dir_all(&p).await.map_err(|e| {
                CliError::FileError(format!("Could not create config directory: {e}"))
            })?;
            p.push(CONFIG_FILE_NAME);
            p
        };

        let config_clone = self.clone();
        let content = tokio::task::spawn_blocking(move || toml::to_string_pretty(&config_clone))
            .await
            .map_err(|e| CliError::UnexpectedError(format!("Task join error: {e}")))?
            .map_err(|e| CliError::UnexpectedError(format!("Could not serialize config: {e}")))?;

        tokio::fs::write(&path, content)
            .await
            .map_err(|e| CliError::FileError(format!("Could not write config file: {e}")))?;

        Ok(())
    }

    /// Load configuration from all available sources
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
        Ok(toml::from_str::<Config>("").unwrap())
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

    /// Load configuration from a specific path asynchronously
    ///
    /// # Errors
    /// - Returns `CliError::FileError` if there is an error reading the config file
    /// - Returns `CliError::UnexpectedError` if there is an error parsing the config
    pub async fn from_path_async(path: &Path) -> CliResult<Self> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| CliError::FileError(format!("Could not read config file: {e}")))?;

        let config = tokio::task::spawn_blocking(move || toml::from_str::<Config>(&content))
            .await
            .map_err(|e| CliError::UnexpectedError(format!("Task join error: {e}")))?
            .map_err(|e| CliError::UnexpectedError(format!("Could not parse config file: {e}")))?;

        Ok(config)
    }

    /// Load configuration from environment variables
    ///
    /// # Errors
    /// - Returns `CliError::FileError` if there is an error with file operations
    /// - Returns `CliError::UnexpectedError` for unexpected errors
    pub fn from_env() -> CliResult<Self> {
        let mut config = toml::from_str::<Config>("").unwrap();

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
            rate_limit: crate::utils::config_helpers::merge_rate_limit_configs(
                base.rate_limit,
                overlay.rate_limit,
            ),
        }
    }
}

/// Async configuration manager with caching and file watching
#[derive(Debug)]
pub struct AsyncConfigManager {
    cache: Arc<RwLock<Option<(Config, SystemTime)>>>,
    cache_ttl: Duration,
    pub config_paths: Vec<PathBuf>,
}

/// Configuration change event
#[derive(Clone, Debug)]
pub enum ConfigChange {
    FileModified(PathBuf),
    EnvironmentChanged(String),
    ManualUpdate(Config),
}

/// Lock-free configuration cache for high-performance access
#[derive(Debug, Clone)]
pub struct LockFreeConfigCache {
    cache: Arc<DashMap<String, Arc<Config>>>,
    timestamps: Arc<DashMap<String, Instant>>,
    ttl: Duration,
}

impl Default for AsyncConfigManager {
    fn default() -> Self {
        Self::new(Duration::from_secs(300)) // 5 minutes TTL
    }
}

impl AsyncConfigManager {
    /// Create a new async config manager with specified TTL
    pub fn new(cache_ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(None)),
            cache_ttl,
            config_paths: crate::utils::config_helpers::get_config_paths(),
        }
    }

    /// Load configuration asynchronously with caching
    pub async fn load_config(&self) -> CliResult<Config> {
        // Fast path: check cache first
        {
            let cache = self.cache.read().await;
            if let Some((config, timestamp)) = cache.as_ref() {
                if timestamp.elapsed().unwrap_or(Duration::MAX) < self.cache_ttl {
                    return Ok(config.clone());
                }
            }
        }

        // Slow path: reload from disk asynchronously
        let config = self.load_from_sources().await?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            *cache = Some((config.clone(), SystemTime::now()));
        }

        Ok(config)
    }

    /// Load configuration from all sources in parallel
    pub async fn load_from_sources(&self) -> CliResult<Config> {
        // Load file configs in parallel
        let file_futures: Vec<_> = self
            .config_paths
            .iter()
            .filter(|path| path.exists())
            .map(|path| self.load_config_file_async(path.clone()))
            .collect();

        let file_configs = futures::future::join_all(file_futures)
            .await
            .into_iter()
            .filter_map(|result| result.ok())
            .collect::<Vec<_>>();

        // Load environment config
        let env_config = Self::from_env_async().await?;

        // Merge all configs efficiently
        let mut final_config = Config::default();
        for config in file_configs {
            final_config = Self::merge_configs_efficient(final_config, config);
        }
        final_config = Self::merge_configs_efficient(final_config, env_config);

        Ok(final_config)
    }

    /// Load a single config file asynchronously
    async fn load_config_file_async(&self, path: PathBuf) -> CliResult<Config> {
        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| CliError::FileError(format!("Could not read config file: {e}")))?;

        // Parse TOML in a blocking task to avoid blocking the async runtime
        let config = tokio::task::spawn_blocking(move || toml::from_str::<Config>(&content))
            .await
            .map_err(|e| CliError::UnexpectedError(format!("Task join error: {e}")))?
            .map_err(|e| CliError::UnexpectedError(format!("Could not parse config file: {e}")))?;

        Ok(config)
    }

    /// Load environment variables asynchronously
    async fn from_env_async() -> CliResult<Config> {
        // Spawn environment parsing in blocking task
        tokio::task::spawn_blocking(|| Config::from_env())
            .await
            .map_err(|e| CliError::UnexpectedError(format!("Task join error: {e}")))?
    }

    /// Efficient config merging without intermediate allocations
    pub fn merge_configs_efficient(base: Config, overlay: Config) -> Config {
        Config {
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
            ui: UiConfig {
                show_progress: if overlay.ui.show_progress != default_show_progress() {
                    overlay.ui.show_progress
                } else {
                    base.ui.show_progress
                },
                progress_style: if overlay.ui.progress_style != default_progress_style() {
                    overlay.ui.progress_style
                } else {
                    base.ui.progress_style
                },
                progress_refresh_rate: if overlay.ui.progress_refresh_rate
                    != default_progress_refresh_rate()
                {
                    overlay.ui.progress_refresh_rate
                } else {
                    base.ui.progress_refresh_rate
                },
            },
            proxy: overlay.proxy.or(base.proxy),
            rate_limit: overlay.rate_limit, // Use overlay rate_limit directly for simplicity
        }
    }
}

impl LockFreeConfigCache {
    /// Create a new lock-free config cache
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            timestamps: Arc::new(DashMap::new()),
            ttl,
        }
    }

    /// Get configuration from cache if valid, otherwise load and cache it
    pub async fn get_or_load<F, Fut>(&self, key: &str, loader: F) -> CliResult<Arc<Config>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = CliResult<Config>>,
    {
        // Check if we have a valid cached entry
        if let Some(config) = self.cache.get(key) {
            if let Some(timestamp) = self.timestamps.get(key) {
                if timestamp.elapsed() < self.ttl {
                    return Ok(config.clone());
                }
            }
        }

        // Load fresh config
        let new_config = Arc::new(loader().await?);

        // Cache the result
        self.cache.insert(key.to_string(), new_config.clone());
        self.timestamps.insert(key.to_string(), Instant::now());

        Ok(new_config)
    }

    /// Invalidate cache entry
    pub fn invalidate(&self, key: &str) {
        self.cache.remove(key);
        self.timestamps.remove(key);
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        self.cache.clear();
        self.timestamps.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.cache.len(), self.timestamps.len())
    }
}
