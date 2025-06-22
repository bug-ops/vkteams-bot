//! Diagnostic commands module
//!
//! This module contains all commands related to diagnostics, testing, and system information.

use crate::commands::{Command, OutputFormat};
use crate::config::Config;
use crate::constants::ui::emoji;
use crate::errors::prelude::{CliError, Result as CliResult};
use crate::file_utils;
use crate::output::{CliResponse, OutputFormatter};
use crate::utils::output::print_success_result;
use crate::utils::{validate_directory_path, validate_file_id};
use async_trait::async_trait;
use clap::{Subcommand, ValueHint};
use colored::Colorize;
use serde_json::json;
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
        #[arg(short = 'p', long, required = false, value_name = "FILE_PATH", value_hint = ValueHint::DirPath)]
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
            DiagnosticCommands::GetSelf { detailed } => execute_get_self(bot, *detailed).await,
            DiagnosticCommands::GetEvents { listen } => {
                execute_get_events(bot, listen.unwrap_or(false)).await
            }
            DiagnosticCommands::GetFile { file_id, file_path } => {
                execute_get_file(bot, file_id, file_path).await
            }
            DiagnosticCommands::HealthCheck => execute_health_check(bot).await,
            DiagnosticCommands::NetworkTest => execute_network_test(bot).await,
            DiagnosticCommands::SystemInfo => execute_system_info().await,
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
            DiagnosticCommands::RateLimitTest {
                requests,
                delay_ms: _,
            } => {
                if *requests == 0 || *requests > 1000 {
                    return Err(CliError::InputError(
                        "Number of requests must be between 1 and 1000".to_string(),
                    ));
                }
            }
            _ => {} // Other commands don't need validation
        }
        Ok(())
    }

    /// New method for structured output support
    async fn execute_with_output(&self, bot: &Bot, output_format: &OutputFormat) -> CliResult<()> {
        let response = match self {
            DiagnosticCommands::GetSelf { detailed } => {
                execute_get_self_structured(bot, *detailed).await
            }
            DiagnosticCommands::GetEvents { listen } => {
                execute_get_events_structured(bot, listen.unwrap_or(false)).await
            }
            DiagnosticCommands::GetFile { file_id, file_path } => {
                execute_get_file_structured(bot, file_id, file_path).await
            }
            DiagnosticCommands::HealthCheck => execute_health_check_structured(bot).await,
            DiagnosticCommands::NetworkTest => execute_network_test_structured(bot).await,
            DiagnosticCommands::SystemInfo => execute_system_info_structured().await,
            DiagnosticCommands::RateLimitTest { requests, delay_ms } => {
                execute_rate_limit_test_structured(bot, *requests, *delay_ms).await
            }
        };

        OutputFormatter::print(&response, output_format)?;

        if !response.success {
            return Err(CliError::UnexpectedError("Command failed".to_string()));
        }

        Ok(())
    }
}

// Command execution functions

async fn execute_get_self(bot: &Bot, detailed: bool) -> CliResult<()> {
    debug!("Getting bot information");

    let request = RequestSelfGet::new(());
    let result = bot
        .send_api_request(request)
        .await
        .map_err(CliError::ApiError)?;

    if detailed {
        info!("Bot information retrieved successfully");
        print_success_result(&result, &OutputFormat::Pretty)?;
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
        println!(
            "{} Starting event listener. Press Ctrl+C to stop.",
            emoji::ROCKET
        );

        match bot.event_listener(handle_event).await {
            Ok(()) => (),
            Err(e) => return Err(CliError::ApiError(e)),
        }
    } else {
        let result = bot
            .send_api_request(RequestEventsGet::new(bot.get_last_event_id()).with_poll_time(30))
            .await
            .map_err(CliError::ApiError)?;

        info!("Successfully retrieved events");
        print_success_result(&result, &OutputFormat::Pretty)?;
    }

    Ok(())
}

async fn execute_get_file(bot: &Bot, file_id: &str, file_path: &str) -> CliResult<()> {
    debug!("Downloading file {} to {}", file_id, file_path);

    let downloaded_path = file_utils::download_and_save_file(bot, file_id, file_path).await?;

    info!("Successfully downloaded file with ID: {}", file_id);
    println!(
        "{} File downloaded to: {}",
        emoji::CHECK,
        downloaded_path.display().to_string().green()
    );

    Ok(())
}

async fn execute_health_check(bot: &Bot) -> CliResult<()> {
    println!(
        "{} Performing comprehensive health check...",
        emoji::TEST_TUBE.bold().blue()
    );
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
                println!(
                    "{} - High latency: {}ms",
                    "WARN".yellow(),
                    latency.as_millis()
                );
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
        println!(
            "{} Some health checks failed. Check configuration and network connectivity.",
            emoji::WARNING.bold().yellow()
        );
    }

    Ok(())
}

async fn execute_network_test(bot: &Bot) -> CliResult<()> {
    println!(
        "{} Testing network connectivity...",
        emoji::GEAR.bold().blue()
    );
    println!();

    // Test multiple endpoints with timing
    let endpoints = vec![("Bot Info", RequestSelfGet::new(()))];

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
    println!(
        "{} Testing rate limits with {} requests ({}ms delay)...",
        emoji::ROCKET.bold().blue(),
        requests,
        delay_ms
    );
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
                println!(
                    "Request {}/{}: {} ({}ms)",
                    i,
                    requests,
                    "OK".green(),
                    duration.as_millis()
                );
            }
            Err(e) => {
                failed += 1;
                println!("Request {}/{}: {} - {}", i, requests, "FAILED".red(), e);
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
    println!(
        "  Success rate: {:.1}%",
        (successful as f64 / requests as f64) * 100.0
    );
    println!("  Total time: {:.2}s", total_time.as_secs_f64());
    println!(
        "  Average rate: {:.1} req/s",
        requests as f64 / total_time.as_secs_f64()
    );

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
    debug!("Last event id: {:?}", bot.get_last_event_id());

    if let Ok(json_str) = serde_json::to_string_pretty(&result) {
        println!("{}", json_str.green());
    } else {
        println!("Event: {:?}", result);
    }

    Ok(())
}

// Validation functions are now imported from utils/validation module

// Structured output versions

async fn execute_get_self_structured(
    bot: &Bot,
    detailed: bool,
) -> CliResponse<serde_json::Value> {
    debug!("Getting bot information (structured)");

    let request = RequestSelfGet::new(());
    match bot.send_api_request(request).await {
        Ok(result) => {
            info!("Bot information retrieved successfully");
            let data = if detailed {
                serde_json::to_value(&result).unwrap_or(json!({}))
            } else {
                json!({
                    "bot_id": result.user_id,
                    "nickname": result.nick,
                    "first_name": result.first_name,
                    "about": result.about,
                    "photo": result.photo
                })
            };
            CliResponse::success("get-self", data)
        }
        Err(e) => CliResponse::error("get-self", format!("Failed to get bot info: {}", e)),
    }
}

async fn execute_get_events_structured(
    bot: &Bot,
    listen: bool,
) -> CliResponse<serde_json::Value> {
    debug!("Getting events, listen mode: {}", listen);

    if listen {
        // For listen mode, we can't return structured data easily
        // Return a message indicating the mode
        CliResponse::success(
            "get-events",
            json!({
                "mode": "listen",
                "message": "Event listener started. Press Ctrl+C to stop.",
                "note": "Use regular execute mode for event listening"
            }),
        )
    } else {
        match bot
            .send_api_request(RequestEventsGet::new(bot.get_last_event_id()).with_poll_time(30))
            .await
        {
            Ok(result) => {
                info!("Successfully retrieved events");
                let data = serde_json::to_value(&result).unwrap_or(json!({}));
                CliResponse::success("get-events", data)
            }
            Err(e) => CliResponse::error("get-events", format!("Failed to get events: {}", e)),
        }
    }
}

async fn execute_get_file_structured(
    bot: &Bot,
    file_id: &str,
    file_path: &str,
) -> CliResponse<serde_json::Value> {
    debug!("Downloading file {} to {}", file_id, file_path);

    match file_utils::download_and_save_file(bot, file_id, file_path).await {
        Ok(downloaded_path) => {
            info!("Successfully downloaded file with ID: {}", file_id);
            let data = json!({
                "file_id": file_id,
                "download_path": downloaded_path.display().to_string(),
                "status": "downloaded"
            });
            CliResponse::success("get-file", data)
        }
        Err(e) => CliResponse::error("get-file", format!("Failed to download file: {}", e)),
    }
}

async fn execute_health_check_structured(bot: &Bot) -> CliResponse<serde_json::Value> {
    debug!("Performing health check");

    let mut tests = Vec::new();
    let mut all_passed = true;

    // Test 1: Basic connectivity
    let start = std::time::Instant::now();
    let connectivity_result = match bot.send_api_request(RequestSelfGet::new(())).await {
        Ok(_) => {
            json!({
                "name": "API Connectivity",
                "status": "pass",
                "latency_ms": start.elapsed().as_millis()
            })
        }
        Err(e) => {
            all_passed = false;
            json!({
                "name": "API Connectivity",
                "status": "fail",
                "error": e.to_string()
            })
        }
    };
    tests.push(connectivity_result);

    // Test 2: Configuration check
    let config_result = match Config::from_file() {
        Ok(config) => {
            if config.api.token.is_some() && config.api.url.is_some() {
                json!({
                    "name": "Configuration",
                    "status": "pass",
                    "details": "All required fields present"
                })
            } else {
                all_passed = false;
                json!({
                    "name": "Configuration",
                    "status": "fail",
                    "error": "Missing required configuration"
                })
            }
        }
        Err(_) => {
            all_passed = false;
            json!({
                "name": "Configuration",
                "status": "fail",
                "error": "Configuration file not found"
            })
        }
    };
    tests.push(config_result);

    // Test 3: Network latency
    let latency_start = std::time::Instant::now();
    let latency_result = match bot.send_api_request(RequestSelfGet::new(())).await {
        Ok(_) => {
            let latency = latency_start.elapsed();
            let status = if latency.as_millis() < 1000 {
                "pass"
            } else {
                "warn"
            };
            json!({
                "name": "Network Latency",
                "status": status,
                "latency_ms": latency.as_millis()
            })
        }
        Err(e) => {
            all_passed = false;
            json!({
                "name": "Network Latency",
                "status": "fail",
                "error": e.to_string()
            })
        }
    };
    tests.push(latency_result);

    let data = json!({
        "overall_status": if all_passed { "healthy" } else { "unhealthy" },
        "tests": tests,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    CliResponse::success("health-check", data)
}

async fn execute_network_test_structured(bot: &Bot) -> CliResponse<serde_json::Value> {
    debug!("Testing network connectivity");

    let mut results = Vec::new();

    // Test multiple endpoints
    let endpoints = vec![("Bot Info", RequestSelfGet::new(()))];

    for (name, request) in endpoints {
        let start = std::time::Instant::now();

        let result = match bot.send_api_request(request).await {
            Ok(_) => {
                json!({
                    "endpoint": name,
                    "status": "ok",
                    "latency_ms": start.elapsed().as_millis()
                })
            }
            Err(e) => {
                json!({
                    "endpoint": name,
                    "status": "failed",
                    "error": e.to_string()
                })
            }
        };
        results.push(result);
    }

    let data = json!({
        "test_results": results,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    CliResponse::success("network-test", data)
}

async fn execute_system_info_structured() -> CliResponse<serde_json::Value> {
    debug!("Gathering system information");

    let mut env_vars = serde_json::Map::new();
    let vars = [
        "VKTEAMS_BOT_API_TOKEN",
        "VKTEAMS_BOT_API_URL",
        "VKTEAMS_PROXY",
        "VKTEAMS_LOG_LEVEL",
    ];

    for var in &vars {
        match std::env::var(var) {
            Ok(value) => {
                if var.contains("TOKEN") {
                    env_vars.insert(
                        var.to_string(),
                        json!(format!("{}***", &value[..8.min(value.len())])),
                    );
                } else {
                    env_vars.insert(var.to_string(), json!(value));
                }
            }
            Err(_) => {
                env_vars.insert(var.to_string(), json!("Not set"));
            }
        }
    }

    let config_status = match Config::from_file() {
        Ok(_) => "found",
        Err(_) => "not_found",
    };

    let data = json!({
        "runtime": {
            "os": std::env::consts::OS,
            "architecture": std::env::consts::ARCH,
            "family": std::env::consts::FAMILY,
            "current_directory": std::env::current_dir().ok().map(|p| p.display().to_string())
        },
        "environment": env_vars,
        "configuration": {
            "status": config_status
        }
    });

    CliResponse::success("system-info", data)
}

async fn execute_rate_limit_test_structured(
    bot: &Bot,
    requests: u32,
    delay_ms: u64,
) -> CliResponse<serde_json::Value> {
    debug!("Testing rate limits with {} requests", requests);

    let mut successful = 0;
    let mut failed = 0;
    let mut request_results = Vec::new();
    let start_time = std::time::Instant::now();

    for i in 1..=requests {
        let request_start = std::time::Instant::now();

        let result = match bot.send_api_request(RequestSelfGet::new(())).await {
            Ok(_) => {
                successful += 1;
                json!({
                    "request_number": i,
                    "status": "success",
                    "latency_ms": request_start.elapsed().as_millis()
                })
            }
            Err(e) => {
                failed += 1;
                json!({
                    "request_number": i,
                    "status": "failed",
                    "error": e.to_string()
                })
            }
        };
        request_results.push(result);

        if i < requests {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
        }
    }

    let total_time = start_time.elapsed();
    let success_rate = (successful as f64 / requests as f64) * 100.0;
    let average_rate = requests as f64 / total_time.as_secs_f64();

    let data = json!({
        "summary": {
            "total_requests": requests,
            "successful": successful,
            "failed": failed,
            "success_rate_percent": success_rate,
            "total_time_seconds": total_time.as_secs_f64(),
            "average_rate_per_second": average_rate,
            "delay_between_requests_ms": delay_ms
        },
        "request_details": request_results
    });

    CliResponse::success("rate-limit-test", data)
}

// Utility functions

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    fn dummy_bot() -> Bot {
        Bot::with_params(&APIVersionUrl::V1, "dummy_token", "https://dummy.api.com").unwrap()
    }

    #[test]
    fn test_validate_get_file_empty_file_id() {
        let cmd = DiagnosticCommands::GetFile {
            file_id: "".to_string(),
            file_path: "/tmp".to_string(),
        };
        let res = cmd.validate();
        assert!(res.is_err());
    }

    #[test]
    fn test_validate_get_file_invalid_path() {
        let cmd = DiagnosticCommands::GetFile {
            file_id: "fileid".to_string(),
            file_path: "".to_string(),
        };
        let res = cmd.validate();
        assert!(res.is_ok()); // пустой путь допустим
    }

    #[test]
    fn test_validate_rate_limit_zero() {
        let cmd = DiagnosticCommands::RateLimitTest {
            requests: 0,
            delay_ms: 100,
        };
        let res = cmd.validate();
        assert!(res.is_err());
    }

    #[test]
    fn test_validate_rate_limit_too_many() {
        let cmd = DiagnosticCommands::RateLimitTest {
            requests: 1001,
            delay_ms: 100,
        };
        let res = cmd.validate();
        assert!(res.is_err());
    }

    #[test]
    fn test_validate_rate_limit_valid() {
        let cmd = DiagnosticCommands::RateLimitTest {
            requests: 10,
            delay_ms: 100,
        };
        let res = cmd.validate();
        assert!(res.is_ok());
    }

    #[test]
    fn test_execute_get_self_api_error() {
        let cmd = DiagnosticCommands::GetSelf { detailed: true };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }

    #[test]
    fn test_execute_get_events_api_error() {
        let cmd = DiagnosticCommands::GetEvents {
            listen: Some(false),
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }

    #[test]
    fn test_execute_get_file_api_error() {
        let cmd = DiagnosticCommands::GetFile {
            file_id: "fileid".to_string(),
            file_path: "/tmp".to_string(),
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }

    #[test]
    fn test_execute_health_check_api_error() {
        let cmd = DiagnosticCommands::HealthCheck;
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_ok());
    }

    #[test]
    fn test_execute_network_test_api_error() {
        let cmd = DiagnosticCommands::NetworkTest;
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_ok());
    }

    #[test]
    fn test_execute_system_info() {
        let cmd = DiagnosticCommands::SystemInfo;
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        // SystemInfo не требует bot, но для совместимости передаём
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_ok());
    }

    #[test]
    fn test_execute_rate_limit_test_api_error() {
        let cmd = DiagnosticCommands::RateLimitTest {
            requests: 2,
            delay_ms: 10,
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_ok());
    }

    #[test]
    fn test_diagnostic_commands_variants() {
        let get_self = DiagnosticCommands::GetSelf { detailed: true };
        assert_eq!(get_self.name(), "get-self");
        if let DiagnosticCommands::GetSelf { detailed } = get_self {
            assert!(detailed);
        }

        let get_events = DiagnosticCommands::GetEvents { listen: Some(true) };
        assert_eq!(get_events.name(), "get-events");
        if let DiagnosticCommands::GetEvents { listen } = get_events {
            assert_eq!(listen, Some(true));
        }

        let get_file = DiagnosticCommands::GetFile {
            file_id: "file123".to_string(),
            file_path: "/tmp".to_string(),
        };
        assert_eq!(get_file.name(), "get-file");
        if let DiagnosticCommands::GetFile { file_id, file_path } = get_file {
            assert_eq!(file_id, "file123");
            assert_eq!(file_path, "/tmp");
        }

        let health = DiagnosticCommands::HealthCheck;
        assert_eq!(health.name(), "health-check");

        let net = DiagnosticCommands::NetworkTest;
        assert_eq!(net.name(), "network-test");

        let sys = DiagnosticCommands::SystemInfo;
        assert_eq!(sys.name(), "system-info");

        let rate = DiagnosticCommands::RateLimitTest {
            requests: 10,
            delay_ms: 100,
        };
        assert_eq!(rate.name(), "rate-limit-test");
        if let DiagnosticCommands::RateLimitTest { requests, delay_ms } = rate {
            assert_eq!(requests, 10);
            assert_eq!(delay_ms, 100);
        }
    }
}
