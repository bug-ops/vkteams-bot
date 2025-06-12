//! Utilities module for VK Teams Bot CLI
//!
//! This module contains common utility functions and helpers that are used
//! across different parts of the CLI application.

pub mod bot;
pub mod config_helpers;
pub mod error_handling;
pub mod output;
pub mod path;
pub mod time;
pub mod validation;

// Re-export commonly used utilities for internal use
pub use bot::{create_bot_instance, create_dummy_bot, needs_bot_instance};
pub use config_helpers::{
    get_config_paths, load_config_with_env_overrides, merge_configs, validate_config,
};
pub use output::{
    print_error_message, print_info_message, print_success_message, print_success_result,
    print_warning_message,
};
pub use time::{
    format_datetime, format_duration, parse_relative_time, parse_schedule_time,
    parse_schedule_time_compat,
};

// Re-export all validation functions
pub use validation::{
    validate_chat_about,
    validate_chat_action,
    // Chat validation
    validate_chat_id,
    validate_chat_title,
    validate_cursor,
    validate_directory_path,
    validate_file_id,
    // File validation
    validate_file_path,
    validate_length,
    validate_message_id,
    // Message validation
    validate_message_text,
    // Generic validation
    validate_not_empty,
    validate_range,
    validate_voice_file_path,
};

pub use path::{ensure_directory_exists, get_file_extension, get_file_name_from_path};
