//! Validation utilities for VK Teams Bot CLI
//!
//! This module provides validation functions for various input types
//! used throughout the CLI application.

pub mod chat;
pub mod file;
pub mod message;
pub mod time;

// Re-export all validation functions
pub use chat::*;
pub use file::*;
pub use message::*;
pub use time::*;

use crate::errors::prelude::{CliError, Result as CliResult};

/// Generic validation trait for implementing custom validators
pub trait Validator<T> {
    fn validate(&self, value: &T) -> CliResult<()>;
}

/// Validate that a string is not empty after trimming
pub fn validate_not_empty(value: &str, field_name: &str) -> CliResult<()> {
    if value.trim().is_empty() {
        return Err(CliError::InputError(format!(
            "{field_name} cannot be empty"
        )));
    }
    Ok(())
}

/// Validate string length is within bounds
pub fn validate_length(value: &str, field_name: &str, min: usize, max: usize) -> CliResult<()> {
    let len = value.len();
    if len < min {
        return Err(CliError::InputError(format!(
            "{field_name} too short (min {min} characters, got {len})"
        )));
    }
    if len > max {
        return Err(CliError::InputError(format!(
            "{field_name} too long (max {max} characters, got {len})"
        )));
    }
    Ok(())
}

/// Validate that a numeric value is within range
pub fn validate_range<T>(value: T, field_name: &str, min: T, max: T) -> CliResult<()>
where
    T: PartialOrd + std::fmt::Display,
{
    if value < min || value > max {
        return Err(CliError::InputError(format!(
            "{field_name} must be between {min} and {max} (got {value})"
        )));
    }
    Ok(())
}
