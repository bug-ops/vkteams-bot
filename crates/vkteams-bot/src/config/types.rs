use once_cell::sync::Lazy;
use serde::{self, Deserialize, Serialize};
use std::borrow::Cow;

pub static APP_FOLDER: &str = "VKTEAMS_BOT_CONFIG";
pub static CONFIG: Lazy<Config> = Lazy::new(Config::new);
/// Configuration file
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    #[cfg(feature = "otlp")]
    #[serde(default)]
    pub otlp: OtlpConfig,
    #[cfg(feature = "ratelimit")]
    #[serde(default)]
    pub rate_limit: RateLimit,
    #[serde(default)]
    pub network: NetworkConfig,
    #[cfg(feature = "longpoll")]
    #[serde(default)]
    pub listener: EventListenerConfig,
    #[cfg(feature = "storage")]
    #[serde(default)]
    pub storage: crate::storage::StorageConfig,
}

/// Otlp variables
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[repr(C)]
pub struct OtlpConfig {
    #[serde(default = "default_instance_id")]
    pub instance_id: Cow<'static, str>,
    #[serde(default = "default_deployment_environment_name")]
    pub deployment_environment_name: Cow<'static, str>,
    #[serde(default = "default_exporter_endpoint")]
    pub exporter_endpoint: Option<Cow<'static, str>>,
    #[serde(default = "default_exporter_timeout")]
    pub exporter_timeout: u64,
    #[serde(default = "default_exporter_metric_interval")]
    pub exporter_metric_interval: u64,
    #[serde(default = "default_ratio")]
    pub ratio: f64,
    #[serde(default = "default_otlp_filter_default")]
    pub otel_filter_default: Cow<'static, str>,
    #[serde(default = "default_fmt_filter_default")]
    pub fmt_filter_default: Cow<'static, str>,
    #[serde(default = "default_fmt_ansi")]
    pub fmt_ansi: bool,
    #[serde(default = "default_fmt_filter_self_directive")]
    pub fmt_filter_self_directive: Cow<'static, str>,
    #[serde(default = "default_otel")]
    pub otel: Vec<OtelDirective>,
    #[serde(default = "default_fmt")]
    pub fmt: Vec<FmtDirective>,
    #[serde(default)]
    pub log_format: LogFormat,
}

impl Default for OtlpConfig {
    fn default() -> Self {
        Self {
            instance_id: default_instance_id(),
            deployment_environment_name: default_deployment_environment_name(),
            exporter_endpoint: default_exporter_endpoint(),
            exporter_timeout: default_exporter_timeout(),
            exporter_metric_interval: default_exporter_metric_interval(),
            ratio: default_ratio(),
            otel_filter_default: default_otlp_filter_default(),
            fmt_filter_default: default_fmt_filter_default(),
            fmt_ansi: default_fmt_ansi(),
            fmt_filter_self_directive: default_fmt_filter_self_directive(),
            otel: default_otel(),
            fmt: default_fmt(),
            log_format: LogFormat::default(),
        }
    }
}

fn default_instance_id() -> Cow<'static, str> {
    Cow::Borrowed("bot")
}
fn default_deployment_environment_name() -> Cow<'static, str> {
    Cow::Borrowed("dev")
}
fn default_exporter_endpoint() -> Option<Cow<'static, str>> {
    None
}
fn default_exporter_timeout() -> u64 {
    0
}
fn default_exporter_metric_interval() -> u64 {
    0
}
fn default_ratio() -> f64 {
    1.0
}
fn default_otlp_filter_default() -> Cow<'static, str> {
    Cow::Borrowed("debug")
}
fn default_fmt_filter_default() -> Cow<'static, str> {
    Cow::Borrowed("debug")
}
fn default_fmt_ansi() -> bool {
    true
}
fn default_fmt_filter_self_directive() -> Cow<'static, str> {
    Cow::Borrowed("debug")
}
fn default_otel() -> Vec<OtelDirective> {
    vec![OtelDirective {
        otel_filter_directive: Cow::Borrowed("h2=off"),
    }]
}
fn default_fmt() -> Vec<FmtDirective> {
    vec![
        FmtDirective {
            fmt_filter_directive: Cow::Borrowed("axum=trace"),
        },
        FmtDirective {
            fmt_filter_directive: Cow::Borrowed("tower_http=debug"),
        },
        FmtDirective {
            fmt_filter_directive: Cow::Borrowed("opentelemetry=info"),
        },
    ]
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[repr(C)]
pub struct OtelDirective {
    pub otel_filter_directive: Cow<'static, str>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[repr(C)]
pub struct FmtDirective {
    pub fmt_filter_directive: Cow<'static, str>,
}
/// Rate limit configuration
#[cfg(feature = "ratelimit")]
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub struct RateLimit {
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default = "default_duration")]
    pub duration: u64,
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
    #[serde(default = "default_retry_attempts")]
    pub retry_attempts: u16,
    #[serde(default = "default_init_bucket")]
    pub init_bucket: usize,
    #[serde(default = "default_cleanup_interval")]
    pub cleanup_interval: u64,
    #[serde(default = "default_bucket_lifetime")]
    pub bucket_lifetime: u64,
}

#[cfg(feature = "ratelimit")]
impl Default for RateLimit {
    fn default() -> Self {
        Self {
            limit: default_limit(),
            duration: default_duration(),
            retry_delay: default_retry_delay(),
            retry_attempts: default_retry_attempts(),
            init_bucket: default_init_bucket(),
            cleanup_interval: default_cleanup_interval(),
            bucket_lifetime: default_bucket_lifetime(),
        }
    }
}
#[cfg(feature = "ratelimit")]
fn default_limit() -> usize {
    100
}
#[cfg(feature = "ratelimit")]
fn default_duration() -> u64 {
    60
}
#[cfg(feature = "ratelimit")]
fn default_retry_delay() -> u64 {
    1000
}
#[cfg(feature = "ratelimit")]
fn default_retry_attempts() -> u16 {
    3
}
#[cfg(feature = "ratelimit")]
fn default_init_bucket() -> usize {
    1
}
#[cfg(feature = "ratelimit")]
fn default_cleanup_interval() -> u64 {
    600
}
#[cfg(feature = "ratelimit")]
fn default_bucket_lifetime() -> u64 {
    3600
}

/// Configuration for event listener
#[cfg(feature = "longpoll")]
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub struct EventListenerConfig {
    /// Maximum number of events to process in a single batch
    #[serde(default = "default_max_events_per_batch")]
    pub max_events_per_batch: usize,
    /// Backoff time in milliseconds when no events received
    #[serde(default = "default_empty_backoff_ms")]
    pub empty_backoff_ms: u64,
    /// Maximum backoff time in milliseconds
    #[serde(default = "default_max_backoff_ms")]
    pub max_backoff_ms: u64,
    /// Whether to use exponential backoff when no events received
    #[serde(default = "default_use_exponential_backoff")]
    pub use_exponential_backoff: bool,
    /// Maximum memory usage for event processing in bytes (0 means no limit)
    #[serde(default = "default_max_memory_usage")]
    pub max_memory_usage: usize,
}

#[cfg(feature = "longpoll")]
impl Default for EventListenerConfig {
    fn default() -> Self {
        Self {
            max_events_per_batch: default_max_events_per_batch(),
            empty_backoff_ms: default_empty_backoff_ms(),
            max_backoff_ms: default_max_backoff_ms(),
            use_exponential_backoff: default_use_exponential_backoff(),
            max_memory_usage: default_max_memory_usage(),
        }
    }
}

#[cfg(feature = "longpoll")]
fn default_max_events_per_batch() -> usize {
    50
}
#[cfg(feature = "longpoll")]
fn default_empty_backoff_ms() -> u64 {
    500
}
fn default_max_backoff_ms() -> u64 {
    5000
}
#[cfg(feature = "longpoll")]
fn default_use_exponential_backoff() -> bool {
    true
}
#[cfg(feature = "longpoll")]
fn default_max_memory_usage() -> usize {
    0
}

/// Network configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct NetworkConfig {
    /// Number of retry attempts for failed requests
    #[serde(default = "default_retries")]
    pub retries: usize,
    /// Maximum backoff time in milliseconds
    #[serde(default = "default_max_backoff_ms")]
    pub max_backoff_ms: u64,
    /// Request timeout in seconds
    #[serde(default = "default_request_timeout_secs")]
    pub request_timeout_secs: u64,
    /// Connection timeout in seconds
    #[serde(default = "default_connect_timeout_secs")]
    pub connect_timeout_secs: u64,
    /// Pool idle timeout in seconds
    #[serde(default = "default_pool_idle_timeout_secs")]
    pub pool_idle_timeout_secs: u64,
    /// Maximum number of idle connections per host
    #[serde(default = "default_max_idle_connections")]
    pub max_idle_connections: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            retries: default_retries(),
            max_backoff_ms: default_max_backoff_ms(),
            request_timeout_secs: default_request_timeout_secs(),
            connect_timeout_secs: default_connect_timeout_secs(),
            pool_idle_timeout_secs: default_pool_idle_timeout_secs(),
            max_idle_connections: default_max_idle_connections(),
        }
    }
}

fn default_retries() -> usize {
    3
}
fn default_request_timeout_secs() -> u64 {
    30
}
fn default_connect_timeout_secs() -> u64 {
    10
}
fn default_pool_idle_timeout_secs() -> u64 {
    90
}
fn default_max_idle_connections() -> usize {
    10
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LogFormat {
    #[default]
    Pretty,
    Json,
    Full,
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml;

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        #[cfg(feature = "otlp")]
        {
            // Проверяем, что otlp присутствует и instance_id по умолчанию "bot"
            assert_eq!(config.otlp.instance_id, "bot");
        }
        #[cfg(feature = "ratelimit")]
        {
            // Проверяем значения по умолчанию для rate_limit
            assert_eq!(config.rate_limit.limit, 100);
        }
        #[cfg(feature = "storage")]
        {
            // Проверяем storage config
            assert_eq!(config.storage.database.max_connections, 20);
            assert!(config.storage.database.auto_migrate);
        }
        // Проверяем network config
        assert_eq!(config.network.retries, 3);
        assert_eq!(config.network.request_timeout_secs, 30);
    }

    #[test]
    fn test_serialize_deserialize_config() {
        let mut config = Config::default();
        #[cfg(feature = "otlp")]
        {
            config.otlp.instance_id = "test_id".into();
        }
        #[cfg(feature = "storage")]
        {
            config.storage.database.max_connections = 42;
        }
        config.network.retries = 7;
        let toml_str = toml::to_string(&config).unwrap();
        let deser: Config = toml::from_str(&toml_str).unwrap();
        #[cfg(feature = "otlp")]
        {
            assert_eq!(deser.otlp.instance_id, "test_id");
        }
        #[cfg(feature = "storage")]
        {
            assert_eq!(deser.storage.database.max_connections, 42);
        }
        assert_eq!(deser.network.retries, 7);
    }

    #[cfg(feature = "ratelimit")]
    #[test]
    fn test_default_limit() {
        assert_eq!(default_limit(), 100);
    }

    #[cfg(feature = "ratelimit")]
    #[test]
    fn test_default_duration() {
        assert_eq!(default_duration(), 60);
    }

    #[cfg(feature = "ratelimit")]
    #[test]
    fn test_default_retry_delay() {
        assert_eq!(default_retry_delay(), 1000);
    }
}
