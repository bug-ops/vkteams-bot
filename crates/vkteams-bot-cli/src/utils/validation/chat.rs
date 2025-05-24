//! Chat-related validation functions
//!
//! This module contains validation functions for chat IDs, usernames,
//! and other chat-related identifiers.

use crate::constants::validation;
use crate::errors::prelude::{CliError, Result as CliResult};
use super::{validate_not_empty, validate_length};

/// Validate a chat ID or user ID
///
/// # Arguments
/// * `chat_id` - The chat/user ID to validate
///
/// # Returns
/// * `Ok(())` if the chat ID is valid
/// * `Err(CliError::InputError)` if the chat ID is invalid
///
/// # Validation Rules
/// - Cannot be empty
/// - Must be between 1 and 64 characters
/// - Can contain alphanumeric characters, underscores, dots, hyphens, and @ symbols
pub fn validate_chat_id(chat_id: &str) -> CliResult<()> {
    validate_not_empty(chat_id, "Chat ID")?;
    validate_length(
        chat_id,
        "Chat ID",
        validation::MIN_USERNAME_LENGTH,
        validation::MAX_USERNAME_LENGTH,
    )?;

    // Validate character set
    if !chat_id.chars().all(|c| {
        c.is_alphanumeric() || c == '_' || c == '.' || c == '-' || c == '@'
    }) {
        return Err(CliError::InputError(
            "Chat ID contains invalid characters. Only alphanumeric, underscore, dot, hyphen, and @ are allowed".to_string()
        ));
    }

    Ok(())
}

/// Validate a chat title
///
/// # Arguments
/// * `title` - The chat title to validate
///
/// # Returns
/// * `Ok(())` if the title is valid
/// * `Err(CliError::InputError)` if the title is invalid
pub fn validate_chat_title(title: &str) -> CliResult<()> {
    validate_not_empty(title, "Chat title")?;
    validate_length(
        title,
        "Chat title",
        validation::MIN_CHAT_TITLE_LENGTH,
        validation::MAX_CHAT_TITLE_LENGTH,
    )?;
    Ok(())
}

/// Validate a chat description/about text
///
/// # Arguments
/// * `about` - The chat description to validate
///
/// # Returns
/// * `Ok(())` if the description is valid
/// * `Err(CliError::InputError)` if the description is invalid
pub fn validate_chat_about(about: &str) -> CliResult<()> {
    validate_not_empty(about, "Chat description")?;
    validate_length(
        about,
        "Chat description",
        1,
        validation::MAX_MESSAGE_LENGTH,
    )?;
    Ok(())
}

/// Validate a chat action
///
/// # Arguments
/// * `action` - The action to validate (e.g., "typing", "looking")
///
/// # Returns
/// * `Ok(())` if the action is valid
/// * `Err(CliError::InputError)` if the action is invalid
pub fn validate_chat_action(action: &str) -> CliResult<()> {
    use crate::constants::api::actions;
    
    match action {
        actions::TYPING | actions::LOOKING => Ok(()),
        _ => Err(CliError::InputError(format!(
            "Invalid action: {}. Available actions: {}, {}",
            action, actions::TYPING, actions::LOOKING
        ))),
    }
}

/// Validate a cursor value for pagination
///
/// # Arguments
/// * `cursor` - The cursor string to validate
///
/// # Returns
/// * `Ok(())` if the cursor is valid
/// * `Err(CliError::InputError)` if the cursor is invalid
pub fn validate_cursor(cursor: &str) -> CliResult<()> {
    cursor.parse::<u32>()
        .map_err(|_| CliError::InputError("Cursor must be a valid number".to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_chat_id_valid() {
        assert!(validate_chat_id("user123").is_ok());
        assert!(validate_chat_id("@username").is_ok());
        assert!(validate_chat_id("chat.group-1").is_ok());
        assert!(validate_chat_id("test_user").is_ok());
    }

    #[test]
    fn test_validate_chat_id_invalid() {
        assert!(validate_chat_id("").is_err());
        assert!(validate_chat_id("   ").is_err());
        assert!(validate_chat_id("user with spaces").is_err());
        assert!(validate_chat_id("user#invalid").is_err());
    }

    #[test]
    fn test_validate_chat_title() {
        assert!(validate_chat_title("Valid Title").is_ok());
        assert!(validate_chat_title("").is_err());
        assert!(validate_chat_title("   ").is_err());
    }

    #[test]
    fn test_validate_chat_action() {
        assert!(validate_chat_action("typing").is_ok());
        assert!(validate_chat_action("looking").is_ok());
        assert!(validate_chat_action("invalid").is_err());
    }

    #[test]
    fn test_validate_cursor() {
        assert!(validate_cursor("123").is_ok());
        assert!(validate_cursor("0").is_ok());
        assert!(validate_cursor("abc").is_err());
        assert!(validate_cursor("12.5").is_err());
    }
}