pub mod cli;
pub mod config;
pub mod errors;
pub mod file_utils;
pub mod progress;

use cli::Cli;
use config::Config;
use std::process::exit;
use tracing::debug;
use vkteams_bot::prelude::*;

#[tokio::main]
async fn main() {
    let _guard = otlp::init().map_err(|e| BotError::Otlp(e.into()));

    // Load configuration
    let config = match Config::load() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to load configuration: {err}");
            exit(exitcode::CONFIG);
        }
    };

    debug!("Configuration loaded successfully");

    match Cli::with_config(config).match_input().await {
        Ok(()) => (),
        Err(err) => {
            err.exit_with_error();
        }
    }
}
