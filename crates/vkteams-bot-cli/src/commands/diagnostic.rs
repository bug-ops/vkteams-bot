//! Diagnostic commands module
//!
//! This module contains all commands related to diagnostics, testing, and system information.

use crate::commands::Command;
use crate::constants::ui::emoji;
use crate::errors::prelude::{CliError, Result as CliResult};
use crate::file_utils;
use crate::config::Config;
use async_trait::async_trait;
use clap::Subcommand;
use colored::Colorize;
use tracing::{debug, info};
use vkteams_bot::prelude::*;

/// All diagnostic-related commands
#[derive(Subcommand, Debug, Clone)]
pub enum DiagnosticCommands {
    /// Get bot information and status
    GetSelf {
        /// Show detailed bot information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Get events once or listen with optional flag
    GetEvents {
        #[arg(short, long, required = false, value_name = "LISTEN")]
        listen: Option<bool>,
    },
    /// Download file with given ID into specified path
    GetFile {
        #[arg(short = 'f', long, required = true, value_name = "FILE_ID")]
        file_id: String,
        #[arg(short = 'p', long, required = false, value_name = "FILE_PATH")]
        file_path: String,
    },
    /// Perform comprehensive health check
    HealthCheck,
    /// Test network connectivity to API endpoints
    NetworkTest,
    /// Show system and environment information
    SystemInfo,
    /// Test API rate limits
    RateLimitTest {
        /// Number of requests to send
        #[arg(short = 'n', long, default_value = "10")]
        requests: u32,
        /// Delay between requests in milliseconds
        #[arg(short = 'd', long, default_value = "100")]
        delay_ms: u64,
    },
}

#[async_trait]
impl Command for DiagnosticCommands {
    async fn execute(&self, bot: &Bot) -> CliResult<()> {
        match self {
            DiagnosticCommands::GetSelf { detailed } => {
                execute_get_self(bot, *detailed).await
            }
            DiagnosticCommands::GetEvents { listen } => {
                execute_get_events(bot, listen.unwrap_or(false)).await
            }
            DiagnosticCommands::GetFile { file_id, file_path } => {
                execute_get_file(bot, file_id, file_path).await
            }
            DiagnosticCommands::HealthCheck => {
                execute_health_check(bot).await
            }
            DiagnosticCommands::NetworkTest => {
                execute_network_test(bot).await
            }
            DiagnosticCommands::SystemInfo => {
                execute_system_info().await
            }
            DiagnosticCommands::RateLimitTest { requests, delay_ms } => {
                execute_rate_limit_test(bot, *requests, *delay_ms).await
            }
        }
    }

    fn name(&self) -> &'static str {
        match self {
            DiagnosticCommands::GetSelf { .. } => "get-self",
            DiagnosticCommands::GetEvents { .. } => "get-events",
            DiagnosticCommands::GetFile { .. } => "get-file",
            DiagnosticCommands::HealthCheck => "health-check",
            DiagnosticCommands::NetworkTest => "network-test",
            DiagnosticCommands::SystemInfo => "system-info",
            DiagnosticCommands::RateLimitTest { .. } => "rate-limit-test",
        }
    }

    fn validate(&self) -> CliResult<()> {
        match self {
            DiagnosticCommands::GetFile { file_id, file_path } => {
                validate_file_id(file_id)?;
                if !file_path.is_empty() {
                    validate_directory_path(file_path)?;
                }
            }
            DiagnosticCommands::RateLimitTest { requests, delay_ms: _ } => {
                if *requests == 0 || *requests > 1000 {
                    return Err(CliError::InputError(
                        "Number of requests must be between 1 and 1000".to_string()
                    ));
                }
            }
            _ => {} // Other commands don't need validation
        }
        Ok(())
    }
}

// Command execution functions

async fn execute_get_self(bot: &Bot, detailed: bool) -> CliResult<()> {
    debug!("Getting bot information");
    
    let request = RequestSelfGet::new(());
    let result = bot.send_api_request(request).await
        .map_err(CliError::ApiError)?;

    if detailed {
        info!("Bot information retrieved successfully");
        print_success_result(&result)?;
    } else {
        // Show simplified bot info
        println!("{} Bot is configured and accessible", emoji::CHECK);
        if let Ok(json_str) = serde_json::to_string_pretty(&result) {
            println!("{}", json_str.green());
        }
    }
    
    Ok(())
}

async fn execute_get_events(bot: &Bot, listen: bool) -> CliResult<()> {
    debug!("Getting events, listen mode: {}", listen);
    
    if listen {
        info!("Starting event listener (long polling)...");
        println!("{} Starting event listener. Press Ctrl+C to stop.", emoji::ROCKET);
        
        match bot.event_listener(handle_event).await {
            Ok(()) => (),
            Err(e) => return Err(CliError::ApiError(e)),
        }
    } else {
        let result = bot
            .send_api_request(RequestEventsGet::new(bot.get_last_event_id().await))
            .await
            .map_err(CliError::ApiError)?;

        info!("Successfully retrieved events");
        print_success_result(&result)?;
    }
    
    Ok(())
}

async fn execute_get_file(bot: &Bot, file_id: &str, file_path: &str) -> CliResult<()> {
    debug!("Downloading file {} to {}", file_id, file_path);
    
    let config = Config::default(); // TODO: Pass actual config
    let downloaded_path = file_utils::download_and_save_file(bot, file_id, file_path, &config).await?;
    
    info!("Successfully downloaded file with ID: {}", file_id);
    println!("{} File downloaded to: {}", emoji::CHECK, downloaded_path.display().to_string().green());
    
    Ok(())
}

async fn execute_health_check(bot: &Bot) -> CliResult<()> {
    println!("{} Performing comprehensive health check...", emoji::TEST_TUBE.bold().blue());
    println!();
    
    let mut all_passed = true;
    
    // Test 1: Basic connectivity
    print!("{} Testing basic API connectivity... ", emoji::GEAR);
    match bot.send_api_request(RequestSelfGet::new(())).await {
        Ok(_) => println!("{}", "PASS".green()),
        Err(e) => {
            println!("{} - {}", "FAIL".red(), e);
            all_passed = false;
        }
    }
    
    // Test 2: Configuration check
    print!("{} Checking configuration... ", emoji::GEAR);
    match Config::from_file() {
        Ok(config) => {
            if config.api.token.is_some() && config.api.url.is_some() {
                println!("{}", "PASS".green());
            } else {
                println!("{} - Missing required configuration", "FAIL".red());
                all_passed = false;
            }
        }
        Err(_) => {
            println!("{} - Configuration file not found", "FAIL".red());
            all_passed = false;
        }
    }
    
    // Test 3: Network latency
    print!("{} Testing network latency... ", emoji::GEAR);
    let start = std::time::Instant::now();
    match bot.send_api_request(RequestSelfGet::new(())).await {
        Ok(_) => {
            let latency = start.elapsed();
            if latency.as_millis() < 1000 {
                println!("{} - {}ms", "PASS".green(), latency.as_millis());
            } else {
                println!("{} - High latency: {}ms", "WARN".yellow(), latency.as_millis());
            }
        }
        Err(e) => {
            println!("{} - {}", "FAIL".red(), e);
            all_passed = false;
        }
    }
    
    println!();
    if all_passed {
        println!("{} All health checks passed!", emoji::CHECK.bold().green());
    } else {
        println!("{} Some health checks failed. Check configuration and network connectivity.", emoji::WARNING.bold().yellow());
    }
    
    Ok(())
}

async fn execute_network_test(bot: &Bot) -> CliResult<()> {
    println!("{} Testing network connectivity...", emoji::GEAR.bold().blue());
    println!();
    
    // Test multiple endpoints with timing
    let endpoints = vec![
        ("Bot Info", RequestSelfGet::new(())),
    ];
    
    for (name, request) in endpoints {
        print!("Testing {}: ", name);
        let start = std::time::Instant::now();
        
        match bot.send_api_request(request).await {
            Ok(_) => {
                let duration = start.elapsed();
                println!("{} ({}ms)", "OK".green(), duration.as_millis());
            }
            Err(e) => {
                println!("{} - {}", "FAILED".red(), e);
            }
        }
    }
    
    println!();
    println!("{} Network test completed", emoji::CHECK);
    
    Ok(())
}

async fn execute_system_info() -> CliResult<()> {
    println!("{} System Information", emoji::INFO.bold().blue());
    println!();
    
    // Runtime information
    println!("{}", "Runtime:".bold().green());
    println!("  OS: {}", std::env::consts::OS);
    println!("  Architecture: {}", std::env::consts::ARCH);
    println!("  Family: {}", std::env::consts::FAMILY);
    
    // Current directory
    if let Ok(current_dir) = std::env::current_dir() {
        println!("  Current directory: {}", current_dir.display());
    }
    
    // Environment variables
    println!("\n{}", "Environment:".bold().green());
    let env_vars = [
        "VKTEAMS_BOT_API_TOKEN",
        "VKTEAMS_BOT_API_URL", 
        "VKTEAMS_PROXY",
        "VKTEAMS_LOG_LEVEL",
    ];
    
    for var in &env_vars {
        match std::env::var(var) {
            Ok(value) => {
                if var.contains("TOKEN") {
                    println!("  {}: {}***", var, &value[..8.min(value.len())]);
                } else {
                    println!("  {}: {}", var, value);
                }
            }
            Err(_) => println!("  {}: {}", var, "Not set".dimmed()),
        }
    }
    
    // Configuration file status
    println!("\n{}", "Configuration:".bold().green());
    match Config::from_file() {
        Ok(_) => println!("  Configuration file: {}", "Found".green()),
        Err(_) => println!("  Configuration file: {}", "Not found".red()),
    }
    
    Ok(())
}

async fn execute_rate_limit_test(bot: &Bot, requests: u32, delay_ms: u64) -> CliResult<()> {
    println!("{} Testing rate limits with {} requests ({}ms delay)...", 
             emoji::ROCKET.bold().blue(), requests, delay_ms);
    println!();
    
    let mut successful = 0;
    let mut failed = 0;
    let start_time = std::time::Instant::now();
    
    for i in 1..=requests {
        let request_start = std::time::Instant::now();
        
        match bot.send_api_request(RequestSelfGet::new(())).await {
            Ok(_) => {
                successful += 1;
                let duration = request_start.elapsed();
                println!("Request {}/{}: {} ({}ms)", 
                        i, requests, "OK".green(), duration.as_millis());
            }
            Err(e) => {
                failed += 1;
                println!("Request {}/{}: {} - {}", 
                        i, requests, "FAILED".red(), e);
            }
        }
        
        if i < requests {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
        }
    }
    
    let total_time = start_time.elapsed();
    
    println!();
    println!("{}", "Rate Limit Test Results:".bold().green());
    println!("  Total requests: {}", requests);
    println!("  Successful: {}", successful.to_string().green());
    println!("  Failed: {}", failed.to_string().red());
    println!("  Success rate: {:.1}%", (successful as f64 / requests as f64) * 100.0);
    println!("  Total time: {:.2}s", total_time.as_secs_f64());
    println!("  Average rate: {:.1} req/s", requests as f64 / total_time.as_secs_f64());
    
    Ok(())
}

// Event handler for long polling
async fn handle_event<T>(
    bot: Bot,
    result: T,
) -> std::result::Result<(), vkteams_bot::error::BotError>
where
    T: serde::Serialize + std::fmt::Debug,
{
    debug!("Last event id: {:?}", bot.get_last_event_id().await);
    
    if let Ok(json_str) = serde_json::to_string_pretty(&result) {
        println!("{}", json_str.green());
    } else {
        println!("Event: {:?}", result);
    }
    
    Ok(())
}

// Validation functions

fn validate_file_id(file_id: &str) -> CliResult<()> {
    if file_id.trim().is_empty() {
        return Err(CliError::InputError("File ID cannot be empty".to_string()));
    }
    
    // Basic file ID format validation
    if !file_id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(CliError::InputError(
            "File ID contains invalid characters. Only alphanumeric, underscore, and hyphen are allowed".to_string()
        ));
    }
    
    Ok(())
}

fn validate_directory_path(dir_path: &str) -> CliResult<()> {
    if dir_path.trim().is_empty() {
        return Ok(()); // Empty path is allowed, will use default
    }
    
    let path = std::path::Path::new(dir_path);
    if path.exists() && !path.is_dir() {
        return Err(CliError::FileError(format!(
            "Path exists but is not a directory: {}", dir_path
        )));
    }
    
    Ok(())
}

// Utility functions

fn print_success_result<T>(result: &T) -> CliResult<()>
where
    T: serde::Serialize,
{
    let json_str = serde_json::to_string_pretty(result)
        .map_err(|e| CliError::UnexpectedError(format!("Failed to serialize response: {e}")))?;
    
    println!("{}", json_str.green());
    Ok(())
}