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
    pub otlp: OtlpConfig,
    #[cfg(feature = "ratelimit")]
    pub rate_limit: RateLimit,
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
