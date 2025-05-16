use once_cell::sync::Lazy;
use serde::{self, Deserialize, Serialize};
use std::borrow::Cow;
use std::sync::Arc;

pub static APP_NAME: &str = "APP_NAME";
pub static CONFIG: Lazy<Arc<Config>> = Lazy::new(Config::new);
/// Configuration file
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    #[cfg(feature = "otlp")]
    pub otlp: OtlpConfig,
    #[cfg(feature = "ratelimit")]
    pub rate_limit: RateLimit,
    /// Network configuration
    #[serde(default)]
    pub network: NetworkConfig,
}
/// Otlp variables
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[repr(C)]
pub struct OtlpConfig {
    pub instance_id: Cow<'static, str>,
    pub deployment_environment_name: Cow<'static, str>,
    pub exporter_endpoint: Cow<'static, str>,
    pub exporter_timeout: u64,
    pub exporter_metric_interval: u64,
    pub ratio: f64,
    pub otel_filter_default: Cow<'static, str>,
    pub fmt_filter_default: Cow<'static, str>,
    pub fmt_ansi: bool,
    pub fmt_filter_self_directive: Cow<'static, str>,
    pub otel: Vec<OtelDirective>,
    pub fmt: Vec<FmtDirective>,
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
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub struct RateLimit {
    pub limit: usize,
    pub duration: u64,
    pub retry_delay: u64,
    pub retry_attempts: u16,
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

fn default_retries() -> usize { 3 }
fn default_max_backoff_ms() -> u64 { 5000 }
fn default_request_timeout_secs() -> u64 { 30 }
fn default_connect_timeout_secs() -> u64 { 10 }
fn default_pool_idle_timeout_secs() -> u64 { 90 }
fn default_max_idle_connections() -> usize { 10 }
