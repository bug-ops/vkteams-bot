use once_cell::sync::Lazy;
use serde::{self, Deserialize, Serialize};
use std::borrow::Cow;

pub static APP_NAME: &str = "APP_NAME";
pub static CONFIG: Lazy<Config> = Lazy::new(Config::new);
/// Configuration file
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    #[cfg(feature = "otlp")]
    pub otlp: OtlpConfig,
    #[cfg(feature = "ratelimit")]
    pub rate_limit: RateLimit,
    pub network: NetworkConfig,
    #[cfg(feature = "longpoll")]
    pub listener: EventListenerConfig,
}
/// Otlp variables
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
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
#[derive(Debug, Serialize, Deserialize, Default, Clone, Eq, PartialEq, Hash)]
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
}

fn default_limit() -> usize {
    100
}
fn default_duration() -> u64 {
    60
}
fn default_retry_delay() -> u64 {
    1000
}
fn default_retry_attempts() -> u16 {
    3
}

/// Configuration for event listener
#[cfg(feature = "longpoll")]
#[derive(Debug, Serialize, Deserialize, Default, Clone, Eq, PartialEq, Hash)]
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
fn default_max_events_per_batch() -> usize {
    50
}
#[cfg(feature = "longpoll")]
fn default_empty_backoff_ms() -> u64 {
    500
}
#[cfg(feature = "longpoll")]
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
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
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
