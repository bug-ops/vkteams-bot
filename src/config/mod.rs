mod types;
use std::sync::Arc;
use types::APP_NAME;
pub use types::{CONFIG, Config, OtlpConfig};

impl Config {
    fn new() -> Arc<Self> {
        // Get APP_NAME from .env file
        Arc::new(
            std::env::var(APP_NAME)
                // Build config file path
                .map(|app| format!(".config/{app}.toml"))
                // Read config file to string
                .map(std::fs::read_to_string)
                .expect("Config file not found")
                // Parse config file to Config struct
                .map(|str| toml::from_str::<Config>(&str))
                .expect("Error reading config file")
                .expect("Error parsing file"),
        )
    }
}
