use pretty_env_logger::env_logger;
use std::process::exit;
use tracing::debug;

pub mod cli;
pub mod config;
pub mod errors;
pub mod file_utils;

use cli::Cli;
use config::Config;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    // Load configuration
    let config = match Config::load() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to load configuration: {}", err);
            exit(exitcode::CONFIG);
        }
    };
    
    debug!("Configuration loaded successfully");
    
    match Cli::with_config(config).match_input().await {
        Ok(_) => (),
        Err(err) => {
            err.exit_with_error();
        }
    }
}
