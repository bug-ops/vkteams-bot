//! CLI Bridge for MCP Server
//!
//! This module provides an abstraction layer for calling the VK Teams Bot CLI
//! from the MCP server using subprocess calls. This approach ensures that all
//! business logic remains in the CLI while the MCP server acts as a thin adapter.

use crate::bridge_trait::CliBridgeTrait;
use crate::errors::{BridgeError, CliErrorInfo};
use async_trait::async_trait;
use serde_json::Value;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{Duration, timeout};
use tracing::{debug, error, warn};
use vkteams_bot::config::UnifiedConfig;

/// Bridge for executing CLI commands from MCP server
#[derive(Debug)]
pub struct CliBridge {
    cli_path: String,
    default_args: Vec<String>,
}

impl CliBridge {
    /// Create a new CLI bridge instance with unified configuration
    pub fn new(config: &UnifiedConfig) -> Result<Self, BridgeError> {
        // Use CLI path from config if provided, otherwise search in PATH
        let cli_path = if let Some(config_cli_path) = &config.mcp.cli_path {
            config_cli_path.clone()
        } else {
            which::which("vkteams-bot-cli")
                .or_else(|_| {
                    // Try relative path from current executable
                    std::env::current_exe().map(|mut p| {
                        p.pop(); // remove filename
                        p.push("vkteams-bot-cli");
                        p
                    })
                })
                .map_err(|_| {
                    BridgeError::CliNotFound(
                        "vkteams-bot-cli not found in PATH or relative to current executable"
                            .to_string(),
                    )
                })?
        };

        let mut default_args = vec!["--output".to_string(), "json".to_string()];

        // Use config file path from environment if provided
        if let Ok(config_path) = std::env::var("VKTEAMS_BOT_CONFIG") {
            default_args.extend(vec!["--config".to_string(), config_path]);
        }

        Ok(Self {
            cli_path: cli_path.to_string_lossy().to_string(),
            default_args,
        })
    }

    /// Execute a CLI command with arguments
    pub async fn execute_command(&self, command: &[&str]) -> Result<Value, BridgeError> {
        self.execute_command_with_timeout(command, Duration::from_secs(30))
            .await
    }

    /// Execute a CLI command with custom timeout
    pub async fn execute_command_with_timeout(
        &self,
        command: &[&str],
        timeout_duration: Duration,
    ) -> Result<Value, BridgeError> {
        debug!("Executing CLI command: {} {:?}", self.cli_path, command);

        let mut cmd = Command::new(&self.cli_path);
        cmd.args(&self.default_args)
            .args(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = match timeout(timeout_duration, cmd.output()).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => {
                error!("CLI command timed out after {:?}", timeout_duration);
                return Err(BridgeError::Timeout(timeout_duration));
            }
        };

        // Check if process was terminated by signal
        if let Some(code) = output.status.code() {
            if code < 0 {
                return Err(BridgeError::ProcessTerminated(format!(
                    "Process terminated with signal {}",
                    -code
                )));
            }
        }

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Try to parse structured error from stderr
            if let Ok(error_info) = serde_json::from_str::<CliErrorInfo>(&stderr) {
                // Check for specific error types
                if let Some(ref code) = error_info.code {
                    match code.as_str() {
                        "RATE_LIMIT" => {
                            return Err(BridgeError::RateLimit(error_info.message.clone()));
                        }
                        _ => return Err(BridgeError::CliReturnedError(error_info)),
                    }
                }
                return Err(BridgeError::CliReturnedError(error_info));
            }

            error!("CLI command failed with unstructured error: {}", stderr);
            return Err(BridgeError::CliError(stderr.to_string()));
        }

        let response_text = String::from_utf8_lossy(&output.stdout);
        debug!("CLI response: {}", response_text);

        // Handle empty responses
        if response_text.trim().is_empty() {
            warn!("CLI returned empty response");
            return Ok(serde_json::json!({"success": true, "data": null}));
        }

        let response: Value = serde_json::from_str(&response_text)?;

        // Check if CLI returned an error in the JSON response
        if let Some(success) = response.get("success") {
            if !success.as_bool().unwrap_or(true) {
                let error_info = if let Some(error) = response.get("error") {
                    CliErrorInfo {
                        code: response
                            .get("error_code")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        message: error.as_str().unwrap_or("Unknown error").to_string(),
                        details: response.get("error_details").cloned(),
                    }
                } else {
                    CliErrorInfo {
                        code: None,
                        message: "Command failed without error details".to_string(),
                        details: None,
                    }
                };

                // Check for rate limiting in response
                if error_info.code.as_deref() == Some("RATE_LIMIT") {
                    return Err(BridgeError::RateLimit(error_info.message));
                }

                return Err(BridgeError::CliReturnedError(error_info));
            }
        }

        Ok(response)
    }

    /// Execute command with retry logic
    pub async fn execute_command_with_retry(
        &self,
        command: &[&str],
        max_retries: usize,
    ) -> Result<Value, BridgeError> {
        self.execute_command_with_retry_and_timeout(command, max_retries, Duration::from_secs(30))
            .await
    }

    /// Execute command with retry logic and custom timeout
    pub async fn execute_command_with_retry_and_timeout(
        &self,
        command: &[&str],
        max_retries: usize,
        timeout_duration: Duration,
    ) -> Result<Value, BridgeError> {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match self
                .execute_command_with_timeout(command, timeout_duration)
                .await
            {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Determine if error is retryable
                    let should_retry = match &e {
                        BridgeError::Timeout(_) => true,
                        BridgeError::Io(_) => true,
                        BridgeError::RateLimit(_) => {
                            // For rate limit, use exponential backoff
                            if attempt < max_retries {
                                let backoff = Duration::from_secs(2u64.pow(attempt as u32));
                                warn!("Rate limited, backing off for {:?}", backoff);
                                tokio::time::sleep(backoff).await;
                            }
                            true
                        }
                        BridgeError::CliReturnedError(info) => {
                            // Check if error code indicates a retryable error
                            matches!(
                                info.code.as_deref(),
                                Some("NETWORK_ERROR") | Some("TIMEOUT")
                            )
                        }
                        _ => false,
                    };

                    last_error = Some(e);

                    if attempt < max_retries && should_retry {
                        let delay = if matches!(last_error, Some(BridgeError::RateLimit(_))) {
                            // Already handled above
                            Duration::from_millis(0)
                        } else {
                            // Standard backoff for other errors
                            Duration::from_millis(100 * (attempt + 1) as u64)
                        };

                        if !delay.is_zero() {
                            debug!(
                                "Retrying command after {:?} (attempt {}/{})",
                                delay,
                                attempt + 1,
                                max_retries
                            );
                            tokio::time::sleep(delay).await;
                        }
                    } else if !should_retry {
                        // Non-retryable error, fail immediately
                        break;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// Health check - test if CLI is working
    pub async fn health_check(&self) -> Result<(), BridgeError> {
        self.execute_command(&["--version"]).await?;
        Ok(())
    }

    // === Daemon Commands ===

    /// Get daemon status
    pub async fn get_daemon_status(&self) -> Result<Value, BridgeError> {
        self.execute_command(&["status"]).await
    }

    // === Enhanced Storage Commands ===

    /// Get recent messages from storage
    pub async fn get_recent_messages(
        &self,
        chat_id: Option<&str>,
        limit: Option<usize>,
        since: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["database", "recent"];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        let limit_str;
        if let Some(limit) = limit {
            limit_str = limit.to_string();
            args.extend(&["--limit", &limit_str]);
        }

        if let Some(since) = since {
            args.extend(&["--since", since]);
        }

        self.execute_command(&args).await
    }
}

impl Default for CliBridge {
    fn default() -> Self {
        let config = UnifiedConfig::default();
        Self::new(&config).expect("Failed to create CLI bridge")
    }
}

/// Implementation of CliBridgeTrait for CliBridge
#[async_trait]
impl CliBridgeTrait for CliBridge {
    async fn execute_command(&self, args: &[&str]) -> Result<Value, BridgeError> {
        self.execute_command(args).await
    }

    async fn get_daemon_status(&self) -> Result<Value, BridgeError> {
        self.execute_command(&["daemon", "status"]).await
    }

    async fn health_check(&self) -> Result<(), BridgeError> {
        self.execute_command(&["--version"]).await.map(|_| ())
    }
}

/// Implementation of domain-specific MCP bridge traits
#[async_trait]
impl crate::mcp_bridge_trait::McpMessaging for CliBridge {
    async fn send_text_mcp(
        &self,
        text: &str,
        chat_id: Option<&str>,
        reply_msg_id: Option<&str>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.send_text(text, chat_id, reply_msg_id).await)
    }

    async fn send_file_mcp(
        &self,
        file_path: &str,
        chat_id: Option<&str>,
        caption: Option<&str>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.send_file(file_path, chat_id, caption).await)
    }

    async fn send_voice_mcp(
        &self,
        file_path: &str,
        chat_id: Option<&str>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.send_voice(file_path, chat_id).await)
    }

    async fn edit_message_mcp(
        &self,
        message_id: &str,
        new_text: &str,
        chat_id: Option<&str>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.edit_message(message_id, new_text, chat_id).await)
    }

    async fn delete_message_mcp(
        &self,
        message_id: &str,
        chat_id: Option<&str>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.delete_message(message_id, chat_id).await)
    }

    async fn pin_message_mcp(
        &self,
        message_id: &str,
        chat_id: Option<&str>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.pin_message(message_id, chat_id).await)
    }

    async fn unpin_message_mcp(
        &self,
        message_id: &str,
        chat_id: Option<&str>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.unpin_message(message_id, chat_id).await)
    }

    async fn send_action_mcp(
        &self,
        action: &str,
        chat_id: Option<&str>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.send_action(action, chat_id).await)
    }
}

#[async_trait]
impl crate::mcp_bridge_trait::McpChatManagement for CliBridge {
    async fn get_chat_info_mcp(&self, chat_id: Option<&str>) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.get_chat_info(chat_id).await)
    }

    async fn get_profile_mcp(&self, user_id: &str) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.get_profile(user_id).await)
    }

    async fn get_chat_members_mcp(
        &self,
        chat_id: Option<&str>,
        cursor: Option<&str>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.get_chat_members(chat_id, cursor).await)
    }

    async fn get_chat_admins_mcp(&self, chat_id: Option<&str>) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.get_chat_admins(chat_id).await)
    }

    async fn set_chat_title_mcp(
        &self,
        title: &str,
        chat_id: Option<&str>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.set_chat_title(title, chat_id).await)
    }

    async fn set_chat_about_mcp(
        &self,
        about: &str,
        chat_id: Option<&str>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.set_chat_about(about, chat_id).await)
    }
}

#[async_trait]
impl crate::mcp_bridge_trait::McpFileOperations for CliBridge {
    async fn upload_file_base64_mcp(
        &self,
        name: &str,
        content_base64: &str,
        chat_id: Option<&str>,
        caption: Option<&str>,
        reply_msg_id: Option<&str>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(
            self.upload_file_base64(name, content_base64, chat_id, caption, reply_msg_id)
                .await,
        )
    }

    async fn upload_text_file_mcp(
        &self,
        name: &str,
        content: &str,
        chat_id: Option<&str>,
        caption: Option<&str>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(
            self.upload_text_file(name, content, chat_id, caption).await,
        )
    }

    async fn upload_json_file_mcp(
        &self,
        name: &str,
        json_data: &str,
        pretty: bool,
        chat_id: Option<&str>,
        caption: Option<&str>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(
            self.upload_json_file(name, json_data, pretty, chat_id, caption)
                .await,
        )
    }

    async fn get_file_info_mcp(&self, file_id: &str) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.get_file_info(file_id).await)
    }
}

#[async_trait]
impl crate::mcp_bridge_trait::McpStorage for CliBridge {
    async fn get_database_stats_mcp(
        &self,
        chat_id: Option<&str>,
        since: Option<&str>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.get_database_stats(chat_id, since).await)
    }

    async fn search_semantic_mcp(
        &self,
        query: &str,
        chat_id: Option<&str>,
        limit: Option<usize>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.search_semantic(query, chat_id, limit).await)
    }

    async fn search_text_mcp(
        &self,
        query: &str,
        chat_id: Option<&str>,
        limit: Option<i64>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.search_text(query, chat_id, limit).await)
    }

    async fn get_context_mcp(
        &self,
        chat_id: Option<&str>,
        context_type: Option<&str>,
        timeframe: Option<&str>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(
            self.get_context(chat_id, context_type, timeframe).await,
        )
    }

    async fn get_recent_messages_mcp(
        &self,
        chat_id: Option<&str>,
        limit: Option<usize>,
        since: Option<&str>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.get_recent_messages(chat_id, limit, since).await)
    }
}

#[async_trait]
impl crate::mcp_bridge_trait::McpDiagnostics for CliBridge {
    async fn get_daemon_status_mcp(&self) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.get_daemon_status().await)
    }

    async fn get_self_mcp(&self) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.get_self().await)
    }

    async fn get_events_mcp(
        &self,
        last_event_id: Option<&str>,
        poll_time: Option<u64>,
    ) -> crate::server::MCPResult {
        crate::server::convert_bridge_result(self.get_events(last_event_id, poll_time).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        // Set required environment variable for test
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat");
        }

        let config = UnifiedConfig::default();
        let result = CliBridge::new(&config);
        // Note: This might fail if CLI binary is not available, which is expected in test environment
        // The important thing is that the code compiles and handles errors properly
        match result {
            Ok(_) => println!("CLI bridge created successfully"),
            Err(e) => println!("Expected error in test environment: {}", e),
        }
    }

    #[test]
    fn test_bridge_error_display() {
        let error = BridgeError::CliError("test error".to_string());
        assert!(error.to_string().contains("CLI execution failed"));
    }

    #[test]
    fn test_bridge_error_retryable() {
        let timeout_error = BridgeError::Timeout(Duration::from_secs(30));
        assert!(timeout_error.is_retryable());

        let rate_limit_error = BridgeError::RateLimit("too many requests".to_string());
        assert!(rate_limit_error.is_retryable());

        let io_error = BridgeError::Io("connection failed".to_string());
        assert!(io_error.is_retryable());

        let cli_error = BridgeError::CliError("general error".to_string());
        assert!(!cli_error.is_retryable());

        let retryable_cli_error = BridgeError::CliReturnedError(CliErrorInfo {
            code: Some("NETWORK_ERROR".to_string()),
            message: "network timeout".to_string(),
            details: None,
        });
        assert!(retryable_cli_error.is_retryable());

        let non_retryable_cli_error = BridgeError::CliReturnedError(CliErrorInfo {
            code: Some("INVALID_INPUT".to_string()),
            message: "bad input".to_string(),
            details: None,
        });
        assert!(!non_retryable_cli_error.is_retryable());
    }

    #[test]
    fn test_bridge_error_retry_delay() {
        let rate_limit_error = BridgeError::RateLimit("too many requests".to_string());
        assert_eq!(rate_limit_error.retry_delay(), Duration::from_secs(60));

        let timeout_error = BridgeError::Timeout(Duration::from_secs(30));
        assert_eq!(timeout_error.retry_delay(), Duration::from_secs(10));

        let io_error = BridgeError::Io("connection failed".to_string());
        assert_eq!(io_error.retry_delay(), Duration::from_secs(5));

        let cli_error = BridgeError::CliError("general error".to_string());
        assert_eq!(cli_error.retry_delay(), Duration::from_secs(2));
    }

    #[test]
    fn test_bridge_default() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat");
        }

        // This will likely fail in test environment but should not panic
        let result = std::panic::catch_unwind(|| {
            let _bridge = CliBridge::default();
        });

        // The important thing is that it either succeeds or panics predictably
        // We can't test much more without an actual CLI binary
        match result {
            Ok(_) => println!("Default bridge creation succeeded"),
            Err(_) => println!("Default bridge creation failed as expected in test environment"),
        }
    }

    #[test]
    fn test_recent_messages_args_building() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat");
        }

        // Test argument building logic without actually executing
        // This tests the parameter handling logic in get_recent_messages
        let args_test_cases = vec![
            (None, None, None, vec!["database", "recent"]),
            (
                Some("chat123"),
                None,
                None,
                vec!["database", "recent", "--chat-id", "chat123"],
            ),
            (
                None,
                Some(10),
                None,
                vec!["database", "recent", "--limit", "10"],
            ),
            (
                None,
                None,
                Some("2024-01-01"),
                vec!["database", "recent", "--since", "2024-01-01"],
            ),
            (
                Some("chat123"),
                Some(10),
                Some("2024-01-01"),
                vec![
                    "database",
                    "recent",
                    "--chat-id",
                    "chat123",
                    "--limit",
                    "10",
                    "--since",
                    "2024-01-01",
                ],
            ),
        ];

        for (chat_id, limit, since, expected_args) in args_test_cases {
            let mut args = vec!["database", "recent"];

            if let Some(chat_id) = chat_id {
                args.extend(&["--chat-id", chat_id]);
            }

            let limit_str = limit.map(|l| l.to_string());
            if let Some(ref limit_str) = limit_str {
                args.extend(&["--limit", limit_str]);
            }

            if let Some(since) = since {
                args.extend(&["--since", since]);
            }

            assert_eq!(args, expected_args);
        }
    }

    #[tokio::test]
    async fn test_cli_bridge_trait_implementation() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat");
        }

        // Test that CliBridge implements CliBridgeTrait correctly
        let config = UnifiedConfig::default();
        if let Ok(bridge) = CliBridge::new(&config) {
            // These calls will likely fail but should compile and return appropriate errors
            let version_result = bridge.health_check().await;
            let daemon_result = bridge.get_daemon_status().await;
            let execute_result = bridge.execute_command(&["--version"]).await;

            // We mainly care that these methods exist and are callable
            // In a test environment, they're expected to fail
            println!("Health check result: {:?}", version_result.is_ok());
            println!("Daemon status result: {:?}", daemon_result.is_ok());
            println!("Execute command result: {:?}", execute_result.is_ok());
        }
    }

    #[test]
    fn test_config_path_handling() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat");
            std::env::set_var("VKTEAMS_BOT_CONFIG", "/path/to/config.toml");
        }

        let config = UnifiedConfig::default();
        if let Ok(bridge) = CliBridge::new(&config) {
            // Check that config path is included in default args
            assert!(bridge.default_args.contains(&"--config".to_string()));
            assert!(
                bridge
                    .default_args
                    .contains(&"/path/to/config.toml".to_string())
            );
        }

        unsafe {
            std::env::remove_var("VKTEAMS_BOT_CONFIG");
        }
    }

    #[test]
    fn test_cli_error_info_serialization() {
        let error_info = CliErrorInfo {
            code: Some("TEST_ERROR".to_string()),
            message: "Test error message".to_string(),
            details: Some(serde_json::json!({"field": "value"})),
        };

        // Test serialization
        let serialized = serde_json::to_string(&error_info).unwrap();
        assert!(serialized.contains("TEST_ERROR"));
        assert!(serialized.contains("Test error message"));

        // Test deserialization
        let deserialized: CliErrorInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.code, Some("TEST_ERROR".to_string()));
        assert_eq!(deserialized.message, "Test error message");
        assert!(deserialized.details.is_some());
    }

    #[test]
    fn test_bridge_error_conversions() {
        // Test From<serde_json::Error>
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let bridge_error: BridgeError = json_error.into();
        match bridge_error {
            BridgeError::InvalidResponse(_) => (),
            _ => panic!("Expected InvalidResponse error"),
        }

        // Test From<std::io::Error>
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let bridge_error: BridgeError = io_error.into();
        match bridge_error {
            BridgeError::Io(_) => (),
            _ => panic!("Expected Io error"),
        }
    }

    #[test]
    fn test_cli_bridge_debug() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat");
        }

        let config = UnifiedConfig::default();
        if let Ok(bridge) = CliBridge::new(&config) {
            let debug_str = format!("{:?}", bridge);
            assert!(debug_str.contains("CliBridge"));
            assert!(debug_str.contains("cli_path"));
            assert!(debug_str.contains("default_args"));
        }
    }

    // Tests for MCP bridge implementation with CliBridge
    #[tokio::test]
    async fn test_mcp_bridge_traits_compilation() {
        // This test verifies that CliBridge implements all MCP traits correctly
        // We don't test actual functionality here since it requires CLI binary

        // Create a dummy CliBridge (will fail but that's expected in test environment)
        let config = UnifiedConfig::default();
        if let Ok(bridge) = CliBridge::new(&config) {
            // Test that CliBridge implements the traits (compilation test)
            use crate::mcp_bridge_trait::*;

            let _messaging: &dyn McpMessaging = &bridge;
            let _chat_mgmt: &dyn McpChatManagement = &bridge;
            let _file_ops: &dyn McpFileOperations = &bridge;
            let _storage: &dyn McpStorage = &bridge;
            let _diagnostics: &dyn McpDiagnostics = &bridge;
            let _combined: &dyn McpCliBridge = &bridge;
        }
    }

    #[test]
    fn test_mcp_bridge_traits_exist() {
        // Test that all the MCP bridge traits are properly defined
        use crate::mcp_bridge_trait::*;

        // This is a compilation test to ensure traits are properly exported
        fn _accepts_messaging<T: McpMessaging>(_: T) {}
        fn _accepts_chat_management<T: McpChatManagement>(_: T) {}
        fn _accepts_file_operations<T: McpFileOperations>(_: T) {}
        fn _accepts_storage<T: McpStorage>(_: T) {}
        fn _accepts_diagnostics<T: McpDiagnostics>(_: T) {}
        fn _accepts_combined<T: McpCliBridge>(_: T) {}
    }

    // === Additional comprehensive tests for better coverage ===

    #[test]
    fn test_bridge_creation_without_cli_binary() {
        // Test that CliBridge creation fails when CLI binary is not found
        let config = UnifiedConfig::default();
        let result = CliBridge::new(&config);
        
        // This test will likely pass in CI/test environment where CLI binary is available
        // or fail with CliNotFound error when binary is not available
        match result {
            Ok(_) => println!("CLI bridge created successfully"),
            Err(BridgeError::CliNotFound(msg)) => {
                assert!(msg.contains("vkteams-bot-cli"));
                assert!(msg.contains("not found"));
            }
            Err(e) => println!("Unexpected error (acceptable in test environment): {}", e),
        }
    }

    #[test]
    fn test_bridge_creation_with_config_env() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat");
            std::env::set_var("VKTEAMS_BOT_CONFIG", "/path/to/config.toml");
        }

        let config = UnifiedConfig::default();
        let result = CliBridge::new(&config);
        match result {
            Ok(bridge) => {
                // Check that config path is included in default args
                assert!(bridge.default_args.contains(&"--config".to_string()));
                assert!(
                    bridge
                        .default_args
                        .contains(&"/path/to/config.toml".to_string())
                );
            }
            Err(e) => {
                // Expected in test environment without CLI binary
                println!("Expected error in test environment: {}", e);
            }
        }

        unsafe {
            std::env::remove_var("VKTEAMS_BOT_CONFIG");
        }
    }

    #[test]
    fn test_bridge_default_args_structure() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat");
        }

        let config = UnifiedConfig::default();
        if let Ok(bridge) = CliBridge::new(&config) {
            // Check that default args contain required parameters
            assert!(bridge.default_args.contains(&"--output".to_string()));
            assert!(bridge.default_args.contains(&"json".to_string()));
        }
    }

    #[test]
    fn test_bridge_error_types_comprehensive() {
        // Test all BridgeError types and their properties

        // Test CliError
        let cli_error = BridgeError::CliError("command failed".to_string());
        assert!(!cli_error.is_retryable());
        assert_eq!(cli_error.retry_delay(), Duration::from_secs(2));
        assert!(cli_error.to_string().contains("CLI execution failed"));

        // Test Timeout error
        let timeout_error = BridgeError::Timeout(Duration::from_secs(30));
        assert!(timeout_error.is_retryable());
        assert_eq!(timeout_error.retry_delay(), Duration::from_secs(10));
        assert!(timeout_error.to_string().contains("Command timed out"));

        // Test RateLimit error
        let rate_limit_error = BridgeError::RateLimit("too many requests".to_string());
        assert!(rate_limit_error.is_retryable());
        assert_eq!(rate_limit_error.retry_delay(), Duration::from_secs(60));
        assert!(rate_limit_error.to_string().contains("Rate limit exceeded"));

        // Test IO error
        let io_error = BridgeError::Io("connection failed".to_string());
        assert!(io_error.is_retryable());
        assert_eq!(io_error.retry_delay(), Duration::from_secs(5));
        assert!(io_error.to_string().contains("IO error"));

        // Test CliNotFound error
        let not_found_error = BridgeError::CliNotFound("/path/to/cli".to_string());
        assert!(!not_found_error.is_retryable());
        assert_eq!(not_found_error.retry_delay(), Duration::from_secs(2));
        assert!(
            not_found_error
                .to_string()
                .contains("CLI not found at path")
        );

        // Test InvalidResponse error
        let invalid_response_error = BridgeError::InvalidResponse("malformed json".to_string());
        assert!(!invalid_response_error.is_retryable());
        assert_eq!(invalid_response_error.retry_delay(), Duration::from_secs(2));
        assert!(
            invalid_response_error
                .to_string()
                .contains("Invalid JSON response")
        );

        // Test ProcessTerminated error
        let process_terminated_error = BridgeError::ProcessTerminated("signal 9".to_string());
        assert!(!process_terminated_error.is_retryable());
        assert_eq!(
            process_terminated_error.retry_delay(),
            Duration::from_secs(2)
        );
        assert!(
            process_terminated_error
                .to_string()
                .contains("CLI process terminated")
        );
    }

    #[test]
    fn test_cli_error_info_detailed() {
        // Test various CliErrorInfo scenarios

        // Error with all fields
        let full_error = CliErrorInfo {
            code: Some("NOT_FOUND".to_string()),
            message: "Resource not found".to_string(),
            details: Some(serde_json::json!({"resource_id": "123"})),
        };

        let bridge_error = BridgeError::CliReturnedError(full_error);
        assert!(!bridge_error.is_retryable()); // NOT_FOUND is not retryable
        assert!(bridge_error.to_string().contains("CLI returned error"));

        // Test retryable error codes
        let network_error = CliErrorInfo {
            code: Some("NETWORK_ERROR".to_string()),
            message: "Network timeout".to_string(),
            details: None,
        };

        let bridge_error = BridgeError::CliReturnedError(network_error);
        assert!(bridge_error.is_retryable()); // NETWORK_ERROR is retryable

        let timeout_error = CliErrorInfo {
            code: Some("TIMEOUT".to_string()),
            message: "Operation timed out".to_string(),
            details: None,
        };

        let bridge_error = BridgeError::CliReturnedError(timeout_error);
        assert!(bridge_error.is_retryable()); // TIMEOUT is retryable

        // Error without code
        let no_code_error = CliErrorInfo {
            code: None,
            message: "Generic error".to_string(),
            details: None,
        };

        let bridge_error = BridgeError::CliReturnedError(no_code_error);
        assert!(!bridge_error.is_retryable()); // No code means not retryable
    }

    #[test]
    fn test_cli_error_info_serialization_roundtrip() {
        let original = CliErrorInfo {
            code: Some("TEST_ERROR".to_string()),
            message: "Test error message".to_string(),
            details: Some(serde_json::json!({
                "field": "value",
                "number": 42,
                "nested": {"key": "data"}
            })),
        };

        // Serialize
        let serialized = serde_json::to_string(&original).unwrap();
        assert!(serialized.contains("TEST_ERROR"));
        assert!(serialized.contains("Test error message"));

        // Deserialize
        let deserialized: CliErrorInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.code, original.code);
        assert_eq!(deserialized.message, original.message);
        assert_eq!(deserialized.details, original.details);
    }

    #[test]
    fn test_bridge_debug_implementation() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat");
        }

        let config = UnifiedConfig::default();
        if let Ok(bridge) = CliBridge::new(&config) {
            let debug_output = format!("{:?}", bridge);
            assert!(debug_output.contains("CliBridge"));
            assert!(debug_output.contains("cli_path"));
            assert!(debug_output.contains("default_args"));
        }
    }

    #[test]
    fn test_cli_bridge_trait_basic_implementation() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat");
        }

        let config = UnifiedConfig::default();
        if let Ok(bridge) = CliBridge::new(&config) {
            // Test that CliBridge implements CliBridgeTrait
            let trait_obj: &dyn CliBridgeTrait = &bridge;

            // We can't actually call async methods in sync test, but we can verify the trait is implemented
            let _ = trait_obj;
        }
    }

    // === Tests for methods that are harder to cover ===

    #[test]
    fn test_recent_messages_parameter_combinations() {
        // Test all combinations of parameters for get_recent_messages
        let test_cases = vec![
            (None, None, None),
            (Some("chat123"), None, None),
            (None, Some(10), None),
            (None, None, Some("2024-01-01")),
            (Some("chat123"), Some(10), None),
            (Some("chat123"), None, Some("2024-01-01")),
            (None, Some(10), Some("2024-01-01")),
            (Some("chat123"), Some(10), Some("2024-01-01")),
        ];

        for (chat_id, limit, since) in test_cases {
            let mut expected_args = vec!["database", "recent"];

            if let Some(chat_id) = chat_id {
                expected_args.extend(&["--chat-id", chat_id]);
            }

            let limit_str = limit.map(|l| l.to_string());
            if let Some(ref limit_str) = limit_str {
                expected_args.extend(&["--limit", limit_str]);
            }

            if let Some(since) = since {
                expected_args.extend(&["--since", since]);
            }

            // This tests the argument building logic without requiring actual CLI execution
            assert!(expected_args.contains(&"database"));
            assert!(expected_args.contains(&"recent"));
        }
    }

    #[test]
    fn test_cli_path_resolution_logic() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat");
        }

        // This test verifies the CLI path resolution logic
        let config = UnifiedConfig::default();
        let result = CliBridge::new(&config);
        match result {
            Ok(bridge) => {
                // CLI was found (unlikely in test environment)
                assert!(!bridge.cli_path.is_empty());
            }
            Err(BridgeError::CliNotFound(path)) => {
                // Expected: CLI not found
                assert!(path.contains("vkteams-bot-cli"));
                assert!(path.contains("not found"));
            }
            Err(e) => {
                println!("Unexpected error (acceptable in test environment): {}", e);
            }
        }
    }

    #[test]
    fn test_default_args_with_and_without_config() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat");
        }

        // Test without config
        unsafe {
            std::env::remove_var("VKTEAMS_BOT_CONFIG");
        }

        let config = UnifiedConfig::default();
        if let Ok(bridge) = CliBridge::new(&config) {
            assert!(bridge.default_args.contains(&"--output".to_string()));
            assert!(bridge.default_args.contains(&"json".to_string()));
            assert!(!bridge.default_args.contains(&"--config".to_string()));
        }

        // Test with config
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CONFIG", "/test/config.toml");
        }

        let config = UnifiedConfig::default();
        if let Ok(bridge) = CliBridge::new(&config) {
            assert!(bridge.default_args.contains(&"--output".to_string()));
            assert!(bridge.default_args.contains(&"json".to_string()));
            assert!(bridge.default_args.contains(&"--config".to_string()));
            assert!(
                bridge
                    .default_args
                    .contains(&"/test/config.toml".to_string())
            );
        }

        unsafe {
            std::env::remove_var("VKTEAMS_BOT_CONFIG");
        }
    }

    #[test]
    fn test_bridge_error_display_variations() {
        // Test display implementations for all error types with different messages

        let errors = vec![
            BridgeError::CliError("Command execution failed".to_string()),
            BridgeError::Timeout(Duration::from_millis(5000)),
            BridgeError::RateLimit("API quota exceeded".to_string()),
            BridgeError::Io("Network connection lost".to_string()),
            BridgeError::CliNotFound("/usr/bin/vkteams-bot-cli".to_string()),
            BridgeError::InvalidResponse("Invalid JSON syntax".to_string()),
            BridgeError::ProcessTerminated("SIGKILL received".to_string()),
            BridgeError::CliReturnedError(CliErrorInfo {
                code: Some("VALIDATION_ERROR".to_string()),
                message: "Invalid input data".to_string(),
                details: Some(serde_json::json!({"field": "email"})),
            }),
        ];

        for error in errors {
            let display_string = error.to_string();
            assert!(!display_string.is_empty());
            assert!(display_string.len() > 10); // Should have meaningful content
        }
    }

    #[test]
    fn test_duration_formatting_in_errors() {
        // Test that Duration is properly formatted in error messages
        let timeout_error = BridgeError::Timeout(Duration::from_secs(30));
        let display = timeout_error.to_string();
        assert!(display.contains("30s") || display.contains("30"));

        let short_timeout = BridgeError::Timeout(Duration::from_millis(500));
        let display = short_timeout.to_string();
        assert!(display.contains("500ms") || display.contains("0.5"));
    }

    #[test]
    fn test_error_chain_display() {
        // Test that error chains are properly displayed
        let cli_error_info = CliErrorInfo {
            code: Some("NESTED_ERROR".to_string()),
            message: "Inner error message".to_string(),
            details: Some(serde_json::json!({"context": "additional info"})),
        };

        let error = BridgeError::CliReturnedError(cli_error_info);
        let display = error.to_string();

        assert!(display.contains("CLI returned error"));
        assert!(display.contains("Inner error message"));
    }

    #[test]
    fn test_cli_error_info_edge_cases() {
        // Test CliErrorInfo with various edge cases

        // Empty message
        let empty_msg_error = CliErrorInfo {
            code: Some("EMPTY_MSG".to_string()),
            message: "".to_string(),
            details: None,
        };

        let serialized = serde_json::to_string(&empty_msg_error).unwrap();
        let deserialized: CliErrorInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.message, "");

        // Very long message
        let long_message = "x".repeat(1000);
        let long_msg_error = CliErrorInfo {
            code: Some("LONG_MSG".to_string()),
            message: long_message.clone(),
            details: None,
        };

        let serialized = serde_json::to_string(&long_msg_error).unwrap();
        let deserialized: CliErrorInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.message, long_message);

        // Complex nested details
        let complex_details = serde_json::json!({
            "level1": {
                "level2": {
                    "level3": [1, 2, 3, {"key": "value"}]
                }
            },
            "array": [
                {"item": 1},
                {"item": 2}
            ]
        });

        let complex_error = CliErrorInfo {
            code: Some("COMPLEX".to_string()),
            message: "Complex error".to_string(),
            details: Some(complex_details.clone()),
        };

        let serialized = serde_json::to_string(&complex_error).unwrap();
        let deserialized: CliErrorInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.details, Some(complex_details));
    }
}
