//! Messaging commands module
//!
//! This module contains all commands related to sending and managing messages.

use crate::commands::{Command, OutputFormat};
use crate::config::Config;
use crate::constants::ui::emoji;
use crate::errors::prelude::{CliError, Result as CliResult};
use crate::file_utils;
use crate::utils::output::print_success_result;
use crate::utils::{
    validate_chat_id, validate_file_path, validate_message_id, validate_message_text,
    validate_voice_file_path,
};

use async_trait::async_trait;
use clap::{Subcommand, ValueHint};
use colored::Colorize;
use tracing::{debug, info};
use vkteams_bot::prelude::*;

/// All messaging-related commands
#[derive(Subcommand, Debug, Clone)]
pub enum MessagingCommands {
    /// Send text message to user or chat
    SendText {
        #[arg(short = 'u', long, required = true, value_name = "CHAT_ID", value_hint = ValueHint::Username)]
        chat_id: String,
        #[arg(short = 'm', long, required = true, value_name = "MESSAGE")]
        message: String,
    },
    /// Send file to user or chat
    SendFile {
        #[arg(short = 'u', long, required = true, value_name = "CHAT_ID", value_hint = ValueHint::Username)]
        chat_id: String,
        #[arg(short = 'p', long, required = true, value_name = "FILE_PATH", value_hint = ValueHint::FilePath)]
        file_path: String,
    },
    /// Send voice message to user or chat
    SendVoice {
        #[arg(short = 'u', long, required = true, value_name = "CHAT_ID", value_hint = ValueHint::Username)]
        chat_id: String,
        #[arg(short = 'p', long, required = true, value_name = "FILE_PATH", value_hint = ValueHint::FilePath)]
        file_path: String,
    },
    /// Edit existing message
    EditMessage {
        #[arg(short = 'c', long, required = true, value_name = "CHAT_ID", value_hint = ValueHint::Username)]
        chat_id: String,
        #[arg(short = 'm', long, required = true, value_name = "MESSAGE_ID")]
        message_id: String,
        #[arg(short = 't', long, required = true, value_name = "NEW_TEXT")]
        new_text: String,
    },
    /// Delete message from chat
    DeleteMessage {
        #[arg(short = 'c', long, required = true, value_name = "CHAT_ID", value_hint = ValueHint::Username)]
        chat_id: String,
        #[arg(short = 'm', long, required = true, value_name = "MESSAGE_ID")]
        message_id: String,
    },
    /// Pin message in chat
    PinMessage {
        #[arg(short = 'c', long, required = true, value_name = "CHAT_ID", value_hint = ValueHint::Username)]
        chat_id: String,
        #[arg(short = 'm', long, required = true, value_name = "MESSAGE_ID")]
        message_id: String,
    },
    /// Unpin message from chat
    UnpinMessage {
        #[arg(short = 'c', long, required = true, value_name = "CHAT_ID", value_hint = ValueHint::Username)]
        chat_id: String,
        #[arg(short = 'm', long, required = true, value_name = "MESSAGE_ID")]
        message_id: String,
    },
}

#[async_trait]
impl Command for MessagingCommands {
    async fn execute(&self, bot: &Bot) -> CliResult<()> {
        match self {
            MessagingCommands::SendText { chat_id, message } => {
                execute_send_text(bot, chat_id, message).await
            }
            MessagingCommands::SendFile { chat_id, file_path } => {
                execute_send_file(bot, chat_id, file_path).await
            }
            MessagingCommands::SendVoice { chat_id, file_path } => {
                execute_send_voice(bot, chat_id, file_path).await
            }
            MessagingCommands::EditMessage {
                chat_id,
                message_id,
                new_text,
            } => execute_edit_message(bot, chat_id, message_id, new_text).await,
            MessagingCommands::DeleteMessage {
                chat_id,
                message_id,
            } => execute_delete_message(bot, chat_id, message_id).await,
            MessagingCommands::PinMessage {
                chat_id,
                message_id,
            } => execute_pin_message(bot, chat_id, message_id).await,
            MessagingCommands::UnpinMessage {
                chat_id,
                message_id,
            } => execute_unpin_message(bot, chat_id, message_id).await,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            MessagingCommands::SendText { .. } => "send-text",
            MessagingCommands::SendFile { .. } => "send-file",
            MessagingCommands::SendVoice { .. } => "send-voice",
            MessagingCommands::EditMessage { .. } => "edit-message",
            MessagingCommands::DeleteMessage { .. } => "delete-message",
            MessagingCommands::PinMessage { .. } => "pin-message",
            MessagingCommands::UnpinMessage { .. } => "unpin-message",
        }
    }

    fn validate(&self) -> CliResult<()> {
        match self {
            MessagingCommands::SendText { chat_id, message } => {
                validate_chat_id(chat_id)?;
                validate_message_text(message)?;
            }
            MessagingCommands::SendFile { chat_id, file_path } => {
                validate_chat_id(chat_id)?;
                validate_file_path(file_path)?;
            }
            MessagingCommands::SendVoice { chat_id, file_path } => {
                validate_chat_id(chat_id)?;
                validate_voice_file_path(file_path)?;
            }
            MessagingCommands::EditMessage {
                chat_id,
                message_id,
                new_text,
            } => {
                validate_chat_id(chat_id)?;
                validate_message_id(message_id)?;
                validate_message_text(new_text)?;
            }
            MessagingCommands::DeleteMessage {
                chat_id,
                message_id,
            }
            | MessagingCommands::PinMessage {
                chat_id,
                message_id,
            }
            | MessagingCommands::UnpinMessage {
                chat_id,
                message_id,
            } => {
                validate_chat_id(chat_id)?;
                validate_message_id(message_id)?;
            }
        }
        Ok(())
    }
}

// Command execution functions

async fn execute_send_text(bot: &Bot, chat_id: &str, message: &str) -> CliResult<()> {
    debug!("Sending text message to {}", chat_id);

    let parser = MessageTextParser::new().add(MessageTextFormat::Plain(message.to_string()));
    let request = RequestMessagesSendText::new(ChatId(chat_id.to_string()))
        .set_text(parser)
        .map_err(|e| CliError::InputError(format!("Failed to create message: {e}")))?;

    let result = bot
        .send_api_request(request)
        .await
        .map_err(CliError::ApiError)?;

    info!("Successfully sent text message to {}", chat_id);
    print_success_result(&result, &OutputFormat::Pretty)?;
    Ok(())
}

async fn execute_send_file(bot: &Bot, chat_id: &str, file_path: &str) -> CliResult<()> {
    debug!("Sending file {} to {}", file_path, chat_id);

    let config = toml::from_str::<Config>("").unwrap();
    file_utils::upload_file(bot, chat_id, file_path, &config).await?;

    info!("Successfully sent file to {}", chat_id);
    println!(
        "{} File sent successfully to {}",
        emoji::CHECK,
        chat_id.green()
    );
    Ok(())
}

async fn execute_send_voice(bot: &Bot, chat_id: &str, file_path: &str) -> CliResult<()> {
    debug!("Sending voice message {} to {}", file_path, chat_id);

    let config = toml::from_str::<Config>("").unwrap();
    file_utils::upload_voice(bot, chat_id, file_path, &config).await?;

    info!("Successfully sent voice message to {}", chat_id);
    println!(
        "{} Voice message sent successfully to {}",
        emoji::CHECK,
        chat_id.green()
    );
    Ok(())
}

async fn execute_edit_message(
    bot: &Bot,
    chat_id: &str,
    message_id: &str,
    new_text: &str,
) -> CliResult<()> {
    debug!("Editing message {} in {}", message_id, chat_id);

    let parser = MessageTextParser::new().add(MessageTextFormat::Plain(new_text.to_string()));
    let request =
        RequestMessagesEditText::new((ChatId(chat_id.to_string()), MsgId(message_id.to_string())))
            .set_text(parser)
            .map_err(|e| CliError::InputError(format!("Failed to set message text: {e}")))?;

    let result = bot
        .send_api_request(request)
        .await
        .map_err(CliError::ApiError)?;

    info!("Successfully edited message {} in {}", message_id, chat_id);
    print_success_result(&result, &OutputFormat::Pretty)?;
    Ok(())
}

async fn execute_delete_message(bot: &Bot, chat_id: &str, message_id: &str) -> CliResult<()> {
    debug!("Deleting message {} from {}", message_id, chat_id);

    let request = RequestMessagesDeleteMessages::new((
        ChatId(chat_id.to_string()),
        MsgId(message_id.to_string()),
    ));

    let result = bot
        .send_api_request(request)
        .await
        .map_err(CliError::ApiError)?;

    info!(
        "Successfully deleted message {} from {}",
        message_id, chat_id
    );
    print_success_result(&result, &OutputFormat::Pretty)?;
    Ok(())
}

async fn execute_pin_message(bot: &Bot, chat_id: &str, message_id: &str) -> CliResult<()> {
    debug!("Pinning message {} in {}", message_id, chat_id);

    let request =
        RequestChatsPinMessage::new((ChatId(chat_id.to_string()), MsgId(message_id.to_string())));

    let result = bot
        .send_api_request(request)
        .await
        .map_err(CliError::ApiError)?;

    info!("Successfully pinned message {} in {}", message_id, chat_id);
    print_success_result(&result, &OutputFormat::Pretty)?;
    Ok(())
}

async fn execute_unpin_message(bot: &Bot, chat_id: &str, message_id: &str) -> CliResult<()> {
    debug!("Unpinning message {} from {}", message_id, chat_id);

    let request =
        RequestChatsUnpinMessage::new((ChatId(chat_id.to_string()), MsgId(message_id.to_string())));

    let result = bot
        .send_api_request(request)
        .await
        .map_err(CliError::ApiError)?;

    info!(
        "Successfully unpinned message {} from {}",
        message_id, chat_id
    );
    print_success_result(&result, &OutputFormat::Pretty)?;
    Ok(())
}

// Validation functions are now imported from utils/validation module
