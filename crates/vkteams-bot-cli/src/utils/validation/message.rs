//! Message-related validation functions
//!
//! This module contains validation functions for message text, message IDs,
//! and other message-related content.

use super::{validate_length, validate_not_empty};
use crate::constants::validation;
use crate::errors::prelude::{CliError, Result as CliResult};

/// Validate message text content
///
/// # Arguments
/// * `message` - The message text to validate
///
/// # Returns
/// * `Ok(())` if the message is valid
/// * `Err(CliError::InputError)` if the message is invalid
///
/// # Validation Rules
/// - Cannot be empty after trimming
/// - Must be between 1 and 4096 characters
/// - Basic content validation (no null bytes)
pub fn validate_message_text(message: &str) -> CliResult<()> {
    validate_not_empty(message, "Message")?;
    validate_length(
        message,
        "Message",
        validation::MIN_MESSAGE_LENGTH,
        validation::MAX_MESSAGE_LENGTH,
    )?;

    // Check for null bytes which can cause issues
    if message.contains('\0') {
        return Err(CliError::InputError(
            "Message cannot contain null bytes".to_string(),
        ));
    }

    Ok(())
}

/// Validate a message ID
///
/// # Arguments
/// * `message_id` - The message ID to validate
///
/// # Returns
/// * `Ok(())` if the message ID is valid
/// * `Err(CliError::InputError)` if the message ID is invalid
pub fn validate_message_id(message_id: &str) -> CliResult<()> {
    validate_not_empty(message_id, "Message ID")?;

    // Message IDs should be alphanumeric with possible hyphens and underscores
    if !message_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(CliError::InputError(
            "Message ID contains invalid characters. Only alphanumeric, hyphen, and underscore are allowed".to_string()
        ));
    }

    Ok(())
}

/// Validate message formatting and content for potential issues
///
/// # Arguments
/// * `message` - The message text to validate for formatting
///
/// # Returns
/// * `Ok(())` if the message formatting is valid
/// * `Err(CliError::InputError)` if there are formatting issues
pub fn validate_message_formatting(message: &str) -> CliResult<()> {
    // Check for excessively long lines that might cause display issues
    let max_line_length = 1000;
    for (line_num, line) in message.lines().enumerate() {
        if line.len() > max_line_length {
            return Err(CliError::InputError(format!(
                "Line {} is too long ({} characters, max {})",
                line_num + 1,
                line.len(),
                max_line_length
            )));
        }
    }

    // Check for excessive consecutive whitespace
    if message.contains("    ") { // 4+ spaces
        // This is just a warning case, not an error
    }

    // Check for potential markdown injection issues
    let suspicious_patterns = ["](javascript:", "](data:", "](vbscript:"];
    for pattern in &suspicious_patterns {
        if message.to_lowercase().contains(pattern) {
            return Err(CliError::InputError(
                "Message contains potentially unsafe URL patterns".to_string(),
            ));
        }
    }

    Ok(())
}

/// Validate message content for basic spam/abuse patterns
///
/// # Arguments
/// * `message` - The message text to validate
///
/// # Returns
/// * `Ok(())` if the message passes basic content validation
/// * `Err(CliError::InputError)` if potential spam/abuse patterns are detected
pub fn validate_message_content(message: &str) -> CliResult<()> {
    // Check for excessive repetition
    let max_repeated_chars = 50;
    let mut current_char = '\0';
    let mut repeat_count = 0;

    for ch in message.chars() {
        if ch == current_char {
            repeat_count += 1;
            if repeat_count > max_repeated_chars {
                return Err(CliError::InputError(format!(
                    "Message contains excessive character repetition (more than {max_repeated_chars} consecutive '{ch}')"
                )));
            }
        } else {
            current_char = ch;
            repeat_count = 1;
        }
    }

    // Check for excessive uppercase (potential shouting)
    let uppercase_ratio = message
        .chars()
        .filter(|c| c.is_alphabetic())
        .map(|c| if c.is_uppercase() { 1.0 } else { 0.0 })
        .sum::<f64>()
        / message.chars().filter(|c| c.is_alphabetic()).count().max(1) as f64;

    if uppercase_ratio > 0.8 && message.len() > 20 {
        // This is more of a warning than an error for CLI usage
        // Users might legitimately want to send uppercase messages
    }

    Ok(())
}

/// Validate that message text is appropriate for the given context
///
/// # Arguments
/// * `message` - The message text to validate
/// * `is_group_chat` - Whether this message is being sent to a group chat
///
/// # Returns
/// * `Ok(())` if the message is appropriate for the context
/// * `Err(CliError::InputError)` if there are context-specific issues
pub fn validate_message_context(message: &str, is_group_chat: bool) -> CliResult<()> {
    // For group chats, warn about @everyone/@channel mentions
    if is_group_chat {
        let mention_patterns = ["@everyone", "@channel", "@all"];
        for pattern in &mention_patterns {
            if message.to_lowercase().contains(pattern) {
                // In a CLI context, this might be intentional, so we don't error
                // but we could log a warning if logging is enabled
            }
        }
    }

    // Check for extremely long single words that might break formatting
    let max_word_length = 200;
    for word in message.split_whitespace() {
        if word.len() > max_word_length {
            return Err(CliError::InputError(format!(
                "Message contains a word that is too long ({} characters, max {})",
                word.len(),
                max_word_length
            )));
        }
    }

    Ok(())
}

/// Perform comprehensive message validation
///
/// # Arguments
/// * `message` - The message text to validate
/// * `is_group_chat` - Whether this message is being sent to a group chat
///
/// # Returns
/// * `Ok(())` if the message passes all validation checks
/// * `Err(CliError::InputError)` if any validation check fails
pub fn validate_message_comprehensive(message: &str, is_group_chat: bool) -> CliResult<()> {
    validate_message_text(message)?;
    validate_message_formatting(message)?;
    validate_message_content(message)?;
    validate_message_context(message, is_group_chat)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_message_text() {
        assert!(validate_message_text("Hello, world!").is_ok());
        assert!(validate_message_text("").is_err());
        assert!(validate_message_text("   ").is_err());
        assert!(validate_message_text("A").is_ok());

        // Test max length
        let long_message = "a".repeat(4097);
        assert!(validate_message_text(&long_message).is_err());

        let max_length_message = "a".repeat(4096);
        assert!(validate_message_text(&max_length_message).is_ok());
    }

    #[test]
    fn test_validate_message_id() {
        assert!(validate_message_id("msg123").is_ok());
        assert!(validate_message_id("msg_123").is_ok());
        assert!(validate_message_id("msg-123").is_ok());
        assert!(validate_message_id("123").is_ok());
        assert!(validate_message_id("").is_err());
        assert!(validate_message_id("msg@123").is_err());
        assert!(validate_message_id("msg.123").is_err());
    }

    #[test]
    fn test_validate_message_formatting() {
        assert!(validate_message_formatting("Normal message").is_ok());

        // Test long line
        let long_line = "a".repeat(1001);
        assert!(validate_message_formatting(&long_line).is_err());

        // Test suspicious patterns
        assert!(validate_message_formatting("Click [here](javascript:alert('xss'))").is_err());
        assert!(validate_message_formatting("Safe [link](https://example.com)").is_ok());
    }

    #[test]
    fn test_validate_message_content() {
        assert!(validate_message_content("Normal message").is_ok());

        // Test excessive repetition
        let repeated = "a".repeat(51);
        assert!(validate_message_content(&repeated).is_err());

        let acceptable_repeat = "a".repeat(50);
        assert!(validate_message_content(&acceptable_repeat).is_ok());
    }

    #[test]
    fn test_validate_message_context() {
        assert!(validate_message_context("Hello everyone", true).is_ok());
        assert!(validate_message_context("Hello everyone", false).is_ok());

        // Test long word
        let long_word = format!("This is a {} word", "a".repeat(201));
        assert!(validate_message_context(&long_word, false).is_err());
    }

    #[test]
    fn test_validate_message_with_null_bytes() {
        let message_with_null = "Hello\0World";
        assert!(validate_message_text(message_with_null).is_err());
    }

    #[test]
    fn test_validate_message_comprehensive() {
        assert!(validate_message_comprehensive("Hello, world!", false).is_ok());
        assert!(validate_message_comprehensive("", false).is_err());

        let problematic_message = "a".repeat(51); // Too much repetition
        assert!(validate_message_comprehensive(&problematic_message, false).is_err());
    }
}

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_validate_message_text_random(s in ".{0,8192}") {
            let _ = validate_message_text(&s);
        }

        #[test]
        fn prop_validate_message_id_random(s in ".{0,256}") {
            let _ = validate_message_id(&s);
        }

        #[test]
        fn prop_validate_message_formatting_random(s in ".{0,4096}") {
            let _ = validate_message_formatting(&s);
        }

        #[test]
        fn prop_validate_message_content_random(s in ".{0,4096}") {
            let _ = validate_message_content(&s);
        }

        #[test]
        fn prop_validate_message_context_random(s in ".{0,4096}", is_group in proptest::bool::ANY) {
            let _ = validate_message_context(&s, is_group);
        }
    }
}
