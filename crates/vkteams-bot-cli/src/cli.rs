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
use std::fmt::Debug;
use std::str::FromStr;
use tracing::{debug, error, info};
use vkteams_bot::prelude::*;
/// `VKTeams` CLI - Interacts with VK Teams API
pub struct Cli {
    /// bot instance (lazily initialized)
    pub bot: Option<Bot>,
    /// matches from clap
    pub matches: Opts,
    /// configuration
    pub config: Config,
}
/// Default implementation for bot with API V1
impl Default for Cli {
    fn default() -> Self {
        debug!("Creating default CLI instance");
        let config = Config::default();
        Self {
            bot: None,
            matches: Opts::parse(),
            config,
        }
    }
}

impl Cli {
    /// Create a new CLI instance with the provided configuration
    pub fn with_config(config: Config) -> Self {
        debug!("Creating CLI instance with custom configuration");

        // Set environment variables from config for later bot initialization
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

        Self {
            bot: None,
            matches: Opts::parse(),
            config,
        }
    }

    /// Initialize bot if not already initialized
    ///
    /// # Errors
    /// - Returns `CliError::InputError` if token or URL are missing
    /// - Returns `CliError::ApiError` if bot initialization fails
    fn ensure_bot(&mut self) -> CliResult<&Bot> {
        if self.bot.is_none() {
            debug!("Initializing bot instance");
            
            let token = self.config.api.token.as_ref()
                .ok_or_else(|| CliError::InputError(
                    "API token is required. Set VKTEAMS_BOT_API_TOKEN or configure via config file.".to_string()
                ))?;
            
            let url = self.config.api.url.as_ref()
                .ok_or_else(|| CliError::InputError(
                    "API URL is required. Set VKTEAMS_BOT_API_URL or configure via config file.".to_string()
                ))?;

            let bot = Bot::with_params(
                APIVersionUrl::V1,
                token.clone(),
                url.clone(),
            ).map_err(CliError::ApiError)?;

            self.bot = Some(bot);
        }

        Ok(self.bot.as_ref().unwrap())
    }
}
/// `VKTeams` CLI - Interacts with VK Teams API
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
/// Subcommands for `VKTeams` CLI
#[derive(Subcommand, Debug, Clone)]
pub enum SubCommand {
    /// Send text message text <MESSAGE> to user with <`USER_ID`>
    SendText {
        #[arg(short, long, required = true, value_name = "USER_ID")]
        user_id: String,
        #[arg(short, long, required = true, value_name = "MESSAGE")]
        message: String,
    },
    /// Send file from <`FILE_PATH`> to user with <`USER_ID`>
    SendFile {
        #[arg(short = 'u', long, required = true, value_name = "USER_ID")]
        user_id: String,
        #[arg(short = 'p', long, required = false, value_name = "FILE_PATH")]
        file_path: String,
    },
    /// Send voice message from <`FILE_PATH`> to user with <`USER_ID`>
    SendVoice {
        #[arg(short = 'u', long, required = true, value_name = "USER_ID")]
        user_id: String,
        #[arg(short = 'p', long, required = true, value_name = "FILE_PATH")]
        file_path: String,
    },

    /// Download file with <`FILE_ID`> into <`FILE_PATH`>
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
    /// Get chat information for <`CHAT_ID`>
    GetChatInfo {
        #[arg(short, long, required = true, value_name = "CHAT_ID")]
        chat_id: String,
    },
    /// Get user profile information for <`USER_ID`>
    GetProfile {
        #[arg(short, long, required = true, value_name = "USER_ID")]
        user_id: String,
    },
    /// Edit message with <`MESSAGE_ID`> in chat <`CHAT_ID`>
    EditMessage {
        #[arg(short = 'c', long, required = true, value_name = "CHAT_ID")]
        chat_id: String,
        #[arg(short = 'm', long, required = true, value_name = "MESSAGE_ID")]
        message_id: String,
        #[arg(short = 't', long, required = true, value_name = "NEW_TEXT")]
        new_text: String,
    },
    /// Delete message with <`MESSAGE_ID`> in chat <`CHAT_ID`>
    DeleteMessage {
        #[arg(short = 'c', long, required = true, value_name = "CHAT_ID")]
        chat_id: String,
        #[arg(short = 'm', long, required = true, value_name = "MESSAGE_ID")]
        message_id: String,
    },
    /// Pin message with <`MESSAGE_ID`> in chat <`CHAT_ID`>
    PinMessage {
        #[arg(short = 'c', long, required = true, value_name = "CHAT_ID")]
        chat_id: String,
        #[arg(short = 'm', long, required = true, value_name = "MESSAGE_ID")]
        message_id: String,
    },
    /// Unpin message with <`MESSAGE_ID`> in chat <`CHAT_ID`>
    UnpinMessage {
        #[arg(short = 'c', long, required = true, value_name = "CHAT_ID")]
        chat_id: String,
        #[arg(short = 'm', long, required = true, value_name = "MESSAGE_ID")]
        message_id: String,
    },
    /// Get chat members for <`CHAT_ID`>
    GetChatMembers {
        #[arg(short, long, required = true, value_name = "CHAT_ID")]
        chat_id: String,
        #[arg(short = 'c', long, value_name = "CURSOR")]
        cursor: Option<String>,
    },
    /// Set chat title for <`CHAT_ID`>
    SetChatTitle {
        #[arg(short = 'c', long, required = true, value_name = "CHAT_ID")]
        chat_id: String,
        #[arg(short = 't', long, required = true, value_name = "TITLE")]
        title: String,
    },
    /// Set chat description for <`CHAT_ID`>
    SetChatAbout {
        #[arg(short = 'c', long, required = true, value_name = "CHAT_ID")]
        chat_id: String,
        #[arg(short = 'a', long, required = true, value_name = "ABOUT")]
        about: String,
    },
    /// Send typing action to chat <`CHAT_ID`>
    SendAction {
        #[arg(short = 'c', long, required = true, value_name = "CHAT_ID")]
        chat_id: String,
        #[arg(short = 'a', long, required = true, value_name = "ACTION")]
        action: String,
    },
    /// Get bot information and status
    GetSelf {
        /// Show detailed bot information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Interactive setup wizard for first-time configuration
    Setup,
    /// Show examples of how to use the CLI
    Examples,
    /// Show detailed information about all available commands
    ListCommands,
    /// Validate current configuration and test bot connection
    Validate,
    /// Schedule a message to be sent later
    Schedule {
        /// Type of message to schedule
        #[command(subcommand)]
        message_type: ScheduleMessageType,
    },
    /// Manage the scheduler service
    Scheduler {
        #[command(subcommand)]
        action: SchedulerAction,
    },
    /// Manage scheduled tasks
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },
    /// Configure the CLI tool
    Config {
        /// Show current configuration
        #[arg(short, long)]
        show: bool,
        /// Initialize a new configuration file
        #[arg(short, long)]
        init: bool,
        /// Interactive configuration wizard
        #[arg(short = 'w', long)]
        wizard: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ScheduleMessageType {
    /// Schedule a text message
    Text {
        #[arg(short = 'u', long, required = true, value_name = "CHAT_ID")]
        chat_id: String,
        #[arg(short = 'm', long, required = true, value_name = "MESSAGE")]
        message: String,
        #[arg(short = 't', long, value_name = "TIME")]
        time: Option<String>,
        #[arg(short = 'c', long, value_name = "CRON")]
        cron: Option<String>,
        #[arg(short = 'i', long, value_name = "SECONDS")]
        interval: Option<u64>,
        #[arg(long, value_name = "RUNS")]
        max_runs: Option<u64>,
    },
    /// Schedule a file message
    File {
        #[arg(short = 'u', long, required = true, value_name = "CHAT_ID")]
        chat_id: String,
        #[arg(short = 'p', long, required = true, value_name = "FILE_PATH")]
        file_path: String,
        #[arg(short = 't', long, value_name = "TIME")]
        time: Option<String>,
        #[arg(short = 'c', long, value_name = "CRON")]
        cron: Option<String>,
        #[arg(short = 'i', long, value_name = "SECONDS")]
        interval: Option<u64>,
        #[arg(long, value_name = "RUNS")]
        max_runs: Option<u64>,
    },
    /// Schedule a voice message
    Voice {
        #[arg(short = 'u', long, required = true, value_name = "CHAT_ID")]
        chat_id: String,
        #[arg(short = 'p', long, required = true, value_name = "FILE_PATH")]
        file_path: String,
        #[arg(short = 't', long, value_name = "TIME")]
        time: Option<String>,
        #[arg(short = 'c', long, value_name = "CRON")]
        cron: Option<String>,
        #[arg(short = 'i', long, value_name = "SECONDS")]
        interval: Option<u64>,
        #[arg(long, value_name = "RUNS")]
        max_runs: Option<u64>,
    },
    /// Schedule a chat action
    Action {
        #[arg(short = 'u', long, required = true, value_name = "CHAT_ID")]
        chat_id: String,
        #[arg(short = 'a', long, required = true, value_name = "ACTION")]
        action: String,
        #[arg(short = 't', long, value_name = "TIME")]
        time: Option<String>,
        #[arg(short = 'c', long, value_name = "CRON")]
        cron: Option<String>,
        #[arg(short = 'i', long, value_name = "SECONDS")]
        interval: Option<u64>,
        #[arg(long, value_name = "RUNS")]
        max_runs: Option<u64>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum SchedulerAction {
    /// Start the scheduler daemon
    Start,
    /// Stop the scheduler daemon
    Stop,
    /// Show scheduler status
    Status,
    /// List all scheduled tasks
    List,
}

#[derive(Subcommand, Debug, Clone)]
pub enum TaskAction {
    /// Show details of a specific task
    Show {
        #[arg(required = true, value_name = "TASK_ID")]
        task_id: String,
    },
    /// Remove a scheduled task
    Remove {
        #[arg(required = true, value_name = "TASK_ID")]
        task_id: String,
    },
    /// Enable a disabled task
    Enable {
        #[arg(required = true, value_name = "TASK_ID")]
        task_id: String,
    },
    /// Disable an active task
    Disable {
        #[arg(required = true, value_name = "TASK_ID")]
        task_id: String,
    },
    /// Run a task immediately (one-time)
    Run {
        #[arg(required = true, value_name = "TASK_ID")]
        task_id: String,
    },
}

/// Implementation for CLI
impl Cli {
    /// Match input with subcommands
    ///
    /// # Errors
    /// - Returns `CliError::ApiError` if there is an error with the API
    /// - Returns `CliError::FileError` if there is an error with file operations
    /// - Returns `CliError::InputError` if there is an error with input validation
    /// - Returns `CliError::UnexpectedError` for unexpected errors
    pub async fn match_input(&mut self) -> CliResult<()> {
        debug!("Match input with subcommands");

        // Check if we need to save configuration
        if let Some(path) = &self.matches.save_config {
            debug!("Saving configuration to {}", path);
            self.config.save(Some(std::path::Path::new(path)))?;
            println!("Configuration saved to: {}", path.green());
            return Ok(());
        }

        // Match subcommands
        match self.matches.subcmd.clone() {
            // Subcommand for send text message
            SubCommand::SendText { user_id, message } => {
                debug!("Send text message");
                let bot = self.ensure_bot()?;
                let parser = MessageTextParser::new().add(MessageTextFormat::Plain(message));

                let request =
                    match RequestMessagesSendText::new(ChatId(user_id.clone())).set_text(parser) {
                        Ok(req) => req,
                        Err(e) => {
                            return Err(CliError::InputError(format!(
                                "Failed to set message text: {e}"
                            )));
                        }
                    };

                let result = match bot.send_api_request(request).await {
                    Ok(res) => res,
                    Err(e) => return Err(CliError::ApiError(e)),
                };

                info!("Successfully sent text message to user: {}", user_id);
                match_result(&result)?;
            }
            // Subcommand for send file
            SubCommand::SendFile { user_id, file_path } => {
                debug!("Send file");
                let config = self.config.clone();
                let bot = self.ensure_bot()?;

                // Use the file_utils for file validation and streaming upload
                let result =
                    file_utils::upload_file(bot, &user_id, &file_path, &config).await?;

                info!("Successfully sent file to user: {}", user_id);
                match_result(&result)?;
            }
            // Subcommand for get events
            SubCommand::GetEvents { listen } => {
                debug!("Get events");
                let bot = self.ensure_bot()?;
                
                if let Some(true) = listen {
                    info!("Starting event listener (long polling)...");
                    match bot.event_listener(match_events).await {
                        Ok(()) => (),
                        Err(e) => return Err(CliError::ApiError(e)),
                    }
                } else {
                    let result = match bot
                        .send_api_request(RequestEventsGet::new(bot.get_last_event_id().await))
                        .await
                    {
                        Ok(res) => res,
                        Err(e) => return Err(CliError::ApiError(e)),
                    };

                    info!("Successfully retrieved events");
                    match_result(&result)?;
                }
            }
            // Subcommand for get file from id
            SubCommand::GetFile { file_id, file_path } => {
                debug!("Get file");
                let config = self.config.clone();
                let bot = self.ensure_bot()?;

                // Use the file_utils for streaming download
                file_utils::download_and_save_file(bot, &file_id, &file_path, &config)
                    .await?;

                info!("Successfully downloaded file with ID: {}", file_id);
            }
            // Subcommand for sending voice messages
            SubCommand::SendVoice { user_id, file_path } => {
                debug!("Send voice message");
                let config = self.config.clone();
                let bot = self.ensure_bot()?;

                // Use the file_utils for voice message upload
                let result =
                    file_utils::upload_voice(&bot, &user_id, &file_path, &config).await?;

                info!("Successfully sent voice message to user: {}", user_id);
                match_result(&result)?;
            }

            // Subcommand for getting chat info
            SubCommand::GetChatInfo { chat_id } => {
                debug!("Get chat info");
                let bot = self.ensure_bot()?;

                let request = RequestChatsGetInfo::new(ChatId(chat_id.clone()));

                let result = match bot.send_api_request(request).await {
                    Ok(res) => res,
                    Err(e) => return Err(CliError::ApiError(e)),
                };

                info!("Successfully retrieved chat info for: {}", chat_id);
                match_result(&result)?;
            }
            // Subcommand for getting user profile
            SubCommand::GetProfile { user_id } => {
                debug!("Get user profile");
                let bot = self.ensure_bot()?;

                let request = RequestChatsGetInfo::new(ChatId(user_id.clone()));

                let result = match bot.send_api_request(request).await {
                    Ok(res) => res,
                    Err(e) => return Err(CliError::ApiError(e)),
                };

                info!("Successfully retrieved profile for user: {}", user_id);
                match_result(&result)?;
            }
            // Subcommand for editing messages
            SubCommand::EditMessage { chat_id, message_id, new_text } => {
                debug!("Edit message");
                let bot = self.ensure_bot()?;

                let parser = MessageTextParser::new().add(MessageTextFormat::Plain(new_text));
                let request = match RequestMessagesEditText::new((
                    ChatId(chat_id.clone()),
                    MsgId(message_id.clone()),
                ))
                .set_text(parser) {
                    Ok(req) => req,
                    Err(e) => {
                        return Err(CliError::InputError(format!(
                            "Failed to set message text: {e}"
                        )));
                    }
                };

                let result = match bot.send_api_request(request).await {
                    Ok(res) => res,
                    Err(e) => return Err(CliError::ApiError(e)),
                };

                info!("Successfully edited message {} in chat {}", message_id, chat_id);
                match_result(&result)?;
            }
            // Subcommand for deleting messages
            SubCommand::DeleteMessage { chat_id, message_id } => {
                debug!("Delete message");
                let bot = self.ensure_bot()?;

                let request = RequestMessagesDeleteMessages::new((
                    ChatId(chat_id.clone()),
                    MsgId(message_id.clone()),
                ));

                let result = match bot.send_api_request(request).await {
                    Ok(res) => res,
                    Err(e) => return Err(CliError::ApiError(e)),
                };

                info!("Successfully deleted message {} from chat {}", message_id, chat_id);
                match_result(&result)?;
            }
            // Subcommand for pinning messages
            SubCommand::PinMessage { chat_id, message_id } => {
                debug!("Pin message");
                let bot = self.ensure_bot()?;

                let request = RequestChatsPinMessage::new((
                    ChatId(chat_id.clone()),
                    MsgId(message_id.clone()),
                ));

                let result = match bot.send_api_request(request).await {
                    Ok(res) => res,
                    Err(e) => return Err(CliError::ApiError(e)),
                };

                info!("Successfully pinned message {} in chat {}", message_id, chat_id);
                match_result(&result)?;
            }
            // Subcommand for unpinning messages
            SubCommand::UnpinMessage { chat_id, message_id } => {
                debug!("Unpin message");
                let bot = self.ensure_bot()?;

                let request = RequestChatsUnpinMessage::new((
                    ChatId(chat_id.clone()),
                    MsgId(message_id.clone()),
                ));

                let result = match bot.send_api_request(request).await {
                    Ok(res) => res,
                    Err(e) => return Err(CliError::ApiError(e)),
                };

                info!("Successfully unpinned message {} from chat {}", message_id, chat_id);
                match_result(&result)?;
            }
            // Subcommand for getting chat members
            SubCommand::GetChatMembers { chat_id, cursor } => {
                debug!("Get chat members");
                let bot = self.ensure_bot()?;

                let mut request = RequestChatsGetMembers::new(ChatId(chat_id.clone()));
                if let Some(cursor_val) = cursor {
                    match cursor_val.parse::<u32>() {
                        Ok(cursor_num) => {
                            request = request.with_cursor(cursor_num);
                        }
                        Err(e) => {
                            return Err(CliError::InputError(format!(
                                "Invalid cursor value, must be a number: {e}"
                            )));
                        }
                    }
                }

                let result = match bot.send_api_request(request).await {
                    Ok(res) => res,
                    Err(e) => return Err(CliError::ApiError(e)),
                };

                info!("Successfully retrieved members for chat: {}", chat_id);
                match_result(&result)?;
            }
            // Subcommand for setting chat title
            SubCommand::SetChatTitle { chat_id, title } => {
                debug!("Set chat title");
                let bot = self.ensure_bot()?;

                let request = RequestChatsSetTitle::new((
                    ChatId(chat_id.clone()),
                    title.clone(),
                ));

                let result = match bot.send_api_request(request).await {
                    Ok(res) => res,
                    Err(e) => return Err(CliError::ApiError(e)),
                };

                info!("Successfully set title for chat {}: {}", chat_id, title);
                match_result(&result)?;
            }
            // Subcommand for setting chat description
            SubCommand::SetChatAbout { chat_id, about } => {
                debug!("Set chat description");
                let bot = self.ensure_bot()?;

                let request = RequestChatsSetAbout::new((
                    ChatId(chat_id.clone()),
                    about.clone(),
                ));

                let result = match bot.send_api_request(request).await {
                    Ok(res) => res,
                    Err(e) => return Err(CliError::ApiError(e)),
                };

                info!("Successfully set description for chat {}: {}", chat_id, about);
                match_result(&result)?;
            }
            // Subcommand for sending chat actions
            SubCommand::SendAction { chat_id, action } => {
                debug!("Send chat action");
                let bot = self.ensure_bot()?;

                let chat_action = match action.as_str() {
                    "typing" => ChatActions::Typing,
                    "looking" => ChatActions::Looking,
                    _ => {
                        return Err(CliError::InputError(format!(
                            "Unknown action: {}. Available actions: typing, looking", action
                        )));
                    }
                };

                let request = RequestChatsSendAction::new((
                    ChatId(chat_id.clone()),
                    chat_action,
                ));

                let result = match bot.send_api_request(request).await {
                    Ok(res) => res,
                    Err(e) => return Err(CliError::ApiError(e)),
                };

                info!("Successfully sent {} action to chat {}", action, chat_id);
                match_result(&result)?;
            }
            // Subcommand for getting bot information
            SubCommand::GetSelf { detailed } => {
                debug!("Get bot information");
                let bot = self.ensure_bot()?;

                let request = RequestSelfGet::new(());

                let result = match bot.send_api_request(request).await {
                    Ok(res) => res,
                    Err(e) => return Err(CliError::ApiError(e)),
                };

                if detailed {
                    info!("Bot information retrieved successfully");
                    match_result(&result)?;
                } else {
                    // Show simplified bot info
                    println!("✅ Bot is configured and accessible");
                    if let Ok(json_str) = serde_json::to_string_pretty(&result) {
                        println!("{}", json_str.green());
                    }
                }
            }
            // Interactive setup wizard
            SubCommand::Setup => {
                println!("{}", "🤖 VK Teams Bot CLI Setup Wizard".bold().blue());
                println!("This wizard will help you configure the CLI tool.\n");

                let mut new_config = Config::default();

                // Get API token
                print!("Enter your VK Teams Bot API token: ");
                use std::io::{self, Write};
                io::stdout().flush().unwrap();
                let mut token = String::new();
                io::stdin().read_line(&mut token).unwrap();
                new_config.api.token = Some(token.trim().to_string());

                // Get API URL
                print!("Enter your VK Teams Bot API URL: ");
                io::stdout().flush().unwrap();
                let mut url = String::new();
                io::stdin().read_line(&mut url).unwrap();
                new_config.api.url = Some(url.trim().to_string());

                // Ask about proxy
                print!("Do you need to configure a proxy? (y/N): ");
                io::stdout().flush().unwrap();
                let mut proxy_choice = String::new();
                io::stdin().read_line(&mut proxy_choice).unwrap();
                if proxy_choice.trim().to_lowercase() == "y" {
                    print!("Enter proxy URL: ");
                    io::stdout().flush().unwrap();
                    let mut proxy_url = String::new();
                    io::stdin().read_line(&mut proxy_url).unwrap();
                    new_config.proxy = Some(crate::config::ProxyConfig {
                        url: proxy_url.trim().to_string(),
                        user: None,
                        password: None,
                    });
                }

                // Test the configuration
                println!("\n🧪 Testing configuration...");
                self.config = new_config.clone();
                match self.ensure_bot() {
                    Ok(_) => {
                        println!("✅ Configuration test successful!");
                        
                        // Save configuration
                        if let Err(e) = new_config.save(None) {
                            eprintln!("⚠️  Warning: Could not save configuration: {}", e);
                        } else {
                            println!("💾 Configuration saved successfully!");
                        }
                        
                        println!("\n🎉 Setup complete! You can now use the CLI tool.");
                        println!("Try: {} to test your setup", "vkteams-bot-cli get-self".green());
                    }
                    Err(e) => {
                        eprintln!("❌ Configuration test failed: {}", e);
                        eprintln!("Please check your token and URL and try again.");
                        return Err(e);
                    }
                }
            }
            // Show usage examples
            SubCommand::Examples => {
                println!("{}", "📚 VK Teams Bot CLI Examples".bold().blue());
                println!();
                
                println!("{}", "Basic Message Operations:".bold().green());
                println!("  {}", "vkteams-bot-cli send-text -u USER_ID -m \"Hello World!\"".cyan());
                println!("  {}", "vkteams-bot-cli send-file -u USER_ID -p /path/to/file.pdf".cyan());
                println!("  {}", "vkteams-bot-cli send-voice -u USER_ID -p /path/to/voice.ogg".cyan());
                println!();
                
                println!("{}", "Chat Management:".bold().green());
                println!("  {}", "vkteams-bot-cli get-chat-info -c CHAT_ID".cyan());
                println!("  {}", "vkteams-bot-cli get-chat-members -c CHAT_ID".cyan());
                println!("  {}", "vkteams-bot-cli set-chat-title -c CHAT_ID -t \"New Title\"".cyan());
                println!("  {}", "vkteams-bot-cli set-chat-about -c CHAT_ID -a \"Chat description\"".cyan());
                println!();
                
                println!("{}", "Message Operations:".bold().green());
                println!("  {}", "vkteams-bot-cli edit-message -c CHAT_ID -m MSG_ID -t \"Updated text\"".cyan());
                println!("  {}", "vkteams-bot-cli delete-message -c CHAT_ID -m MSG_ID".cyan());
                println!("  {}", "vkteams-bot-cli pin-message -c CHAT_ID -m MSG_ID".cyan());
                println!("  {}", "vkteams-bot-cli unpin-message -c CHAT_ID -m MSG_ID".cyan());
                println!();
                
                println!("{}", "File Operations:".bold().green());
                println!("  {}", "vkteams-bot-cli get-file -f FILE_ID -p /download/path/".cyan());
                println!();
                
                println!("{}", "Bot Information:".bold().green());
                println!("  {}", "vkteams-bot-cli get-self".cyan());
                println!("  {}", "vkteams-bot-cli get-self --detailed".cyan());
                println!("  {}", "vkteams-bot-cli get-profile -u USER_ID".cyan());
                println!();
                
                println!("{}", "Event Monitoring:".bold().green());
                println!("  {}", "vkteams-bot-cli get-events".cyan());
                println!("  {}", "vkteams-bot-cli get-events -l true | jq '.events[]'".cyan());
                println!();
                
                println!("{}", "Configuration:".bold().green());
                println!("  {}", "vkteams-bot-cli setup".cyan());
                println!("  {}", "vkteams-bot-cli config --show".cyan());
                println!("  {}", "vkteams-bot-cli config --wizard".cyan());
                println!("  {}", "vkteams-bot-cli validate".cyan());
                println!();
                
                println!("{}", "Chat Actions:".bold().green());
                println!("  {}", "vkteams-bot-cli send-action -c CHAT_ID -a typing".cyan());
                println!("  {}", "vkteams-bot-cli send-action -c CHAT_ID -a looking".cyan());
                println!();
                
                println!("{}", "Scheduled Messages:".bold().green());
                println!("  {}", "vkteams-bot-cli schedule text -u CHAT_ID -m \"Hello\" -t \"2024-01-01 10:00\"".cyan());
                println!("  {}", "vkteams-bot-cli schedule text -u CHAT_ID -m \"Daily reminder\" -c \"0 9 * * *\"".cyan());
                println!("  {}", "vkteams-bot-cli schedule text -u CHAT_ID -m \"Every 5 min\" -i 300".cyan());
                println!("  {}", "vkteams-bot-cli schedule file -u CHAT_ID -p \"/path/to/report.pdf\" -t \"30m\"".cyan());
                println!();
                
                println!("{}", "Scheduler Management:".bold().green());
                println!("  {}", "vkteams-bot-cli scheduler start".cyan());
                println!("  {}", "vkteams-bot-cli scheduler status".cyan());
                println!("  {}", "vkteams-bot-cli scheduler list".cyan());
                println!("  {}", "vkteams-bot-cli task show TASK_ID".cyan());
                println!("  {}", "vkteams-bot-cli task run TASK_ID".cyan());
                println!();
            }
            // List all commands with descriptions
            SubCommand::ListCommands => {
                println!("{}", "🤖 VK Teams Bot CLI Commands Reference".bold().blue());
                println!();
                
                let commands = vec![
                    ("send-text", "Send a text message to a user or chat", "Basic messaging"),
                    ("send-file", "Upload and send a file to a user or chat", "File sharing"),
                    ("send-voice", "Send a voice message from an audio file", "Voice messaging"),
                    ("get-file", "Download a file by its ID to local storage", "File management"),
                    ("get-events", "Retrieve bot events or start long polling", "Event monitoring"),
                    ("get-chat-info", "Get detailed information about a chat", "Chat information"),
                    ("get-profile", "Get user profile information", "User information"),
                    ("edit-message", "Edit an existing message in a chat", "Message management"),
                    ("delete-message", "Delete a message from a chat", "Message management"),
                    ("pin-message", "Pin a message in a chat", "Message management"),
                    ("unpin-message", "Unpin a message from a chat", "Message management"),
                    ("get-chat-members", "List all members of a chat", "Chat management"),
                    ("set-chat-title", "Change the title of a chat", "Chat management"),
                    ("set-chat-about", "Set the description of a chat", "Chat management"),
                    ("send-action", "Send typing or looking action to a chat", "Chat interaction"),
                    ("get-self", "Get bot information and verify connectivity", "Bot management"),
                    ("schedule", "Schedule messages to be sent at specific times", "Scheduling"),
                    ("scheduler", "Manage the scheduler daemon service", "Scheduling"),
                    ("task", "Manage individual scheduled tasks", "Scheduling"),
                    ("setup", "Interactive wizard for first-time configuration", "Configuration"),
                    ("examples", "Show usage examples for all commands", "Help"),
                    ("list-commands", "Show this detailed command reference", "Help"),
                    ("validate", "Test configuration and bot connectivity", "Diagnostics"),
                    ("config", "Manage configuration files and settings", "Configuration"),
                ];
                
                let mut categories: std::collections::HashMap<&str, Vec<(&str, &str)>> = std::collections::HashMap::new();
                
                for (cmd, desc, cat) in commands {
                    categories.entry(cat).or_insert_with(Vec::new).push((cmd, desc));
                }
                
                for (category, cmds) in categories {
                    println!("{}", format!("{}:", category).bold().green());
                    for (cmd, desc) in cmds {
                        println!("  {:<20} {}", cmd.cyan(), desc);
                    }
                    println!();
                }
                
                println!("{}", "💡 Tips:".bold().yellow());
                println!("  • Use {} for command-specific help", "vkteams-bot-cli <command> --help".cyan());
                println!("  • Use {} to see usage examples", "vkteams-bot-cli examples".cyan());
                println!("  • Use {} to test your configuration", "vkteams-bot-cli validate".cyan());
                println!("  • Use {} for interactive setup", "vkteams-bot-cli setup".cyan());
            }
            // Validate configuration and test connection
            SubCommand::Validate => {
                println!("{}", "🔍 Validating Configuration...".bold().blue());
                println!();
                
                // Check if configuration exists
                match Config::from_file() {
                    Ok(config) => {
                        println!("✅ Configuration file found and readable");
                        
                        // Check required fields
                        if config.api.token.is_some() {
                            println!("✅ API token is configured");
                        } else {
                            println!("❌ API token is missing");
                        }
                        
                        if config.api.url.is_some() {
                            println!("✅ API URL is configured");
                        } else {
                            println!("❌ API URL is missing");
                        }
                        
                        // Test bot connection
                        println!("\n🔄 Testing bot connection...");
                        match self.ensure_bot() {
                            Ok(bot) => {
                                println!("✅ Bot initialization successful");
                                
                                // Try to get bot info
                                let request = RequestSelfGet::new(());
                                match bot.send_api_request(request).await {
                                    Ok(bot_info) => {
                                        println!("✅ API connection successful");
                                        println!("✅ Bot is working correctly");
                                        
                                        if let Ok(json_str) = serde_json::to_string_pretty(&bot_info) {
                                            println!("\n{}", "Bot Information:".bold().green());
                                            println!("{}", json_str.green());
                                        }
                                    }
                                    Err(e) => {
                                        println!("❌ API connection failed: {}", e);
                                        return Err(CliError::ApiError(e));
                                    }
                                }
                            }
                            Err(e) => {
                                println!("❌ Bot initialization failed: {}", e);
                                return Err(e);
                            }
                        }
                    }
                    Err(_) => {
                        println!("❌ No configuration file found");
                        println!("💡 Run {} to create initial configuration", "vkteams-bot-cli setup".cyan());
                    }
                }
                
                println!("\n{}", "✨ Validation complete!".bold().green());
            }
            // Schedule commands
            SubCommand::Schedule { message_type } => {
                debug!("Scheduling message");
                let mut scheduler = Scheduler::new()?;
                let bot = self.ensure_bot()?;
                scheduler.set_bot(bot.clone());

                let (task_type, schedule, max_runs) = match message_type {
                    ScheduleMessageType::Text { chat_id, message, time, cron, interval, max_runs } => {
                        let task = TaskType::SendText {
                            chat_id: chat_id.clone(),
                            message: message.clone(),
                        };
                        let schedule = self.parse_schedule_options(&time, &cron, &interval)?;
                        (task, schedule, max_runs.clone())
                    },
                    ScheduleMessageType::File { chat_id, file_path, time, cron, interval, max_runs } => {
                        let task = TaskType::SendFile {
                            chat_id: chat_id.clone(),
                            file_path: file_path.clone(),
                        };
                        let schedule = self.parse_schedule_options(&time, &cron, &interval)?;
                        (task, schedule, max_runs.clone())
                    },
                    ScheduleMessageType::Voice { chat_id, file_path, time, cron, interval, max_runs } => {
                        let task = TaskType::SendVoice {
                            chat_id: chat_id.clone(),
                            file_path: file_path.clone(),
                        };
                        let schedule = self.parse_schedule_options(&time, &cron, &interval)?;
                        (task, schedule, max_runs.clone())
                    },
                    ScheduleMessageType::Action { chat_id, action, time, cron, interval, max_runs } => {
                        let task = TaskType::SendAction {
                            chat_id: chat_id.clone(),
                            action: action.clone(),
                        };
                        let schedule = self.parse_schedule_options(&time, &cron, &interval)?;
                        (task, schedule, max_runs.clone())
                    },
                };

                let task_id = scheduler.add_task(task_type, schedule.clone(), max_runs)?;
                
                println!("✅ Task scheduled successfully!");
                println!("📋 Task ID: {}", task_id.green());
                println!("⏰ Schedule: {}", schedule.description().cyan());
                if let Some(max) = max_runs {
                    println!("🔢 Max runs: {}", max);
                }
                println!("\n💡 Use {} to start the scheduler daemon", "vkteams-bot-cli scheduler start".cyan());
                println!("💡 Use {} to list all tasks", "vkteams-bot-cli scheduler list".cyan());
            }
            // Scheduler management commands
            SubCommand::Scheduler { action } => {
                debug!("Scheduler management command");
                let mut scheduler = Scheduler::new()?;

                match action {
                    SchedulerAction::Start => {
                        println!("🚀 Starting scheduler daemon...");
                        let bot = self.ensure_bot()?;
                        scheduler.set_bot(bot.clone());
                        
                        println!("✅ Scheduler is running. Press Ctrl+C to stop.");
                        if let Err(e) = scheduler.run_scheduler().await {
                            eprintln!("❌ Scheduler error: {}", e);
                            return Err(e);
                        }
                    },
                    SchedulerAction::Stop => {
                        println!("⏹️  Scheduler stop command received");
                        println!("💡 Note: Use Ctrl+C to stop a running scheduler daemon");
                    },
                    SchedulerAction::Status => {
                        let tasks = scheduler.list_tasks();
                        let enabled_tasks = tasks.iter().filter(|t| t.enabled).count();
                        let total_tasks = tasks.len();
                        
                        println!("📊 Scheduler Status:");
                        println!("  📋 Total tasks: {}", total_tasks);
                        println!("  ✅ Enabled tasks: {}", enabled_tasks);
                        println!("  ❌ Disabled tasks: {}", total_tasks - enabled_tasks);
                        
                        if total_tasks > 0 {
                            println!("\n⏰ Next upcoming tasks:");
                            let mut upcoming: Vec<_> = tasks.iter()
                                .filter(|t| t.enabled)
                                .collect();
                            upcoming.sort_by_key(|t| t.next_run);
                            
                            for task in upcoming.iter().take(5) {
                                println!("  {} - {} ({})", 
                                    task.next_run.format("%Y-%m-%d %H:%M UTC").to_string().yellow(),
                                    task.task_type.description().cyan(),
                                    task.id[..8].dimmed()
                                );
                            }
                        }
                    },
                    SchedulerAction::List => {
                        let tasks = scheduler.list_tasks();
                        
                        if tasks.is_empty() {
                            println!("📭 No scheduled tasks found");
                            println!("💡 Use {} to schedule a task", "vkteams-bot-cli schedule --help".cyan());
                            return Ok(());
                        }
                        
                        println!("📋 Scheduled Tasks ({}):", tasks.len());
                        println!();
                        
                        for task in tasks {
                            let status = if task.enabled { "✅" } else { "❌" };
                            println!("{} {} ({})", status, task.task_type.description().cyan(), task.id[..8].dimmed());
                            println!("   ⏰ {}", task.schedule.description().yellow());
                            println!("   📊 Runs: {} {}", task.run_count, 
                                if let Some(max) = task.max_runs { format!("/ {}", max) } else { String::new() }
                            );
                            if let Some(last_run) = task.last_run {
                                println!("   🕐 Last run: {}", last_run.format("%Y-%m-%d %H:%M UTC"));
                            }
                            println!("   ⏭️  Next run: {}", task.next_run.format("%Y-%m-%d %H:%M UTC"));
                            println!();
                        }
                    },
                }
            }
            // Task management commands
            SubCommand::Task { action } => {
                debug!("Task management command");
                let mut scheduler = Scheduler::new()?;

                match action {
                    TaskAction::Show { task_id } => {
                        if let Some(task) = scheduler.get_task(&task_id) {
                            println!("📋 Task Details:");
                            println!("  🆔 ID: {}", task.id.green());
                            println!("  📝 Type: {}", task.task_type.description().cyan());
                            println!("  ⏰ Schedule: {}", task.schedule.description().yellow());
                            println!("  ✅ Enabled: {}", if task.enabled { "Yes".green() } else { "No".red() });
                            println!("  📅 Created: {}", task.created_at.format("%Y-%m-%d %H:%M UTC"));
                            if let Some(last_run) = task.last_run {
                                println!("  🕐 Last run: {}", last_run.format("%Y-%m-%d %H:%M UTC"));
                            }
                            println!("  ⏭️  Next run: {}", task.next_run.format("%Y-%m-%d %H:%M UTC"));
                            println!("  📊 Run count: {}", task.run_count);
                            if let Some(max_runs) = task.max_runs {
                                println!("  🔢 Max runs: {}", max_runs);
                            }
                        } else {
                            return Err(CliError::InputError(format!("Task not found: {}", task_id)));
                        }
                    },
                    TaskAction::Remove { task_id } => {
                        scheduler.remove_task(&task_id)?;
                        println!("✅ Task {} removed successfully", task_id.green());
                    },
                    TaskAction::Enable { task_id } => {
                        scheduler.enable_task(&task_id)?;
                        println!("✅ Task {} enabled", task_id.green());
                    },
                    TaskAction::Disable { task_id } => {
                        scheduler.disable_task(&task_id)?;
                        println!("❌ Task {} disabled", task_id.yellow());
                    },
                    TaskAction::Run { task_id } => {
                        println!("🚀 Running task {} immediately...", task_id.green());
                        let bot = self.ensure_bot()?;
                        scheduler.set_bot(bot.clone());
                        scheduler.run_task_once(&task_id).await?;
                        println!("✅ Task executed successfully");
                    },
                }
            }
            SubCommand::Config { show, init, wizard } => {
                if wizard {
                    println!("{}", "⚙️  Configuration Wizard".bold().blue());
                    println!("Current configuration will be updated.\n");

                    let mut new_config = self.config.clone();
                    
                    use std::io::{self, Write};
                    
                    // Update API token
                    if let Some(current_token) = &new_config.api.token {
                        println!("Current API token: {}***", &current_token[..8.min(current_token.len())]);
                    }
                    print!("Enter new API token (or press Enter to keep current): ");
                    io::stdout().flush().unwrap();
                    let mut token = String::new();
                    io::stdin().read_line(&mut token).unwrap();
                    if !token.trim().is_empty() {
                        new_config.api.token = Some(token.trim().to_string());
                    }

                    // Update API URL
                    if let Some(current_url) = &new_config.api.url {
                        println!("Current API URL: {}", current_url);
                    }
                    print!("Enter new API URL (or press Enter to keep current): ");
                    io::stdout().flush().unwrap();
                    let mut url = String::new();
                    io::stdin().read_line(&mut url).unwrap();
                    if !url.trim().is_empty() {
                        new_config.api.url = Some(url.trim().to_string());
                    }

                    // Save and test
                    self.config = new_config.clone();
                    if let Err(e) = new_config.save(None) {
                        eprintln!("⚠️  Warning: Could not save configuration: {}", e);
                    } else {
                        println!("💾 Configuration updated successfully!");
                    }
                }

                if show {
                    // Print current configuration as TOML
                    match toml::to_string_pretty(&self.config) {
                        Ok(config_str) => {
                            println!("Current configuration:\n{}", config_str.green());
                        }
                        Err(e) => {
                            return Err(CliError::UnexpectedError(format!(
                                "Failed to serialize configuration: {e}"
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
                if !show && !init && !wizard {
                    println!(
                        "Use --show to display current configuration, --init to create a new configuration file, or --wizard for interactive configuration."
                    );
                }
            }
        }
        Ok(())
    }

    /// Parse schedule options (time, cron, or interval) into ScheduleType
    fn parse_schedule_options(
        &self,
        time: &Option<String>,
        cron: &Option<String>,
        interval: &Option<u64>,
    ) -> CliResult<ScheduleType> {
        match (time, cron, interval) {
            (Some(time_str), None, None) => {
                let parsed_time = parse_schedule_time(time_str)?;
                Ok(ScheduleType::Once(parsed_time))
            },
            (None, Some(cron_expr), None) => {
                // Validate cron expression
                if cron::Schedule::from_str(cron_expr).is_err() {
                    return Err(CliError::InputError(format!("Invalid cron expression: {}", cron_expr)));
                }
                Ok(ScheduleType::Cron(cron_expr.clone()))
            },
            (None, None, Some(seconds)) => {
                Ok(ScheduleType::Interval {
                    duration_seconds: *seconds,
                    start_time: chrono::Utc::now(),
                })
            },
            (Some(time_str), None, Some(seconds)) => {
                let start_time = parse_schedule_time(time_str)?;
                Ok(ScheduleType::Interval {
                    duration_seconds: *seconds,
                    start_time,
                })
            },
            _ => Err(CliError::InputError(
                "Specify exactly one of: --time, --cron, or --interval. Use --time with --interval to set start time.".to_string()
            )),
        }
    }
}

/// Match result and print it
///
/// # Errors
/// - Returns `BotError::System` if there is an error processing the event
pub async fn match_events<T>(
    bot: Bot,
    result: T,
) -> std::result::Result<(), vkteams_bot::error::BotError>
where
    T: serde::Serialize,
    T: Debug,
{
    debug!("Last event id: {:?}", bot.get_last_event_id().await);
    if let Err(err) = match_result(&result) {
        error!("Error processing event: {}", err);
        return Err(vkteams_bot::error::BotError::System(err.to_string()));
    }
    Ok(())
}
/// Match result and print it
///
/// # Errors
/// - Returns `CliError::UnexpectedError` if there is an error serializing the result
pub fn match_result<T>(result: &T) -> CliResult<()>
where
    T: serde::Serialize,
    T: Debug,
{
    debug!("Result: {:?}", result);
    let json_str = serde_json::to_string(result)
        .map_err(|e| CliError::UnexpectedError(format!("Failed to serialize response: {e}")))?;

    println!("{}", json_str.green());
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
