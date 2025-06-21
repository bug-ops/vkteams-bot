use crate::config::legacy::Config as CliConfig;
use crate::errors::prelude::Result as CliResult;
use vkteams_bot::config::unified::UnifiedConfig;
use std::path::Path;

/// Adapter for converting between CLI-specific config and unified config
pub struct UnifiedConfigAdapter;

impl UnifiedConfigAdapter {
    /// Load unified configuration with CLI compatibility
    /// 
    /// This method tries to load configuration in the following order:
    /// 1. Unified configuration file (shared-config.toml)
    /// 2. Legacy CLI configuration file (cli_config.toml)
    /// 3. Environment variables
    /// 4. Default values
    pub fn load() -> CliResult<CliConfig> {
        // Try to load unified config first
        if let Ok(unified_config) = Self::load_unified_config() {
            return Ok(Self::convert_from_unified(unified_config));
        }

        // Fall back to legacy CLI config loading
        CliConfig::load()
    }

    /// Load unified configuration from custom path
    pub fn load_from_path(path: &Path) -> CliResult<CliConfig> {
        if Self::is_unified_config_path(path) {
            if let Ok(unified_config) = Self::load_unified_config_from_path(path) {
                Ok(Self::convert_from_unified(unified_config))
            } else {
                // Fall back to legacy if unified config fails
                CliConfig::from_path(path)
            }
        } else {
            // Load as legacy CLI config
            CliConfig::from_path(path)
        }
    }

    /// Load unified configuration from default locations
    fn load_unified_config() -> Result<UnifiedConfig, Box<dyn std::error::Error>> {
        // Try to load from standard unified config locations
        let unified_paths = vec![
            "./shared-config.toml",
            "./config/config.toml",
            "/app/config/config.toml",
        ];

        for path in unified_paths {
            if Path::new(path).exists() {
                return Self::load_unified_config_from_path(Path::new(path));
            }
        }

        // If no file found, return error
        Err("No unified config file found".into())
    }

    /// Load unified configuration from specific path
    fn load_unified_config_from_path(path: &Path) -> Result<UnifiedConfig, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: UnifiedConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Convert unified config to CLI config
    fn convert_from_unified(unified: UnifiedConfig) -> CliConfig {
        CliConfig {
            api: crate::config::legacy::ApiConfig {
                token: unified.api.token,
                url: Some(unified.api.url),
                timeout: unified.api.timeout,
                max_retries: unified.api.max_retries,
            },
            files: crate::config::legacy::FileConfig {
                download_dir: unified.cli.files.download_dir.map(|p| p.to_string_lossy().to_string()),
                upload_dir: unified.cli.files.upload_dir.map(|p| p.to_string_lossy().to_string()),
                max_file_size: unified.cli.files.max_file_size,
                buffer_size: 8192, // Default buffer size as it's not in unified config
            },
            logging: crate::config::legacy::LoggingConfig {
                level: unified.cli.logging.level,
                format: unified.cli.logging.format,
                colors: unified.cli.ui.colored_output, // Map from UI colored output
            },
            ui: crate::config::legacy::UiConfig {
                show_progress: unified.cli.ui.show_progress,
                progress_style: "bar".to_string(), // Default progress style
                progress_refresh_rate: 10, // Default refresh rate
            },
            proxy: None, // Proxy config not implemented in unified yet
            rate_limit: crate::config::legacy::RateLimitConfig::default(),
        }
    }

    /// Convert CLI config to unified config (placeholder implementation)
    pub fn convert_to_unified(_cli_config: CliConfig) -> UnifiedConfig {
        // For now, return a default unified config
        // This would need proper implementation based on requirements
        UnifiedConfig::default()
    }

    /// Check if path is for unified config based on file name
    fn is_unified_config_path(path: &Path) -> bool {
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            matches!(file_name, "shared-config.toml" | "config.toml")
        } else {
            false
        }
    }
}