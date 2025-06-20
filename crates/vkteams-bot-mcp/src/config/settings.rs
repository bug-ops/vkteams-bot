use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub bot: BotConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub api_token: String,
    pub api_url: String,
    pub chat_id: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Config {
            database: DatabaseConfig {
                url: env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./events.db".to_string()),
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
            },
            bot: BotConfig {
                api_token: env::var("VKTEAMS_BOT_API_TOKEN")
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?,
                api_url: env::var("VKTEAMS_BOT_API_URL")
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?,
                chat_id: env::var("VKTEAMS_BOT_CHAT_ID")
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?,
            },
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_env().expect("Failed to load configuration from environment")
    }
}