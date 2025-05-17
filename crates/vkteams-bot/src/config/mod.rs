mod types;
use crate::error::Result;
use tracing::warn;
use types::APP_NAME;
pub use types::{CONFIG, Config, OtlpConfig};

impl Config {
    fn new() -> Self {
        // Get APP_NAME from .env file
        match get_config() {
            Ok(cfg) => cfg,
            Err(err) => {
                warn!(
                    "Failed to read config from file: {}. Use default instead",
                    err
                );
                Config::default()
            }
        }
    }
}

fn get_config() -> Result<Config> {
    std::env::var(APP_NAME)
        // Build config file path
        .map(|app| format!(".config/{app}.toml"))
        // Read config file to string
        .map(std::fs::read_to_string)?
        // Parse config file to Config struct
        .map(|str| toml::from_str::<Config>(&str))?
        .map_err(|e| e.into())
}
