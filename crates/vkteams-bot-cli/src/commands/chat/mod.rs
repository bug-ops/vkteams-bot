//! Chat management commands module
//!
//! This module contains all commands related to chat management and information.

use crate::commands::{Command, OutputFormat};
use crate::constants::api::actions;
use crate::errors::prelude::{CliError, Result as CliResult};
use crate::utils::output::print_success_result;
use crate::utils::{
    validate_chat_about, validate_chat_action, validate_chat_id, validate_chat_title,
    validate_cursor,
};
use async_trait::async_trait;
use clap::{Subcommand, ValueHint};
use tracing::{debug, info};
use vkteams_bot::prelude::*;

/// All chat management commands
#[derive(Subcommand, Debug, Clone)]
pub enum ChatCommands {
    /// Get chat information
    GetChatInfo {
        #[arg(short = 'c', long, required = true, value_name = "CHAT_ID", value_hint = ValueHint::Username)]
        chat_id: String,
    },
    /// Get user profile information
    GetProfile {
        #[arg(short = 'u', long, required = true, value_name = "USER_ID", value_hint = ValueHint::Username)]
        user_id: String,
    },
    /// Get chat members
    GetChatMembers {
        #[arg(short = 'c', long, required = true, value_name = "CHAT_ID", value_hint = ValueHint::Username)]
        chat_id: String,
        #[arg(long, value_name = "CURSOR")]
        cursor: Option<String>,
    },
    /// Set chat title
    SetChatTitle {
        #[arg(short = 'c', long, required = true, value_name = "CHAT_ID", value_hint = ValueHint::Username)]
        chat_id: String,
        #[arg(short = 't', long, required = true, value_name = "TITLE")]
        title: String,
    },
    /// Set chat description
    SetChatAbout {
        #[arg(short = 'c', long, required = true, value_name = "CHAT_ID", value_hint = ValueHint::Username)]
        chat_id: String,
        #[arg(short = 'a', long, required = true, value_name = "ABOUT")]
        about: String,
    },
    /// Send typing or looking action to chat
    SendAction {
        #[arg(short = 'c', long, required = true, value_name = "CHAT_ID", value_hint = ValueHint::Username)]
        chat_id: String,
        #[arg(short = 'a', long, required = true, value_name = "ACTION")]
        action: String,
    },
}

#[async_trait]
impl Command for ChatCommands {
    async fn execute(&self, bot: &Bot) -> CliResult<()> {
        match self {
            ChatCommands::GetChatInfo { chat_id } => execute_get_chat_info(bot, chat_id).await,
            ChatCommands::GetProfile { user_id } => execute_get_profile(bot, user_id).await,
            ChatCommands::GetChatMembers { chat_id, cursor } => {
                execute_get_chat_members(bot, chat_id, cursor.as_deref()).await
            }
            ChatCommands::SetChatTitle { chat_id, title } => {
                execute_set_chat_title(bot, chat_id, title).await
            }
            ChatCommands::SetChatAbout { chat_id, about } => {
                execute_set_chat_about(bot, chat_id, about).await
            }
            ChatCommands::SendAction { chat_id, action } => {
                execute_send_action(bot, chat_id, action).await
            }
        }
    }

    fn name(&self) -> &'static str {
        match self {
            ChatCommands::GetChatInfo { .. } => "get-chat-info",
            ChatCommands::GetProfile { .. } => "get-profile",
            ChatCommands::GetChatMembers { .. } => "get-chat-members",
            ChatCommands::SetChatTitle { .. } => "set-chat-title",
            ChatCommands::SetChatAbout { .. } => "set-chat-about",
            ChatCommands::SendAction { .. } => "send-action",
        }
    }

    fn validate(&self) -> CliResult<()> {
        match self {
            ChatCommands::GetChatInfo { chat_id }
            | ChatCommands::GetProfile { user_id: chat_id } => {
                validate_chat_id(chat_id)?;
            }
            ChatCommands::GetChatMembers { chat_id, cursor } => {
                validate_chat_id(chat_id)?;
                if let Some(cursor_val) = cursor {
                    validate_cursor(cursor_val)?;
                }
            }
            ChatCommands::SetChatTitle { chat_id, title } => {
                validate_chat_id(chat_id)?;
                validate_chat_title(title)?;
            }
            ChatCommands::SetChatAbout { chat_id, about } => {
                validate_chat_id(chat_id)?;
                validate_chat_about(about)?;
            }
            ChatCommands::SendAction { chat_id, action } => {
                validate_chat_id(chat_id)?;
                validate_chat_action(action)?;
            }
        }
        Ok(())
    }
}

// Command execution functions

async fn execute_get_chat_info(bot: &Bot, chat_id: &str) -> CliResult<()> {
    debug!("Getting chat info for {}", chat_id);

    let request = RequestChatsGetInfo::new(ChatId(chat_id.to_string()));
    let result = bot
        .send_api_request(request)
        .await
        .map_err(CliError::ApiError)?;

    info!("Successfully retrieved chat info for {}", chat_id);
    print_success_result(&result, &OutputFormat::Pretty)?;
    Ok(())
}

async fn execute_get_profile(bot: &Bot, user_id: &str) -> CliResult<()> {
    debug!("Getting profile for user {}", user_id);

    let request = RequestChatsGetInfo::new(ChatId(user_id.to_string()));
    let result = bot
        .send_api_request(request)
        .await
        .map_err(CliError::ApiError)?;

    info!("Successfully retrieved profile for user {}", user_id);
    print_success_result(&result, &OutputFormat::Pretty)?;
    Ok(())
}

async fn execute_get_chat_members(bot: &Bot, chat_id: &str, cursor: Option<&str>) -> CliResult<()> {
    debug!("Getting chat members for {}", chat_id);

    let mut request = RequestChatsGetMembers::new(ChatId(chat_id.to_string()));
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

    let result = bot
        .send_api_request(request)
        .await
        .map_err(CliError::ApiError)?;

    info!("Successfully retrieved members for chat {}", chat_id);
    print_success_result(&result, &OutputFormat::Pretty)?;
    Ok(())
}

async fn execute_set_chat_title(bot: &Bot, chat_id: &str, title: &str) -> CliResult<()> {
    debug!("Setting chat title for {} to {}", chat_id, title);

    let request = RequestChatsSetTitle::new((ChatId(chat_id.to_string()), title.to_string()));

    let result = bot
        .send_api_request(request)
        .await
        .map_err(CliError::ApiError)?;

    info!("Successfully set title for chat {}: {}", chat_id, title);
    print_success_result(&result, &OutputFormat::Pretty)?;
    Ok(())
}

async fn execute_set_chat_about(bot: &Bot, chat_id: &str, about: &str) -> CliResult<()> {
    debug!("Setting chat description for {} to {}", chat_id, about);

    let request = RequestChatsSetAbout::new((ChatId(chat_id.to_string()), about.to_string()));

    let result = bot
        .send_api_request(request)
        .await
        .map_err(CliError::ApiError)?;

    info!(
        "Successfully set description for chat {}: {}",
        chat_id, about
    );
    print_success_result(&result, &OutputFormat::Pretty)?;
    Ok(())
}

async fn execute_send_action(bot: &Bot, chat_id: &str, action: &str) -> CliResult<()> {
    debug!("Sending {} action to chat {}", action, chat_id);

    let chat_action = match action {
        actions::TYPING => ChatActions::Typing,
        actions::LOOKING => ChatActions::Looking,
        _ => {
            return Err(CliError::InputError(format!(
                "Unknown action: {}. Available actions: {}, {}",
                action,
                actions::TYPING,
                actions::LOOKING
            )));
        }
    };

    let request = RequestChatsSendAction::new((ChatId(chat_id.to_string()), chat_action));

    let result = bot
        .send_api_request(request)
        .await
        .map_err(CliError::ApiError)?;

    info!("Successfully sent {} action to chat {}", action, chat_id);
    print_success_result(&result, &OutputFormat::Pretty)?;
    Ok(())
}

// Validation functions are now imported from utils/validation module

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_parse_chat_id_valid() {
        let input = "12345@chat";
        let res = validate_chat_id(input);
        assert!(res.is_ok());
    }

    #[test]
    fn test_parse_chat_id_invalid() {
        let input = "user with spaces";
        let res = validate_chat_id(input);
        assert!(res.is_err());
    }

    #[test]
    fn test_handle_empty_input() {
        let input = "";
        let res = validate_chat_id(input);
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_cursor_valid() {
        let input = "123";
        let res = validate_cursor(input);
        assert!(res.is_ok());
    }

    #[test]
    fn test_parse_cursor_empty() {
        let input = "";
        let res = validate_cursor(input);
        assert!(res.is_err());
    }

    #[test]
    fn test_validate_set_chat_title_invalid() {
        let cmd = ChatCommands::SetChatTitle {
            chat_id: "12345@chat".to_string(),
            title: "".to_string(),
        };
        let res = cmd.validate();
        assert!(res.is_err());
    }

    #[test]
    fn test_validate_set_chat_about_invalid() {
        let cmd = ChatCommands::SetChatAbout {
            chat_id: "12345@chat".to_string(),
            about: "".to_string(),
        };
        let res = cmd.validate();
        assert!(res.is_err());
    }

    #[test]
    fn test_validate_send_action_invalid() {
        let cmd = ChatCommands::SendAction {
            chat_id: "12345@chat".to_string(),
            action: "invalid_action".to_string(),
        };
        let res = cmd.validate();
        assert!(res.is_err());
    }

    #[test]
    fn test_validate_get_profile_invalid() {
        let cmd = ChatCommands::GetProfile {
            user_id: "".to_string(),
        };
        let res = cmd.validate();
        assert!(res.is_err());
    }

    #[test]
    fn test_validate_get_chat_members_invalid_cursor() {
        let cmd = ChatCommands::GetChatMembers {
            chat_id: "12345@chat".to_string(),
            cursor: Some("not_a_number".to_string()),
        };
        let res = cmd.validate();
        assert!(res.is_err()); // cursor должен быть числом
    }

    #[test]
    fn test_validate_get_chat_members_empty_cursor() {
        let cmd = ChatCommands::GetChatMembers {
            chat_id: "12345@chat".to_string(),
            cursor: Some("".to_string()),
        };
        let res = cmd.validate();
        assert!(res.is_err());
    }

    // Property-based тесты для title/about/action (короткие и длинные строки)
    #[test]
    fn test_validate_set_chat_title_long() {
        let cmd = ChatCommands::SetChatTitle {
            chat_id: "12345@chat".to_string(),
            title: "a".repeat(300),
        };
        let res = cmd.validate();
        assert!(res.is_err());
    }

    #[test]
    fn test_validate_set_chat_about_long() {
        let cmd = ChatCommands::SetChatAbout {
            chat_id: "12345@chat".to_string(),
            about: "a".repeat(300),
        };
        let res = cmd.validate();
        assert!(res.is_ok()); // валидация не ограничивает длину
    }

    fn dummy_bot() -> Bot {
        Bot::with_params(&APIVersionUrl::V1, "dummy_token", "https://dummy.api.com").unwrap()
    }

    #[test]
    fn test_execute_get_chat_info_api_error() {
        let cmd = ChatCommands::GetChatInfo {
            chat_id: "12345@chat".to_string(),
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }

    #[test]
    fn test_execute_get_profile_api_error() {
        let cmd = ChatCommands::GetProfile {
            user_id: "user123".to_string(),
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }

    #[test]
    fn test_execute_get_chat_members_api_error() {
        let cmd = ChatCommands::GetChatMembers {
            chat_id: "12345@chat".to_string(),
            cursor: None,
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }

    #[test]
    fn test_execute_set_chat_title_api_error() {
        let cmd = ChatCommands::SetChatTitle {
            chat_id: "12345@chat".to_string(),
            title: "New Title".to_string(),
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }

    #[test]
    fn test_execute_set_chat_about_api_error() {
        let cmd = ChatCommands::SetChatAbout {
            chat_id: "12345@chat".to_string(),
            about: "About text".to_string(),
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }

    #[test]
    fn test_execute_send_action_invalid_action() {
        let cmd = ChatCommands::SendAction {
            chat_id: "12345@chat".to_string(),
            action: "invalid".to_string(),
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
        let err = res.unwrap_err();
        match err {
            CliError::InputError(msg) => assert!(msg.contains("Unknown action")),
            _ => panic!("Expected InputError for unknown action"),
        }
    }
}
