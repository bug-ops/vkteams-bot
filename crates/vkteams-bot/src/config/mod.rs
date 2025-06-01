pub mod types;
pub use crate::config::types::{FmtDirective, OtelDirective};
use crate::error::Result;
use types::APP_FOLDER;
pub use types::{CONFIG, Config, LogFormat, OtlpConfig};

impl Config {
    pub fn new() -> Self {
        // Get APP_NAME from .env file
        match get_config() {
            Ok(cfg) => cfg,
            Err(_) => Config::default(),
        }
    }
}

pub fn get_config() -> Result<Config> {
    std::env::var(APP_FOLDER)
        // Read config file to string
        .map(std::fs::read_to_string)?
        // Parse config file to Config struct
        .map(|str| toml::from_str::<Config>(&str))?
        .map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new_default() {
        let config = Config::new();
        #[cfg(feature = "otlp")]
        {
            assert_eq!(config.otlp.instance_id, "bot");
        }
        assert_eq!(config.network.retries, 3);
    }

    #[test]
    fn test_config_default_values() {
        let config = Config::default();
        assert_eq!(config.network.retries, 3);
        assert_eq!(config.network.max_backoff_ms, 5000);
        assert_eq!(config.network.request_timeout_secs, 30);

        #[cfg(feature = "longpoll")]
        {
            assert_eq!(config.listener.empty_backoff_ms, 500);
            assert_eq!(config.listener.max_events_per_batch, 50);
        }

        #[cfg(feature = "otlp")]
        {
            assert_eq!(config.otlp.instance_id, "bot");
            assert_eq!(config.otlp.deployment_environment_name, "dev");
            assert_eq!(config.otlp.exporter_endpoint, None);
            assert_eq!(config.otlp.exporter_timeout, 0);
            assert_eq!(config.otlp.exporter_metric_interval, 0);
            assert_eq!(config.otlp.ratio, 1.0);
            assert_eq!(config.otlp.otel_filter_default, "debug");
            assert_eq!(config.otlp.fmt_filter_default, "debug");
            assert_eq!(config.otlp.fmt_filter_self_directive, "debug");
            assert_eq!(config.otlp.log_format, crate::config::LogFormat::Pretty);
            assert!(config.otlp.fmt_ansi);
        }
    }

    #[test]
    fn test_config_import_types() {
        // Test that all exported types are accessible
        let _config: Config = Config::default();
        let _log_format: LogFormat = LogFormat::Pretty;

        #[cfg(feature = "otlp")]
        {
            let _otlp_config: OtlpConfig = OtlpConfig::default();
        }
    }
}
