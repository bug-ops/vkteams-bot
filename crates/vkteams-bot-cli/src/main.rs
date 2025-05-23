pub mod cli;
pub mod config;
pub mod errors;
pub mod file_utils;
pub mod progress;
pub mod scheduler;

use cli::Cli;
use config::Config;
use std::process::exit;
use tracing::debug;
use vkteams_bot::otlp;
// use vkteams_bot::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = otlp::init()?;
    // Load configuration
    let config = match Config::load() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to load configuration: {err}");
            exit(exitcode::CONFIG);
        }
    };

    debug!("Configuration loaded successfully");

    let mut cli = Cli::with_config(config);
    match cli.match_input().await {
        Ok(()) => (),
        Err(err) => {
            eprintln!("Error: {}", err);
            err.exit_with_error();
        }
    }
    Ok(())
}
