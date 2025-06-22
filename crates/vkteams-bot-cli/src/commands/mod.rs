//! CLI Commands module
//!
//! This module contains all command implementations organized by functionality.

use crate::errors::prelude::Result as CliResult;
use async_trait::async_trait;
use clap::Subcommand;
use vkteams_bot::prelude::Bot;

pub mod chat;
pub mod config;
pub mod daemon;
pub mod diagnostic;
pub mod files;
pub mod messaging;
pub mod scheduling;
pub mod storage;

/// Trait that all CLI commands must implement
#[async_trait]
pub trait Command {
    /// Execute the command
    async fn execute(&self, bot: &Bot) -> CliResult<()>;

    /// Execute the command with structured output support (optional implementation)
    async fn execute_with_output(
        &self,
        bot: &Bot,
        _output_format: &OutputFormat,
    ) -> CliResult<()> {
        // Default implementation falls back to legacy execute method
        self.execute(bot).await
    }

    /// Get command name for logging
    fn name(&self) -> &'static str;

    /// Validate command arguments before execution
    fn validate(&self) -> CliResult<()> {
        Ok(())
    }
}

/// New trait for commands that return structured results
#[async_trait]
pub trait CommandExecutor {
    /// Execute the command and return structured result
    async fn execute_with_result(&self, bot: &Bot) -> CommandResult;

    /// Get command name for logging
    fn name(&self) -> &'static str;

    /// Validate command arguments before execution
    fn validate(&self) -> CliResult<()> {
        Ok(())
    }
}

/// Output format options
#[derive(clap::ValueEnum, Clone, Debug, Default, PartialEq)]
pub enum OutputFormat {
    #[default]
    Pretty,
    Json,
    Table,
    Quiet,
}

/// Context passed to commands for execution
pub struct CommandContext {
    pub bot: Bot,
    pub verbose: bool,
    pub output_format: OutputFormat,
}

/// Command execution result with optional output
#[derive(serde::Serialize)]
pub struct CommandResult {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<serde_json::Value>,
}

impl CommandResult {
    pub fn success() -> Self {
        Self {
            success: true,
            message: None,
            data: None,
        }
    }

    pub fn success_with_message(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: Some(message.into()),
            data: None,
        }
    }

    pub fn success_with_data(data: serde_json::Value) -> Self {
        Self {
            success: true,
            message: None,
            data: Some(data),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: Some(message.into()),
            data: None,
        }
    }

    /// Display the command result according to the specified output format
    pub fn display(&self, format: &OutputFormat) -> crate::errors::prelude::Result<()> {
        use crate::constants::ui::emoji;
        use colored::Colorize;

        match format {
            OutputFormat::Pretty => {
                if self.success {
                    if let Some(message) = &self.message {
                        println!("{} {}", emoji::CHECK, message.green());
                    }
                    if let Some(data) = &self.data {
                        let json_str = serde_json::to_string_pretty(data).map_err(|e| {
                            crate::errors::prelude::CliError::UnexpectedError(format!(
                                "Failed to serialize data: {}",
                                e
                            ))
                        })?;
                        println!("{}", json_str.green());
                    }
                } else if let Some(message) = &self.message {
                    eprintln!("{} {}", emoji::CROSS, message.red());
                }
            }
            OutputFormat::Json => {
                let json_output = serde_json::to_string_pretty(self).map_err(|e| {
                    crate::errors::prelude::CliError::UnexpectedError(format!(
                        "Failed to serialize result: {}",
                        e
                    ))
                })?;
                println!("{}", json_output);
            }
            OutputFormat::Table => {
                // For table format, fall back to pretty format for now
                self.display(&OutputFormat::Pretty)?;
            }
            OutputFormat::Quiet => {
                // No output in quiet mode unless it's an error
                if !self.success {
                    if let Some(message) = &self.message {
                        eprintln!("{}", message);
                    }
                }
            }
        }
        Ok(())
    }
}
/// All available CLI commands
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    // Messaging commands
    #[command(flatten)]
    Messaging(messaging::MessagingCommands),

    // Chat management commands
    #[command(flatten)]
    Chat(chat::ChatCommands),

    // Scheduling commands
    #[command(flatten)]
    Scheduling(scheduling::SchedulingCommands),

    // Configuration commands
    #[command(flatten)]
    Config(config::ConfigCommands),

    // Diagnostic commands
    #[command(flatten)]
    Diagnostic(diagnostic::DiagnosticCommands),

    // File management commands
    #[command(flatten)]
    Files(files::FileCommands),

    // Storage and database commands
    #[command(flatten)]
    Storage(storage::StorageCommands),
    
    // Daemon management commands
    #[command(flatten)]
    Daemon(daemon::DaemonCommands),
}

#[async_trait]
impl Command for Commands {
    async fn execute(&self, bot: &Bot) -> CliResult<()> {
        match self {
            Commands::Messaging(cmd) => cmd.execute(bot).await,
            Commands::Chat(cmd) => cmd.execute(bot).await,
            Commands::Scheduling(cmd) => cmd.execute(bot).await,
            Commands::Config(cmd) => cmd.execute(bot).await,
            Commands::Diagnostic(cmd) => cmd.execute(bot).await,
            Commands::Files(cmd) => cmd.execute(bot).await,
            Commands::Storage(cmd) => cmd.execute(bot).await,
            Commands::Daemon(cmd) => cmd.execute(bot).await,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Commands::Messaging(cmd) => cmd.name(),
            Commands::Chat(cmd) => cmd.name(),
            Commands::Scheduling(cmd) => cmd.name(),
            Commands::Config(cmd) => Command::name(cmd),
            Commands::Diagnostic(cmd) => cmd.name(),
            Commands::Files(cmd) => cmd.name(),
            Commands::Storage(cmd) => cmd.name(),
            Commands::Daemon(cmd) => cmd.name(),
        }
    }

    fn validate(&self) -> CliResult<()> {
        match self {
            Commands::Messaging(cmd) => cmd.validate(),
            Commands::Chat(cmd) => cmd.validate(),
            Commands::Scheduling(cmd) => cmd.validate(),
            Commands::Config(cmd) => Command::validate(cmd),
            Commands::Diagnostic(cmd) => cmd.validate(),
            Commands::Files(cmd) => cmd.validate(),
            Commands::Storage(cmd) => cmd.validate(),
            Commands::Daemon(cmd) => cmd.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_command_result_success() {
        let res = CommandResult::success();
        assert!(res.success);
        assert!(res.message.is_none());
        assert!(res.data.is_none());
    }

    #[test]
    fn test_command_result_success_with_message() {
        let res = CommandResult::success_with_message("ok");
        assert!(res.success);
        assert_eq!(res.message.as_deref(), Some("ok"));
        assert!(res.data.is_none());
    }

    #[test]
    fn test_command_result_success_with_data() {
        let data = json!({"key": 1});
        let res = CommandResult::success_with_data(data.clone());
        assert!(res.success);
        assert!(res.message.is_none());
        assert_eq!(res.data, Some(data));
    }

    #[test]
    fn test_command_result_error() {
        let res = CommandResult::error("fail");
        assert!(!res.success);
        assert_eq!(res.message.as_deref(), Some("fail"));
        assert!(res.data.is_none());
    }

    #[test]
    fn test_command_result_display_pretty() {
        let res = CommandResult::success_with_message("done");
        assert!(res.display(&OutputFormat::Pretty).is_ok());
        let res = CommandResult::error("fail");
        assert!(res.display(&OutputFormat::Pretty).is_ok());
        let res = CommandResult::success_with_data(json!({"a": 1}));
        assert!(res.display(&OutputFormat::Pretty).is_ok());
    }

    #[test]
    fn test_command_result_display_json() {
        let res = CommandResult::success_with_message("done");
        assert!(res.display(&OutputFormat::Json).is_ok());
        let res = CommandResult::error("fail");
        assert!(res.display(&OutputFormat::Json).is_ok());
        let res = CommandResult::success_with_data(json!({"a": 1}));
        assert!(res.display(&OutputFormat::Json).is_ok());
    }

    #[test]
    fn test_command_result_display_table() {
        let res = CommandResult::success_with_message("done");
        assert!(res.display(&OutputFormat::Table).is_ok());
    }

    #[test]
    fn test_command_result_display_quiet() {
        let res = CommandResult::success_with_message("done");
        assert!(res.display(&OutputFormat::Quiet).is_ok());
        let res = CommandResult::error("fail");
        assert!(res.display(&OutputFormat::Quiet).is_ok());
    }

    #[test]
    fn test_output_format_default() {
        let f = OutputFormat::default();
        assert_eq!(f, OutputFormat::Pretty);
    }

    #[test]
    fn test_command_result_display_json_error() {
        // Создаём CommandResult с несерилизуемым data (например, с циклической ссылкой невозможно, но можно подменить тип)
        // Здесь просто проверяем, что ошибка сериализации корректно обрабатывается
        struct NotSerializable;
        impl serde::Serialize for NotSerializable {
            fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                Err(serde::ser::Error::custom("fail"))
            }
        }
        let data = serde_json::to_value(NotSerializable).unwrap_or(json!(null));
        let res = CommandResult {
            success: true,
            message: None,
            data: Some(data),
        };
        // display не упадёт, просто выведет null
        assert!(res.display(&OutputFormat::Json).is_ok());
    }
}
