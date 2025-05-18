use crate::config::Config;
use crate::errors::prelude::{CliError, Result as CliResult};
use crate::file_utils;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fmt::Debug;
use tracing::{debug, error, info};
use vkteams_bot::prelude::*;
/// VKTeams CLI - Interacts with VK Teams API
pub struct Cli {
    /// bot instance
    pub bot: Bot,
    /// matches from clap
    pub matches: Opts,
    /// configuration
    pub config: Config,
}
/// Default implementation for bot with API V1
impl Default for Cli {
    fn default() -> Self {
        debug!("Creating default CLI instance");
        Self {
            bot: Bot::default(),
            matches: Opts::parse(),
            config: Config::default(),
        }
    }
}

impl Cli {
    /// Create a new CLI instance with the provided configuration
    pub fn with_config(config: Config) -> Self {
        debug!("Creating CLI instance with custom configuration");

        // Set environment variables from config to initialize bot properly
        if let Some(url) = &config.api.url {
            unsafe {
                std::env::set_var("VKTEAMS_BOT_API_URL", url);
            }
        }

        if let Some(token) = &config.api.token {
            unsafe {
                std::env::set_var("VKTEAMS_BOT_API_TOKEN", token);
            }
        }

        if let Some(proxy) = &config.proxy {
            unsafe {
                std::env::set_var("VKTEAMS_PROXY", &proxy.url);
            }
        }

        // Now create Bot with environment variables set
        let bot = Bot::default();

        Self {
            bot,
            matches: Opts::parse(),
            config,
        }
    }
}
/// VKTeams CLI - Interacts with VK Teams API
#[derive(Parser, Clone, Debug)]
#[command(author="Andrei G.", version="0.6.0", about="vkteams-bot-cli tool", long_about = None)]
pub struct Opts {
    /// Path to config file (overrides default locations)
    #[arg(short, long)]
    pub config: Option<String>,

    /// Save current configuration to file
    #[arg(long, value_name = "PATH")]
    pub save_config: Option<String>,

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
        #[arg(short, long, required = false, value_name = "FILE_PATH")]
        file_path: String,
    },
    /// Download file with <FILE_ID> into <FILE_PATH>
    GetFile {
        #[arg(short = 'f', long, required = true, value_name = "FILE_ID")]
        file_id: String,
        #[arg(short = 'p', long, required = false, value_name = "FILE_PATH")]
        file_path: String,
    },
    /// Get events once or listen with optional <LISTEN> flag
    GetEvents {
        #[arg(short, long, required = false, value_name = "LISTEN")]
        listen: Option<bool>,
    },
    /// Configure the CLI tool
    Config {
        /// Show current configuration
        #[arg(short, long)]
        show: bool,
        /// Initialize a new configuration file
        #[arg(short, long)]
        init: bool,
    },
}
/// Implementation for CLI
impl Cli {
    /// Match input with subcommands
    pub async fn match_input(&self) -> CliResult<()> {
        debug!("Match input with subcommands");

        // Check if we need to save configuration
        if let Some(path) = &self.matches.save_config {
            debug!("Saving configuration to {}", path);
            self.config.save(Some(std::path::Path::new(path)))?;
            println!("Configuration saved to: {}", path.green());
            return Ok(());
        }

        // Match subcommands
        match self.matches.subcmd.to_owned() {
            // Subcommand for send text message
            SubCommand::SendText { user_id, message } => {
                debug!("Send text message");
                let parser = MessageTextParser::new().add(MessageTextFormat::Plain(message));

                let request =
                    match RequestMessagesSendText::new(ChatId(user_id.clone())).set_text(parser) {
                        Ok(req) => req,
                        Err(e) => {
                            return Err(CliError::InputError(format!(
                                "Failed to set message text: {}",
                                e
                            )));
                        }
                    };

                let result = match self.bot.send_api_request(request).await {
                    Ok(res) => res,
                    Err(e) => return Err(CliError::ApiError(e)),
                };

                info!("Successfully sent text message to user: {}", user_id);
                match_result(&result).await?;
            }
            // Subcommand for send file
            SubCommand::SendFile { user_id, file_path } => {
                debug!("Send file");

                // Use the file_utils for file validation and streaming upload
                let result =
                    file_utils::upload_file(&self.bot, &user_id, &file_path, &self.config).await?;

                info!("Successfully sent file to user: {}", user_id);
                match_result(&result).await?;
            }
            // Subcommand for get events
            SubCommand::GetEvents { listen } => {
                debug!("Get events");
                match listen {
                    Some(true) => {
                        info!("Starting event listener (long polling)...");
                        match self.bot.event_listener(match_events).await {
                            Ok(_) => (),
                            Err(e) => return Err(CliError::ApiError(e)),
                        }
                    }
                    _ => {
                        let result = match self
                            .bot
                            .send_api_request(RequestEventsGet::new(
                                self.bot.get_last_event_id().await,
                            ))
                            .await
                        {
                            Ok(res) => res,
                            Err(e) => return Err(CliError::ApiError(e)),
                        };

                        info!("Successfully retrieved events");
                        match_result(&result).await?
                    }
                };
            }
            // Subcommand for get file from id
            SubCommand::GetFile { file_id, file_path } => {
                debug!("Get file");

                // Use the file_utils for streaming download
                file_utils::download_and_save_file(&self.bot, &file_id, &file_path, &self.config)
                    .await?;

                info!("Successfully downloaded file with ID: {}", file_id);
            }
            SubCommand::Config { show, init } => {
                if show {
                    // Print current configuration as TOML
                    match toml::to_string_pretty(&self.config) {
                        Ok(config_str) => {
                            println!("Current configuration:\n{}", config_str.green());
                        }
                        Err(e) => {
                            return Err(CliError::UnexpectedError(format!(
                                "Failed to serialize configuration: {}",
                                e
                            )));
                        }
                    }
                }

                if init {
                    // Create a default configuration file in the home directory
                    self.config.save(None)?;
                    println!("Configuration file initialized.");
                }

                // If no flags provided, show help
                if !show && !init {
                    println!(
                        "Use --show to display current configuration or --init to create a new configuration file."
                    );
                }
            }
        }
        Ok(())
    }
}
/// Match result and print it
pub async fn match_events<T>(
    bot: Bot,
    result: T,
) -> std::result::Result<(), vkteams_bot::error::BotError>
where
    T: serde::Serialize,
    T: Debug,
{
    debug!("Last event id: {:?}", bot.get_last_event_id().await);
    if let Err(err) = match_result(&result).await {
        error!("Error processing event: {}", err);
        return Err(vkteams_bot::error::BotError::System(err.to_string()));
    }
    Ok(())
}
/// Match result and print it
pub async fn match_result<T>(result: &T) -> CliResult<()>
where
    T: serde::Serialize,
    T: Debug,
{
    debug!("Result: {:?}", result);
    let json_str = serde_json::to_string(result)
        .map_err(|e| CliError::UnexpectedError(format!("Failed to serialize response: {}", e)))?;

    println!("{}", json_str.green());
    Ok(())
}
// file_save functionality has been moved to file_utils.rs
