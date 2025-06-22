//! Configuration management for VK Teams Bot CLI
//!
//! This module provides configuration management with support for both
//! legacy CLI-specific configuration and new unified configuration system.

pub mod legacy;
pub mod unified_adapter;

// Re-export the main config types for backwards compatibility
pub use legacy::{
    ApiConfig,
    // Re-export async and cache types
    AsyncConfigManager,
    CONFIG, // Re-export the static CONFIG instance
    Config,
    FileConfig,
    LockFreeConfigCache,
    LoggingConfig,
    ProxyConfig,
    RateLimitConfig,
    UiConfig,
    default_buffer_size,
    default_log_colors,
    default_log_format,
    default_log_level,
    default_max_file_size,
    default_progress_refresh_rate,
    default_progress_style,
    default_rate_limit_duration,
    default_rate_limit_enabled,
    default_rate_limit_limit,
    default_rate_limit_retry_attempts,
    default_rate_limit_retry_delay,
    default_retries,
    default_show_progress,
    // Re-export default functions for backwards compatibility
    default_timeout,
};

// Export the unified adapter for new code
pub use unified_adapter::UnifiedConfigAdapter;
