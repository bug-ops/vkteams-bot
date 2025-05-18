<<<<<<< HEAD
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
=======
use clap::{Parser, Subcommand};
use colored::Colorize;
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::error;
use vkteams_bot::prelude::*;
/// VKTeams CLI - Interacts with VK Teams API
pub struct Cli {
    /// bot instance
    pub bot: Arc<Bot>,
    /// matches from clap
    pub matches: Opts,
}
/// Default implementation for bot with API V1
impl Default for Cli {
    fn default() -> Self {
        Self {
            bot: Arc::new(Bot::default()),
            matches: Opts::parse(),
        }
    }
}
/// VKTeams CLI - Interacts with VK Teams API
#[derive(Parser, Clone, Debug)]
#[command(author="Andrei G.", version="0.5.2", about="vkteams-bot-cli tool", long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub subcmd: SubCommand,
}
/// Subcommands for VKTeams CLI
#[derive(Subcommand, Debug, Clone)]
pub enum SubCommand {
    /// Send text message text <MESSAGE> to user with <USER_ID>
    SendText {
        #[arg(short, long, required = true, value_name = "USER_ID")]
        user_id: String,
        #[arg(short, long, required = true, value_name = "MESSAGE")]
        message: String,
    },
    /// Send file from <FILE_PATH> to user with <USER_ID>
    SendFile {
        #[arg(short, long, required = true, value_name = "USER_ID")]
        user_id: String,
        #[arg(short, long, required = true, value_name = "FILE_PATH")]
        file_path: String,
    },
    /// Download file with <FILE_ID> into <FILE_PATH>
    GetFile {
        #[arg(short = 'f', long, required = true, value_name = "FILE_ID")]
        file_id: String,
        #[arg(short = 'p', long, required = true, value_name = "FILE_PATH")]
        file_path: String,
    },
    /// Get events once or listen with optional <LISTEN> flag
    GetEvents {
        #[arg(short, long, required = false, value_name = "LISTEN")]
        listen: Option<bool>,
    },
}
/// Implementation for CLI
impl Cli {
    /// Match input with subcommands
    pub async fn match_input(&self) -> Result<()> {
        let bot = Arc::clone(&self.bot);
        // Match subcommands
        match self.matches.subcmd.to_owned() {
            // Subcommand for send text message
            SubCommand::SendText { user_id, message } => {
                let parser = MessageTextParser::new().add(MessageTextFormat::Plain(message));
                let result = bot
                    .send_api_request(
                        RequestMessagesSendText::new(ChatId(user_id)).set_text(parser)?,
                    )
                    .await?;
                match_result(&result).await?;
            }
            // Subcommand for send file
            SubCommand::SendFile { user_id, file_path } => {
                let result = bot
                    .send_api_request(RequestMessagesSendFile::new((
                        ChatId(user_id),
                        MultipartName::File(file_path),
                    )))
                    .await?;
                match_result(&result).await?;
            }
            // Subcommand for get events
            SubCommand::GetEvents { listen } => {
                match listen {
                    Some(true) => bot.event_listener(match_events).await?,
                    _ => {
                        let result = bot
                            .send_api_request(RequestEventsGet::new(bot.get_last_event_id().await))
                            .await?;
                        match_result(&result).await?
                    }
                };
            }
            // Subcommand for get file from id
            SubCommand::GetFile { file_id, file_path } => {
                let result = bot
                    .send_api_request(RequestFilesGetInfo::new(FileId(file_id)))
                    .await?;
                // Download file data
                let file_data = result.download(Client::new()).await?;
                // Save file to the disk
                file_save(&result.file_name.to_owned(), &file_path, file_data).await;
            }
        }
        Ok(())
    }
}
/// Match result and print it
pub async fn match_events<T>(_: Bot, result: T) -> Result<()>
where
    T: serde::Serialize,
{
    match_result(&result).await
}
/// Match result and print it
pub async fn match_result<T>(result: &T) -> Result<()>
where
    T: serde::Serialize,
{
    println!("{}", serde_json::to_string(result)?.green());
    Ok(())
}
/// Save file on disk
pub async fn file_save(file_name: &str, path: &str, file_data: Vec<u8>) {
    let mut file_path = PathBuf::from(path);
    file_path.push(file_name);
    match tokio::fs::write(&file_path, file_data).await {
        Ok(_) => {
            println!(
                "File saved to: `{}`",
                file_path.display().to_string().green()
            );
        }
        Err(e) => {
            error!("Error: {}", e);
            println!("File not saved: {}", e.to_string().red());
        }
    }
>>>>>>> 3f6c614 (move cli)
}
