//! Utilities module for VK Teams Bot CLI
//!
//! This module contains common utility functions and helpers that are used
//! across different parts of the CLI application.

pub mod validation;
pub mod output;
pub mod time;
pub mod config_helpers;
pub mod path;
pub mod bot;
pub mod error_handling;

// Re-export commonly used utilities for internal use
pub use bot::{create_bot_instance, create_dummy_bot, needs_bot_instance};
pub use time::{parse_schedule_time, parse_schedule_time_compat, parse_relative_time, format_duration, format_datetime};
pub use config_helpers::{get_config_paths, merge_configs, validate_config, load_config_with_env_overrides};
pub use output::{print_success_result, print_success_message, print_error_message, print_warning_message, print_info_message};

// Re-export all validation functions
pub use validation::{
    // Chat validation
    validate_chat_id, validate_chat_title, validate_chat_about, validate_chat_action, validate_cursor,
    // File validation  
    validate_file_path, validate_directory_path, validate_file_id, validate_voice_file_path,
    // Message validation
    validate_message_text, validate_message_id,
    // Generic validation
    validate_not_empty, validate_length, validate_range,
};

// TODO: Enable these when needed
// pub use path::*;
// pub use error_handling::*;