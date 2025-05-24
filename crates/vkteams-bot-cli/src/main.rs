pub mod commands;
pub mod config;
pub mod constants;
pub mod errors;
pub mod file_utils;
pub mod progress;
pub mod scheduler;
pub mod utils;

use commands::{Commands, Command, CommandExecutor, CommandResult, OutputFormat};
use config::Config;
use constants::{ui::emoji, exit_codes};
use errors::prelude::Result as CliResult;
use utils::{create_bot_instance, create_dummy_bot, needs_bot_instance};
use clap::Parser;
use colored::Colorize;
use std::process::exit;
use tracing::debug;
use vkteams_bot::prelude::*;
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
#[derive(Parser, Debug)]
#[command(
    name = "vkteams-bot-cli",
    version = "0.6.0",
    about = "VK Teams Bot CLI tool",
    long_about = "A powerful command-line interface for interacting with VK Teams Bot API"
)]
pub struct Cli {
    /// Path to config file (overrides default locations)
    #[arg(short, long, value_name = "CONFIG")]
    pub config: Option<String>,

    /// Save current configuration to file
    #[arg(long, value_name = "PATH")]
    pub save_config: Option<String>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Output format
    #[arg(long, value_enum, default_value = "pretty")]
    pub output: OutputFormat,

    ///
    /// # Command Categories
    /// - **Messaging**: send-text, send-file, send-voice, edit-message, delete-message
    /// - **Chat Management**: get-chat-info, get-chat-members, set-chat-title
    /// - **Scheduling**: schedule, scheduler, task management
    /// - **Configuration**: setup, config, validate, examples
    /// - **Diagnostics**: get-self, get-events, health-check, system-info
    ///
    /// # Getting Help
    /// Use `--help` with any command to see detailed usage information:
    /// ```bash
    /// vkteams-bot-cli send-text --help
    /// vkteams-bot-cli scheduler --help
    /// ```
    #[command(subcommand)]
    pub command: Commands,
}



///
/// # Verbose output with JSON format
/// vkteams-bot-cli --verbose --output json validate
///
/// # Save current config
/// vkteams-bot-cli --save-config backup.toml setup
/// ```
///
/// # Environment Requirements
///
/// - **Tokio Runtime**: The function requires an async runtime (provided by #[tokio::main])
/// - **Network Access**: Required for API commands to reach VK Teams servers
/// - **File System**: Required for configuration and file operations
/// - **Environment Variables**: Optional but may be needed for configuration
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
    let config = match load_configuration(&cli).await {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{} {}", emoji::CROSS, format!("Failed to load configuration: {}", err).red());
            exit(exit_codes::CONFIG);
        }
    };
    
    // Handle config save if requested
    if let Some(path) = &cli.save_config {
        if let Err(err) = save_configuration(&config, path) {
            eprintln!("{} {}", emoji::CROSS, format!("Failed to save configuration: {}", err).red());
            exit(exit_codes::CONFIG);
        }
        println!("{} Configuration saved to: {}", emoji::FLOPPY_DISK, path.green());
        return Ok(());
    }

    debug!("Configuration loaded successfully");

    // Validate command before execution
    if let Err(err) = cli.command.validate() {
        eprintln!("{} {}", emoji::CROSS, format!("Validation error: {}", err).red());
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

async fn load_configuration(cli: &Cli) -> CliResult<Config> {
    // Load from custom path if provided
    if let Some(config_path) = &cli.config {
        debug!("Loading configuration from: {}", config_path);
        return Config::from_path(std::path::Path::new(config_path));
    }
    
    // Load from default locations
    Config::load()
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
/// url = "https://api.teams.vk.com"
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
    config.save(Some(std::path::Path::new(path)))
}

/// Execute command
async fn execute_command(command: &Commands, config: &Config, output_format: &OutputFormat) -> CliResult<()> {
    // Check if command needs bot instance
    let bot = if needs_bot_instance(command) {
        create_bot_instance(config)?
    } else {
        // Commands that don't need bot (like config, examples, etc.)
        create_dummy_bot()
    };

    // Try to execute with new CommandResult system first
    if let Some(result) = try_execute_with_result(command, &bot).await {
        return result.display(output_format);
    }

    // Fall back to old system for commands not yet migrated
    command.execute(&bot).await
}

/// Try to execute command with CommandResult system
async fn try_execute_with_result(command: &Commands, bot: &Bot) -> Option<CommandResult> {
    match command {
        Commands::Config(cmd) => Some(cmd.execute_with_result(bot).await),
        _ => None, // Commands not yet migrated to CommandResult
    }
}






