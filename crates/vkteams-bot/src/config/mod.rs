pub mod types;
pub use crate::config::types::{FmtDirective, OtelDirective};
use crate::error::Result;
use types::APP_FOLDER;
pub use types::{CONFIG, Config, LogFormat, OtlpConfig};

impl Config {
    fn new() -> Self {
        // Get APP_NAME from .env file
        get_config().unwrap_or_default()
    }
}

fn get_config() -> Result<Config> {
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
}
