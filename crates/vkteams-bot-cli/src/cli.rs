use crate::commands::{Commands, OutputFormat};
use clap::{Parser, ValueHint};

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
#[derive(Parser, Debug)]
#[command(
    name = "vkteams-bot-cli",
    version = "0.6.0",
    about = "VK Teams Bot CLI tool",
    long_about = "A powerful command-line interface for interacting with VK Teams Bot API"
)]
pub struct Cli {
    /// Path to config file (overrides default locations)
    #[arg(short, long, value_name = "CONFIG", value_hint = ValueHint::FilePath)]
    pub config: Option<String>,

    /// Save current configuration to file
    #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath)]
    pub save_config: Option<String>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Output format
    #[arg(long, value_enum, default_value = "pretty")]
    pub output: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}
