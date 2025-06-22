pub mod cli;
pub mod commands;
pub mod completion;
pub mod config;
pub mod constants;
pub mod errors;
pub mod file_utils;
pub mod output;
pub mod progress;
pub mod scheduler;
pub mod utils;

use crate::cli::Cli;
use clap::Parser;
use colored::Colorize;
use commands::{Command, Commands, OutputFormat};
use config::{Config, UnifiedConfigAdapter};
use constants::{exit_codes, ui::emoji};
use errors::prelude::Result as CliResult;
use std::path::Path;
use std::process::exit;
use tracing::debug;
use utils::{create_bot_instance, create_dummy_bot, needs_bot_instance};
use vkteams_bot::otlp;

/// Main CLI structure for the VK Teams Bot command-line interface.
///
/// This structure defines all command-line arguments and options available in the
/// VK Teams Bot CLI application. It uses the `clap` derive API to automatically
/// generate argument parsing, help text, and validation logic.
///
/// # Global Options
///
/// The CLI provides several global options that affect application behavior:
/// - Configuration file management (custom config, save config)
/// - Output control (verbose logging, output format)
/// - Subcommand selection for specific operations
///
/// # Usage Examples
///
/// ## Basic Message Sending
/// ```bash
/// vkteams-bot-cli send-text -u USER_ID -m "Hello World!"
/// ```
///
/// ## Custom Configuration
/// ```bash
/// vkteams-bot-cli --config /path/to/config.toml send-file -u USER_ID -p file.pdf
/// ```
///
/// ## Verbose Output with JSON Format
/// ```bash
/// vkteams-bot-cli --verbose --output json get-chat-info -c CHAT_ID
/// ```
///
/// ## Configuration Management
/// ```bash
/// vkteams-bot-cli --save-config /backup/config.toml validate
/// ```
///
/// # Configuration Precedence
///
/// The CLI loads configuration from multiple sources in order of precedence:
/// 1. Custom config file (if `--config` is specified)
/// 2. Environment variables (VKTEAMS_*)
/// 3. Default config locations (~/.config/vkteams-bot/cli_config.toml)
/// 4. Built-in defaults
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let _guard = otlp::init()?;

    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize logging based on verbosity
    if cli.verbose {
        unsafe {
            std::env::set_var("RUST_LOG", "debug");
        }
    }

    // Load configuration
    let config = match load_configuration(&cli) {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "{} {}",
                emoji::CROSS,
                format!("Failed to load configuration: {}", err).red()
            );
            exit(exit_codes::CONFIG);
        }
    };

    // Handle config save if requested
    if let Some(path) = &cli.save_config {
        if let Err(err) = save_configuration(&config, path) {
            eprintln!(
                "{} {}",
                emoji::CROSS,
                format!("Failed to save configuration: {}", err).red()
            );
            exit(exit_codes::CONFIG);
        }
        println!(
            "{} Configuration saved to: {}",
            emoji::FLOPPY_DISK,
            path.green()
        );
        return Ok(());
    }

    debug!("Configuration loaded");
    // Validate command before execution
    if let Err(err) = cli.command.validate() {
        eprintln!(
            "{} {}",
            emoji::CROSS,
            format!("Validation error: {}", err).red()
        );
        exit(exit_codes::USAGE_ERROR);
    }

    // Execute command
    match execute_command(&cli.command, &config, &cli.output).await {
        Ok(()) => {
            debug!("Command executed successfully");
        }
        Err(err) => {
            eprintln!("{} {}", emoji::CROSS, format!("Error: {}", err).red());
            exit(err.exit_code());
        }
    }

    Ok(())
}

fn load_configuration(cli: &Cli) -> CliResult<Config> {
    // Load from custom path if provided using unified adapter
    if let Some(config_path) = &cli.config {
        UnifiedConfigAdapter::load_from_path(Path::new(config_path))
    } else {
        // Try unified adapter first, fall back to legacy
        UnifiedConfigAdapter::load()
    }
}

/// # Ok(())
/// # }
/// ```
///
/// # File Format
///
/// The saved file will be in TOML format with sections like:
/// ```toml
/// [api]
/// token = "your_bot_token"
/// url = "https://api.example.com"
///
/// [files]
/// download_dir = "/downloads"
/// max_file_size = 104857600
///
/// [logging]
/// level = "info"
/// colors = true
/// ```
fn save_configuration(config: &Config, path: &str) -> CliResult<()> {
    config.save(Some(Path::new(path)))
}

/// Execute command
async fn execute_command(
    command: &Commands,
    config: &Config,
    output_format: &OutputFormat,
) -> CliResult<()> {
    let bot = if needs_bot_instance(command) {
        create_bot_instance(config)?
    } else {
        create_dummy_bot()
    };

    // Handle commands that support unified JSON output
    match command {
        Commands::Files(cmd) => cmd.execute_with_output(&bot, output_format).await,
        Commands::Storage(cmd) => cmd.execute_with_output(&bot, output_format).await,
        Commands::Messaging(cmd) => cmd.execute_with_output(&bot, output_format).await,
        Commands::Chat(cmd) => cmd.execute_with_output(&bot, output_format).await,
        Commands::Daemon(cmd) => cmd.execute_with_output(&bot, output_format).await,
        Commands::Diagnostic(cmd) => cmd.execute_with_output(&bot, output_format).await,
        _ => {
            // Fall back to legacy execute method for other commands
            command.execute(&bot).await
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_main_runs() {
        // Smoke test для main (async main не вызывается напрямую)
        // This test just ensures main function exists and compiles
    }
}
