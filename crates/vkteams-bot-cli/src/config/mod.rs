//! Configuration management for VK Teams Bot CLI
//!
//! This module provides configuration management with support for both
//! legacy CLI-specific configuration and new unified configuration system.

pub mod legacy;
pub mod unified_adapter;

// Re-export the main config types for backwards compatibility
pub use legacy::{
    Config, ApiConfig, FileConfig, LoggingConfig, UiConfig, ProxyConfig, RateLimitConfig,
    CONFIG, // Re-export the static CONFIG instance
    // Re-export default functions for backwards compatibility
    default_timeout, default_retries, default_max_file_size, default_buffer_size,
    default_log_level, default_log_format, default_log_colors, default_show_progress,
    default_progress_style, default_progress_refresh_rate, default_rate_limit_enabled,
    default_rate_limit_limit, default_rate_limit_duration, default_rate_limit_retry_delay,
    default_rate_limit_retry_attempts
};

// Export the unified adapter for new code
pub use unified_adapter::UnifiedConfigAdapter;