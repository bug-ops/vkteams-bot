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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{CommandFactory, Parser};

    #[test]
    fn test_cli_parsing_config_and_verbose() {
        let args = [
            "vkteams-bot-cli",
            "--config",
            "myconfig.toml",
            "--save-config",
            "out.toml",
            "--verbose",
            "config",
        ];
        let cli = Cli::parse_from(&args);
        assert_eq!(cli.config.as_deref(), Some("myconfig.toml"));
        assert_eq!(cli.save_config.as_deref(), Some("out.toml"));
        assert!(cli.verbose);
    }

    #[test]
    fn test_cli_output_format_json() {
        let args = ["vkteams-bot-cli", "--output", "json", "config"];
        let cli = Cli::parse_from(&args);
        match cli.output {
            OutputFormat::Json => (),
            _ => panic!("Expected OutputFormat::Json"),
        }
    }

    #[test]
    fn test_cli_help_and_version() {
        Cli::command().debug_assert();
        let mut help_buf = Vec::new();
        Cli::command().write_long_help(&mut help_buf).unwrap();
        let help = String::from_utf8(help_buf).unwrap();
        assert!(
            help.contains(
                "A powerful command-line interface for interacting with VK Teams Bot API"
            )
        );
        let version = Cli::command().get_version().unwrap().to_string();
        assert!(version.starts_with("0."));
    }

    #[test]
    fn test_cli_invalid_output_format() {
        let args = ["vkteams-bot-cli", "--output", "not_a_format", "config"];
        let res = Cli::try_parse_from(&args);
        assert!(res.is_err());
    }
}
