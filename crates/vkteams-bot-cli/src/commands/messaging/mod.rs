//! Messaging commands module
//!
//! This module contains all commands related to sending and managing messages.

use crate::commands::{Command, OutputFormat};
use crate::constants::ui::emoji;
use crate::errors::prelude::{CliError, Result as CliResult};
use crate::file_utils;
use crate::output::{CliResponse, OutputFormatter};
use crate::utils::output::print_success_result;
use crate::utils::{
    validate_chat_id, validate_file_path, validate_message_id, validate_message_text,
    validate_voice_file_path,
};

use async_trait::async_trait;
use clap::{Subcommand, ValueHint};
use colored::Colorize;
use serde_json::json;
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

    /// New method for structured output support
    async fn execute_with_output(&self, bot: &Bot, output_format: &OutputFormat) -> CliResult<()> {
        let response = match self {
            MessagingCommands::SendText { chat_id, message } => {
                execute_send_text_structured(bot, chat_id, message).await
            }
            MessagingCommands::SendFile { chat_id, file_path } => {
                execute_send_file_structured(bot, chat_id, file_path).await
            }
            MessagingCommands::SendVoice { chat_id, file_path } => {
                execute_send_voice_structured(bot, chat_id, file_path).await
            }
            MessagingCommands::EditMessage {
                chat_id,
                message_id,
                new_text,
            } => execute_edit_message_structured(bot, chat_id, message_id, new_text).await,
            MessagingCommands::DeleteMessage {
                chat_id,
                message_id,
            } => execute_delete_message_structured(bot, chat_id, message_id).await,
            MessagingCommands::PinMessage {
                chat_id,
                message_id,
            } => execute_pin_message_structured(bot, chat_id, message_id).await,
            MessagingCommands::UnpinMessage {
                chat_id,
                message_id,
            } => execute_unpin_message_structured(bot, chat_id, message_id).await,
        };

        OutputFormatter::print(&response, output_format)?;

        if !response.success {
            return Err(CliError::UnexpectedError("Command failed".to_string()));
        }

        Ok(())
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

// Structured output versions
async fn execute_send_text_structured(
    bot: &Bot,
    chat_id: &str,
    message: &str,
) -> CliResponse<serde_json::Value> {
    debug!("Sending text message to {}", chat_id);

    let parser = MessageTextParser::new().add(MessageTextFormat::Plain(message.to_string()));
    let request =
        match RequestMessagesSendText::new(ChatId::from_borrowed_str(chat_id)).set_text(parser) {
            Ok(req) => req,
            Err(e) => {
                return CliResponse::error("send-text", format!("Failed to create message: {e}"));
            }
        };

    match bot.send_api_request(request).await {
        Ok(result) => {
            info!("Successfully sent text message to {}", chat_id);
            let data = json!({
                "chat_id": chat_id,
                "message": message,
                "message_id": result.msg_id
            });
            CliResponse::success("send-text", data)
        }
        Err(e) => CliResponse::error("send-text", format!("Failed to send message: {e}")),
    }
}

async fn execute_send_file_structured(
    bot: &Bot,
    chat_id: &str,
    file_path: &str,
) -> CliResponse<serde_json::Value> {
    debug!("Sending file {} to {}", file_path, chat_id);

    match file_utils::upload_file(bot, chat_id, file_path).await {
        Ok(file_id) => {
            info!("Successfully sent file to {}", chat_id);
            let data = json!({
                "chat_id": chat_id,
                "file_path": file_path,
                "file_id": file_id
            });
            CliResponse::success("send-file", data)
        }
        Err(e) => CliResponse::error("send-file", format!("Failed to send file: {e}")),
    }
}

async fn execute_send_voice_structured(
    bot: &Bot,
    chat_id: &str,
    file_path: &str,
) -> CliResponse<serde_json::Value> {
    debug!("Sending voice message {} to {}", file_path, chat_id);

    match file_utils::upload_voice(bot, chat_id, file_path).await {
        Ok(file_id) => {
            info!("Successfully sent voice message to {}", chat_id);
            let data = json!({
                "chat_id": chat_id,
                "file_path": file_path,
                "file_id": file_id
            });
            CliResponse::success("send-voice", data)
        }
        Err(e) => CliResponse::error("send-voice", format!("Failed to send voice: {e}")),
    }
}

async fn execute_edit_message_structured(
    bot: &Bot,
    chat_id: &str,
    message_id: &str,
    new_text: &str,
) -> CliResponse<serde_json::Value> {
    debug!("Editing message {} in {}", message_id, chat_id);

    let parser = MessageTextParser::new().add(MessageTextFormat::Plain(new_text.to_string()));
    let request = match RequestMessagesEditText::new((
        ChatId::from_borrowed_str(chat_id),
        MsgId(message_id.to_string()),
    ))
    .set_text(parser)
    {
        Ok(req) => req,
        Err(e) => {
            return CliResponse::error("edit-message", format!("Failed to set message text: {e}"));
        }
    };

    match bot.send_api_request(request).await {
        Ok(_result) => {
            info!("Successfully edited message {} in {}", message_id, chat_id);
            let data = json!({
                "chat_id": chat_id,
                "message_id": message_id,
                "new_text": new_text
            });
            CliResponse::success("edit-message", data)
        }
        Err(e) => CliResponse::error("edit-message", format!("Failed to edit message: {e}")),
    }
}

async fn execute_delete_message_structured(
    bot: &Bot,
    chat_id: &str,
    message_id: &str,
) -> CliResponse<serde_json::Value> {
    debug!("Deleting message {} from {}", message_id, chat_id);

    let request = RequestMessagesDeleteMessages::new((
        ChatId::from_borrowed_str(chat_id),
        MsgId(message_id.to_string()),
    ));

    match bot.send_api_request(request).await {
        Ok(_result) => {
            info!(
                "Successfully deleted message {} from {}",
                message_id, chat_id
            );
            let data = json!({
                "chat_id": chat_id,
                "message_id": message_id,
                "action": "deleted"
            });
            CliResponse::success("delete-message", data)
        }
        Err(e) => CliResponse::error("delete-message", format!("Failed to delete message: {e}")),
    }
}

async fn execute_pin_message_structured(
    bot: &Bot,
    chat_id: &str,
    message_id: &str,
) -> CliResponse<serde_json::Value> {
    debug!("Pinning message {} in {}", message_id, chat_id);

    let request = RequestChatsPinMessage::new((
        ChatId::from_borrowed_str(chat_id),
        MsgId(message_id.to_string()),
    ));

    match bot.send_api_request(request).await {
        Ok(_result) => {
            info!("Successfully pinned message {} in {}", message_id, chat_id);
            let data = json!({
                "chat_id": chat_id,
                "message_id": message_id,
                "action": "pinned"
            });
            CliResponse::success("pin-message", data)
        }
        Err(e) => CliResponse::error("pin-message", format!("Failed to pin message: {e}")),
    }
}

async fn execute_unpin_message_structured(
    bot: &Bot,
    chat_id: &str,
    message_id: &str,
) -> CliResponse<serde_json::Value> {
    debug!("Unpinning message {} from {}", message_id, chat_id);

    let request = RequestChatsUnpinMessage::new((
        ChatId::from_borrowed_str(chat_id),
        MsgId(message_id.to_string()),
    ));

    match bot.send_api_request(request).await {
        Ok(_result) => {
            info!(
                "Successfully unpinned message {} from {}",
                message_id, chat_id
            );
            let data = json!({
                "chat_id": chat_id,
                "message_id": message_id,
                "action": "unpinned"
            });
            CliResponse::success("unpin-message", data)
        }
        Err(e) => CliResponse::error("unpin-message", format!("Failed to unpin message: {e}")),
    }
}

// Legacy output versions (for backward compatibility)
async fn execute_send_text(bot: &Bot, chat_id: &str, message: &str) -> CliResult<()> {
    debug!("Sending text message to {}", chat_id);

    let parser = MessageTextParser::new().add(MessageTextFormat::Plain(message.to_string()));
    let request = RequestMessagesSendText::new(ChatId::from_borrowed_str(chat_id))
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

    file_utils::upload_file(bot, chat_id, file_path).await?;

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

    file_utils::upload_voice(bot, chat_id, file_path).await?;

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
    let request = RequestMessagesEditText::new((
        ChatId::from_borrowed_str(chat_id),
        MsgId(message_id.to_string()),
    ))
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
        ChatId::from_borrowed_str(chat_id),
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

    let request = RequestChatsPinMessage::new((
        ChatId::from_borrowed_str(chat_id),
        MsgId(message_id.to_string()),
    ));

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

    let request = RequestChatsUnpinMessage::new((
        ChatId::from_borrowed_str(chat_id),
        MsgId(message_id.to_string()),
    ));

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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_send_text_valid() {
        let cmd = MessagingCommands::SendText {
            chat_id: "user123".to_string(),
            message: "Hello".to_string(),
        };
        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_send_text_invalid_chat_id() {
        let cmd = MessagingCommands::SendText {
            chat_id: "user with spaces".to_string(),
            message: "Hello".to_string(),
        };
        assert!(cmd.validate().is_err());
    }

    #[test]
    fn test_send_text_empty_message() {
        let cmd = MessagingCommands::SendText {
            chat_id: "user123".to_string(),
            message: "".to_string(),
        };
        assert!(cmd.validate().is_err());
    }

    #[test]
    fn test_send_file_invalid_path() {
        let cmd = MessagingCommands::SendFile {
            chat_id: "user123".to_string(),
            file_path: "nonexistent.file".to_string(),
        };
        // Путь не существует, validate_file_path вернет ошибку
        assert!(cmd.validate().is_err());
    }

    #[test]
    fn test_send_voice_invalid_path() {
        let cmd = MessagingCommands::SendVoice {
            chat_id: "user123".to_string(),
            file_path: "nonexistent.ogg".to_string(),
        };
        assert!(cmd.validate().is_err());
    }

    #[test]
    fn test_edit_message_invalid_message_id() {
        let cmd = MessagingCommands::EditMessage {
            chat_id: "user123".to_string(),
            message_id: "id with space".to_string(),
            new_text: "new text".to_string(),
        };
        assert!(cmd.validate().is_err());
    }

    #[test]
    fn test_delete_message_empty_message_id() {
        let cmd = MessagingCommands::DeleteMessage {
            chat_id: "user123".to_string(),
            message_id: "".to_string(),
        };
        assert!(cmd.validate().is_err());
    }

    #[test]
    fn test_pin_message_valid() {
        let cmd = MessagingCommands::PinMessage {
            chat_id: "user123".to_string(),
            message_id: "msg123".to_string(),
        };
        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_unpin_message_invalid_chat_id() {
        let cmd = MessagingCommands::UnpinMessage {
            chat_id: "invalid id".to_string(),
            message_id: "msg123".to_string(),
        };
        assert!(cmd.validate().is_err());
    }

    fn dummy_bot() -> Bot {
        Bot::with_params(&APIVersionUrl::V1, "dummy_token", "https://dummy.api.com").unwrap()
    }

    #[test]
    fn test_execute_send_text_api_error() {
        let cmd = MessagingCommands::SendText {
            chat_id: "12345@chat".to_string(),
            message: "hello".to_string(),
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }

    #[test]
    fn test_execute_send_file_api_error() {
        let cmd = MessagingCommands::SendFile {
            chat_id: "12345@chat".to_string(),
            file_path: "/tmp/file.txt".to_string(),
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }

    #[test]
    fn test_execute_send_voice_api_error() {
        let cmd = MessagingCommands::SendVoice {
            chat_id: "12345@chat".to_string(),
            file_path: "/tmp/voice.ogg".to_string(),
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }

    #[test]
    fn test_execute_edit_message_api_error() {
        let cmd = MessagingCommands::EditMessage {
            chat_id: "12345@chat".to_string(),
            message_id: "msgid".to_string(),
            new_text: "new text".to_string(),
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }

    #[test]
    fn test_execute_delete_message_api_error() {
        let cmd = MessagingCommands::DeleteMessage {
            chat_id: "12345@chat".to_string(),
            message_id: "msgid".to_string(),
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }

    #[test]
    fn test_execute_pin_message_api_error() {
        let cmd = MessagingCommands::PinMessage {
            chat_id: "12345@chat".to_string(),
            message_id: "msgid".to_string(),
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }

    #[test]
    fn test_execute_unpin_message_api_error() {
        let cmd = MessagingCommands::UnpinMessage {
            chat_id: "12345@chat".to_string(),
            message_id: "msgid".to_string(),
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }
}
