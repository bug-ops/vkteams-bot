//! VK Teams Bot CLI Library
//!
//! This library provides the core functionality for the VK Teams Bot CLI application.
//! It is organized into modular components for better maintainability and testing.

pub mod commands;
pub mod completion;
pub mod config;
pub mod constants;
pub mod errors;
pub mod file_utils;
pub mod progress;
pub mod scheduler;
pub mod utils;

// Re-export commonly used types for convenience
pub use commands::Command;
pub use config::Config;
pub use errors::prelude::*;
pub use utils::*;