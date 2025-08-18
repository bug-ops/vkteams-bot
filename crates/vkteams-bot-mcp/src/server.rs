//! New MCP Server implementation using CLI Bridge
//!
//! This module contains the new MCP server implementation that uses the CLI
//! bridge instead of direct library calls. This ensures single source of truth
//! for all business logic in the CLI.

use crate::cli_bridge::CliBridge;
use crate::errors::BridgeError;
use crate::types::{
    ChatInfoParams, DeleteMessageParams, EditMessageParams, EventsGetParams, FileInfoParams,
    GetChatAdminsParams, GetChatMembersParams, GetContextParams, GetDatabaseStatsParams,
    GetProfileParams, GetRecentMessagesParams, PinMessageParams, SearchSemanticParams,
    SearchTextParams, SendActionParams, SendFileParams, SendTextParams, SendVoiceParams, Server,
    SetChatAboutParams, SetChatTitleParams, UnpinMessageParams, UploadFileFromBase64Params,
    UploadJsonFileParams, UploadTextAsFileParams,
};
use rmcp::{
    ServerHandler,
    handler::server::tool::Parameters,
    model::{CallToolResult, Content, ErrorCode, ErrorData, ServerInfo},
    tool, tool_handler, tool_router,
};
use serde_json::Value;
use std::result::Result;
use std::sync::Arc;
use tracing::{error, warn};
use vkteams_bot::config::UnifiedConfig;

pub type MCPResult = Result<CallToolResult, ErrorData>;

/// Convert CLI bridge result to MCP result with enhanced error handling
#[inline]
#[allow(dead_code)] // Used in server method implementations via direct calls
fn convert_bridge_result(result: Result<Value, BridgeError>) -> MCPResult {
    match result {
        Ok(json_response) => {
            // CLI already returns structured JSON, just pass it through
            let json_string = match serde_json::to_string(&json_response) {
                Ok(json) => json,
                Err(e) => {
                    error!("Failed to serialize JSON response: {e}");
                    warn!("Falling back to empty JSON object for response");
                    "{}".to_string()
                }
            };
            Ok(CallToolResult::success(vec![Content::text(json_string)]))
        }
        Err(e) => {
            // Map error types to appropriate error codes
            let (code, message, data) = match &e {
                BridgeError::RateLimit(msg) => {
                    warn!("Rate limit error: {}", msg);
                    (-429, format!("Rate limit exceeded: {msg}"), None)
                }
                BridgeError::Timeout(duration) => {
                    error!("Command timed out after {:?}", duration);
                    (-504, format!("Command timed out after {duration:?}"), None)
                }
                BridgeError::CliReturnedError(info) => {
                    error!("CLI returned error: {:?}", info);
                    let code = match info.code.as_deref() {
                        Some("NOT_FOUND") => -404,
                        Some("UNAUTHORIZED") => -401,
                        Some("FORBIDDEN") => -403,
                        Some("INVALID_INPUT") => -400,
                        Some("RATE_LIMIT") => -429,
                        _ => -500,
                    };
                    (code, info.message.clone(), info.details.clone())
                }
                BridgeError::CliNotFound(path) => {
                    error!("CLI not found at: {}", path);
                    (
                        -503,
                        format!("Service unavailable: CLI not found at {path}"),
                        None,
                    )
                }
                BridgeError::InvalidResponse(err) => {
                    error!("Invalid JSON response: {}", err);
                    (-502, format!("Invalid response from CLI: {err}"), None)
                }
                _ => {
                    error!("CLI bridge error: {}", e);
                    (-500, format!("Internal error: {e}"), None)
                }
            };

            Err(ErrorData {
                code: ErrorCode(code),
                message: message.into(),
                data,
            })
        }
    }
}

#[tool_handler]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(r#"VKTeams MCP Server — a server for managing a VK Teams bot via MCP (Model Context Protocol).
        
        This server uses CLI-as-backend architecture for unified command execution.
        
        Tools:
            - Send text messages to chat (send_text)
            - Get bot information (self_get)
            - Get chat information (chat_info)
            - Get file information (file_info)
            - Get events (events_get)
            - Send files and voice messages (send_file, send_voice)
            - Edit, delete, pin, and unpin messages (edit_message, delete_message, pin_message, unpin_message)
            - Get chat members and admins (get_chat_members, get_chat_admins)
            - Set chat title and description (set_chat_title, set_chat_about)
            - Send chat actions (typing/looking) (send_action)
            - Enhanced file uploads (upload_file_from_base64, upload_text_as_file, upload_json_file)
            - Storage operations (search_semantic, search_text, get_database_stats, get_context)
            "#.into()),
            ..Default::default()
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl Server {
    pub fn bridge(&self) -> Arc<CliBridge> {
        Arc::clone(&self.cli)
    }
    /// Create a new Server instance with unified configuration
    pub fn new() -> Self {
        let mut config = Self::load_config();
        config.apply_env_overrides();

        let cli = CliBridge::new(&config).expect("Failed to create CLI bridge");

        Self {
            cli: Arc::new(cli),
            config,
            tool_router: Self::tool_router(),
        }
    }

    /// Try to create a new Server instance with error handling
    pub fn try_new() -> Result<Self, BridgeError> {
        let mut config = Self::load_config();
        config.apply_env_overrides();

        let cli = CliBridge::new(&config)?;

        Ok(Self {
            cli: Arc::new(cli),
            config,
            tool_router: Self::tool_router(),
        })
    }

    /// Create Server with custom configuration
    pub fn with_config(mut config: UnifiedConfig) -> Self {
        config.apply_env_overrides();

        let cli = CliBridge::new(&config).expect("Failed to create CLI bridge");

        Self {
            cli: Arc::new(cli),
            config,
            tool_router: Self::tool_router(),
        }
    }

    /// Try to create Server with custom configuration
    pub fn try_with_config(mut config: UnifiedConfig) -> Result<Self, BridgeError> {
        config.apply_env_overrides();

        let cli = CliBridge::new(&config)?;

        Ok(Self {
            cli: Arc::new(cli),
            config,
            tool_router: Self::tool_router(),
        })
    }

    /// Load configuration from file or use defaults
    fn load_config() -> UnifiedConfig {
        // Try environment variable first (highest priority)
        if let Ok(config_path) = std::env::var("VKTEAMS_BOT_CONFIG") {
            match UnifiedConfig::load_from_file(&config_path) {
                Ok(config) => {
                    eprintln!("✓ Loaded config from VKTEAMS_BOT_CONFIG: {config_path}");
                    return config;
                }
                Err(e) => {
                    eprintln!(
                        "⚠ Failed to load config from VKTEAMS_BOT_CONFIG ({config_path}): {e} - trying fallback locations"
                    );
                }
            }
        }

        // Try to load from standard locations
        let config_paths = [
            "config.toml",
            "shared-config.toml",
            "/etc/vkteams-bot/config.toml",
        ];

        // Try static paths
        for path in &config_paths {
            match UnifiedConfig::load_from_file(path) {
                Ok(config) => {
                    eprintln!("✓ Loaded config from: {path}");
                    return config;
                }
                Err(_) => {
                    // Silent continue for expected missing files
                }
            }
        }

        // Try user config directory
        if let Some(home_dir) = dirs::home_dir() {
            let user_home_path = home_dir.join(".config/vkteams-bot/config.toml");
            match UnifiedConfig::load_from_file(&user_home_path) {
                Ok(config) => {
                    eprintln!(
                        "✓ Loaded config from user directory: {}",
                        user_home_path.display()
                    );
                    return config;
                }
                Err(_) => {
                    // Silent fallback - user config is optional
                }
            }
        }

        eprintln!("ℹ Using default configuration (no config file found in standard locations)");
        // Fall back to default (env overrides will be applied in new/with_config)
        UnifiedConfig::default()
    }

    // === Messaging Commands ===

    #[tool(description = "Send text message to chat")]
    async fn send_text(
        &self,
        Parameters(SendTextParams {
            text,
            chat_id,
            reply_msg_id,
        }): Parameters<SendTextParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(
            self.cli
                .send_text(&text, target_chat_id, reply_msg_id.as_deref())
                .await,
        )
    }

    #[tool(description = "Send file to chat")]
    async fn send_file(
        &self,
        Parameters(SendFileParams {
            file_path,
            chat_id,
            caption,
        }): Parameters<SendFileParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(
            self.cli
                .send_file(&file_path, target_chat_id, caption.as_deref())
                .await,
        )
    }

    #[tool(description = "Send voice message to chat")]
    async fn send_voice(
        &self,
        Parameters(SendVoiceParams { file_path, chat_id }): Parameters<SendVoiceParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(self.cli.send_voice(&file_path, target_chat_id).await)
    }

    #[tool(description = "Edit existing message")]
    async fn edit_message(
        &self,
        Parameters(EditMessageParams {
            message_id,
            new_text,
            chat_id,
        }): Parameters<EditMessageParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(
            self.cli
                .edit_message(&message_id, &new_text, target_chat_id)
                .await,
        )
    }

    #[tool(description = "Delete message from chat")]
    async fn delete_message(
        &self,
        Parameters(DeleteMessageParams {
            message_id,
            chat_id,
        }): Parameters<DeleteMessageParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(self.cli.delete_message(&message_id, target_chat_id).await)
    }

    #[tool(description = "Pin message in chat")]
    async fn pin_message(
        &self,
        Parameters(PinMessageParams {
            message_id,
            chat_id,
        }): Parameters<PinMessageParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(self.cli.pin_message(&message_id, target_chat_id).await)
    }

    #[tool(description = "Unpin message from chat")]
    async fn unpin_message(
        &self,
        Parameters(UnpinMessageParams {
            message_id,
            chat_id,
        }): Parameters<UnpinMessageParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(self.cli.unpin_message(&message_id, target_chat_id).await)
    }

    // === Chat Management Commands ===

    #[tool(description = "Get information about the chat")]
    async fn chat_info(
        &self,
        Parameters(ChatInfoParams { chat_id }): Parameters<ChatInfoParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(self.cli.get_chat_info(target_chat_id).await)
    }

    #[tool(description = "Get user profile information")]
    async fn get_profile(
        &self,
        Parameters(GetProfileParams { user_id }): Parameters<GetProfileParams>,
    ) -> MCPResult {
        convert_bridge_result(self.cli.get_profile(&user_id).await)
    }

    #[tool(description = "Get chat members")]
    async fn get_chat_members(
        &self,
        Parameters(GetChatMembersParams { chat_id, cursor }): Parameters<GetChatMembersParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(
            self.cli
                .get_chat_members(target_chat_id, cursor.as_deref())
                .await,
        )
    }

    #[tool(description = "Get chat administrators")]
    async fn get_chat_admins(
        &self,
        Parameters(GetChatAdminsParams { chat_id }): Parameters<GetChatAdminsParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(self.cli.get_chat_admins(target_chat_id).await)
    }

    #[tool(description = "Set chat title")]
    async fn set_chat_title(
        &self,
        Parameters(SetChatTitleParams { title, chat_id }): Parameters<SetChatTitleParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(self.cli.set_chat_title(&title, target_chat_id).await)
    }

    #[tool(description = "Set chat description")]
    async fn set_chat_about(
        &self,
        Parameters(SetChatAboutParams { about, chat_id }): Parameters<SetChatAboutParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(self.cli.set_chat_about(&about, target_chat_id).await)
    }

    #[tool(description = "Send typing or looking action to chat")]
    async fn send_action(
        &self,
        Parameters(SendActionParams { action, chat_id }): Parameters<SendActionParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(self.cli.send_action(&action, target_chat_id).await)
    }

    // === File Upload Commands ===

    #[tool(description = "Upload file from base64 content")]
    async fn upload_file_from_base64(
        &self,
        Parameters(UploadFileFromBase64Params {
            file_name,
            base64_content,
            chat_id,
            caption,
            reply_msg_id,
        }): Parameters<UploadFileFromBase64Params>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(
            self.cli
                .upload_file_base64(
                    &file_name,
                    &base64_content,
                    target_chat_id,
                    caption.as_deref(),
                    reply_msg_id.as_deref(),
                )
                .await,
        )
    }

    #[tool(description = "Upload text content as file")]
    async fn upload_text_as_file(
        &self,
        Parameters(UploadTextAsFileParams {
            file_name,
            content,
            chat_id,
            caption,
        }): Parameters<UploadTextAsFileParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(
            self.cli
                .upload_text_file(&file_name, &content, target_chat_id, caption.as_deref())
                .await,
        )
    }

    #[tool(description = "Upload JSON data as file")]
    async fn upload_json_file(
        &self,
        Parameters(UploadJsonFileParams {
            file_name,
            json_data,
            pretty,
            chat_id,
            caption,
        }): Parameters<UploadJsonFileParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(
            self.cli
                .upload_json_file(
                    &file_name,
                    &json_data,
                    pretty.unwrap_or(true),
                    target_chat_id,
                    caption.as_deref(),
                )
                .await,
        )
    }

    #[tool(description = "Get file information")]
    async fn file_info(
        &self,
        Parameters(FileInfoParams { file_id }): Parameters<FileInfoParams>,
    ) -> MCPResult {
        convert_bridge_result(self.cli.get_file_info(&file_id).await)
    }

    // === Storage Commands ===

    #[tool(description = "Search messages using semantic similarity")]
    async fn search_semantic(
        &self,
        Parameters(SearchSemanticParams {
            query,
            chat_id,
            limit,
        }): Parameters<SearchSemanticParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(
            self.cli
                .search_semantic(&query, target_chat_id, limit)
                .await,
        )
    }

    #[tool(description = "Search messages using text search")]
    async fn search_text(
        &self,
        Parameters(SearchTextParams {
            query,
            chat_id,
            limit,
        }): Parameters<SearchTextParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(self.cli.search_text(&query, target_chat_id, limit).await)
    }

    #[tool(description = "Get database statistics")]
    async fn get_database_stats(
        &self,
        Parameters(GetDatabaseStatsParams { chat_id, since }): Parameters<GetDatabaseStatsParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(
            self.cli
                .get_database_stats(target_chat_id, since.as_deref())
                .await,
        )
    }

    #[tool(description = "Get conversation context")]
    async fn get_context(
        &self,
        Parameters(GetContextParams {
            chat_id,
            context_type,
            timeframe,
        }): Parameters<GetContextParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(
            self.cli
                .get_context(
                    target_chat_id,
                    context_type.as_deref(),
                    timeframe.as_deref(),
                )
                .await,
        )
    }

    // === Daemon Management Commands ===

    #[tool(description = "Get daemon status and statistics")]
    async fn daemon_status(&self) -> MCPResult {
        convert_bridge_result(self.cli.get_daemon_status().await)
    }

    #[tool(description = "Get recent messages from storage")]
    async fn get_recent_messages(
        &self,
        Parameters(GetRecentMessagesParams {
            chat_id,
            limit,
            since,
        }): Parameters<GetRecentMessagesParams>,
    ) -> MCPResult {
        let target_chat_id = chat_id.as_deref().or(self.config.mcp.chat_id.as_deref());
        convert_bridge_result(
            self.cli
                .get_recent_messages(target_chat_id, limit, since.as_deref())
                .await,
        )
    }

    // === Diagnostic Commands ===

    #[tool(description = "Get information about the bot")]
    async fn self_get(&self) -> MCPResult {
        convert_bridge_result(self.cli.get_self().await)
    }

    #[tool(description = "Get events from the bot")]
    async fn events_get(
        &self,
        Parameters(EventsGetParams {
            last_event_id,
            poll_time,
        }): Parameters<EventsGetParams>,
    ) -> MCPResult {
        convert_bridge_result(
            self.cli
                .get_events(last_event_id.as_deref(), poll_time)
                .await,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_bridge::{CliBridgeTrait, MockCliBridge};
    use crate::errors::CliErrorInfo;
    use std::time::Duration;

    /// Helper function to create a mock CLI bridge with common responses
    fn create_mock_with_responses() -> MockCliBridge {
        let mut mock = MockCliBridge::new();

        // Add common success responses
        mock.add_success_response(
            "self-get".to_string(),
            serde_json::json!({"userId": "bot123", "firstName": "TestBot"}),
        );
        mock.add_success_response(
            "chat-info".to_string(),
            serde_json::json!({"chatId": "chat123", "title": "Test Chat"}),
        );
        mock.add_success_response(
            "send-text".to_string(),
            serde_json::json!({"msgId": "msg123", "ok": true}),
        );

        mock
    }

    #[test]
    fn test_convert_bridge_result_success() {
        let json_val = serde_json::json!({"success": true, "data": "test"});
        let result = convert_bridge_result(Ok(json_val));
        assert!(result.is_ok());

        if let Ok(call_result) = result {
            assert!(!call_result.content.expect("content").is_empty());
        }
    }

    #[test]
    fn test_convert_bridge_result_error() {
        let error = BridgeError::CliError("test error".to_string());
        let result = convert_bridge_result(Err(error));
        assert!(result.is_err());

        if let Err(error_data) = result {
            assert_eq!(error_data.code.0, -500);
            assert!(error_data.message.contains("test error"));
        }
    }

    #[test]
    fn test_convert_bridge_result_rate_limit() {
        let error = BridgeError::RateLimit("too many requests".to_string());
        let result = convert_bridge_result(Err(error));
        assert!(result.is_err());

        if let Err(error_data) = result {
            assert_eq!(error_data.code.0, -429);
            assert!(error_data.message.contains("Rate limit exceeded"));
        }
    }

    #[test]
    fn test_convert_bridge_result_timeout() {
        let error = BridgeError::Timeout(Duration::from_secs(30));
        let result = convert_bridge_result(Err(error));
        assert!(result.is_err());

        if let Err(error_data) = result {
            assert_eq!(error_data.code.0, -504);
            assert!(error_data.message.contains("timed out"));
        }
    }

    #[test]
    fn test_convert_bridge_result_cli_returned_error() {
        let cli_error = CliErrorInfo {
            code: Some("NOT_FOUND".to_string()),
            message: "Resource not found".to_string(),
            details: Some(serde_json::json!({"resource_id": "123"})),
        };
        let error = BridgeError::CliReturnedError(cli_error);
        let result = convert_bridge_result(Err(error));
        assert!(result.is_err());

        if let Err(error_data) = result {
            assert_eq!(error_data.code.0, -404);
            assert!(error_data.message.contains("Resource not found"));
            assert!(error_data.data.is_some());
        }
    }

    #[test]
    fn test_convert_bridge_result_cli_not_found() {
        let error = BridgeError::CliNotFound("/path/to/cli".to_string());
        let result = convert_bridge_result(Err(error));
        assert!(result.is_err());

        if let Err(error_data) = result {
            assert_eq!(error_data.code.0, -503);
            assert!(error_data.message.contains("Service unavailable"));
        }
    }

    #[test]
    fn test_convert_bridge_result_invalid_response() {
        let error = BridgeError::InvalidResponse("malformed json".to_string());
        let result = convert_bridge_result(Err(error));
        assert!(result.is_err());

        if let Err(error_data) = result {
            assert_eq!(error_data.code.0, -502);
            assert!(error_data.message.contains("Invalid response"));
        }
    }

    #[tokio::test]
    async fn test_mock_cli_bridge_basic() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "self-get".to_string(),
            serde_json::json!({"userId": "123", "firstName": "Bot"}),
        );

        let result = mock.execute_command(&["self-get"]).await;
        assert!(result.is_ok());

        if let Ok(response) = result {
            assert_eq!(response["success"], true);
            assert_eq!(response["data"]["userId"], "123");
        }
    }

    #[tokio::test]
    async fn test_mock_cli_bridge_error() {
        let mut mock = MockCliBridge::new();
        mock.add_error_response("send-text".to_string(), "Message too long".to_string());

        let result = mock
            .execute_command(&["send-text", "very long message..."])
            .await;
        assert!(result.is_err());

        if let Err(BridgeError::CliReturnedError(info)) = result {
            assert_eq!(info.message, "Message too long");
        }
    }

    #[tokio::test]
    async fn test_mock_cli_bridge_partial_match() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "chat-info".to_string(),
            serde_json::json!({"chatId": "chat123", "title": "Test Chat"}),
        );

        let result = mock
            .execute_command(&["chat-info", "--chat-id", "chat123"])
            .await;
        assert!(result.is_ok());

        if let Ok(response) = result {
            assert_eq!(response["success"], true);
            assert_eq!(response["data"]["chatId"], "chat123");
        }
    }

    #[tokio::test]
    async fn test_mock_cli_bridge_default_response() {
        let mock = MockCliBridge::new();
        let result = mock.execute_command(&["unknown-command"]).await;
        assert!(result.is_ok());

        if let Ok(response) = result {
            assert_eq!(response["success"], true);
            assert_eq!(response["command"], "unknown-command");
        }
    }

    #[tokio::test]
    async fn test_mock_health_check() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "--version".to_string(),
            serde_json::json!({"version": "1.0.0"}),
        );

        let result = mock.health_check().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_daemon_status() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "daemon status".to_string(),
            serde_json::json!({"status": "running", "uptime": 3600}),
        );

        let result = mock.get_daemon_status().await;
        assert!(result.is_ok());

        if let Ok(response) = result {
            assert_eq!(response["success"], true);
            assert_eq!(response["data"]["status"], "running");
        }
    }

    #[test]
    fn test_cli_error_info_creation() {
        let info = CliErrorInfo {
            code: Some("INVALID_INPUT".to_string()),
            message: "Invalid parameter".to_string(),
            details: Some(serde_json::json!({"field": "message_id"})),
        };

        assert_eq!(info.code.as_ref(), Some(&"INVALID_INPUT".to_string()));
        assert_eq!(info.message, "Invalid parameter");
        assert!(info.details.is_some());
    }

    #[test]
    fn test_error_code_mapping() {
        let test_cases = vec![
            ("NOT_FOUND", -404),
            ("UNAUTHORIZED", -401),
            ("FORBIDDEN", -403),
            ("INVALID_INPUT", -400),
            ("RATE_LIMIT", -429),
            ("UNKNOWN", -500),
        ];

        for (code, expected_code) in test_cases {
            let cli_error = CliErrorInfo {
                code: Some(code.to_string()),
                message: "test".to_string(),
                details: None,
            };
            let error = BridgeError::CliReturnedError(cli_error);
            let result = convert_bridge_result(Err(error));

            if let Err(error_data) = result {
                assert_eq!(error_data.code.0, expected_code, "Failed for code: {code}");
            }
        }
    }

    #[test]
    fn test_server_info_structure() {
        // Test server info generation without requiring CLI bridge
        use rmcp::model::{ServerCapabilities, ServerInfo};

        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_prompts()
            .build();

        let info = ServerInfo {
            capabilities,
            instructions: Some(r#"VKTeams MCP Server — a server for managing a VK Teams bot via MCP (Model Context Protocol).
        
        This server uses CLI-as-backend architecture for unified command execution.
        
        Tools:
            - Send text messages to chat (send_text)
            - Get bot information (self_get)
            - Get chat information (chat_info)
            - Get file information (file_info)
            - Get events (events_get)
            - Send files and voice messages (send_file, send_voice)
            - Edit, delete, pin, and unpin messages (edit_message, delete_message, pin_message, unpin_message)
            - Get chat members and admins (get_chat_members, get_chat_admins)
            - Set chat title and description (set_chat_title, set_chat_about)
            - Send chat actions (typing/looking) (send_action)
            - Enhanced file uploads (upload_file_from_base64, upload_text_as_file, upload_json_file)
            - Storage operations (search_semantic, search_text, get_database_stats, get_context)
            "#.into()),
            ..Default::default()
        };

        assert!(info.capabilities.tools.is_some());
        assert!(info.capabilities.prompts.is_some());
        assert!(info.instructions.is_some());

        let instructions = info.instructions.unwrap();
        assert!(instructions.contains("VKTeams MCP Server"));
        assert!(instructions.contains("send_text"));
        assert!(instructions.contains("chat_info"));
    }

    // Test individual mock CLI bridge commands functionality
    #[tokio::test]
    async fn test_mock_messaging_commands() {
        let mut mock = create_mock_with_responses();

        // Test send text
        let result = mock
            .execute_command(&["send-text", "Hello!", "--chat-id", "chat123"])
            .await;
        assert!(result.is_ok());

        // Test send file
        mock.add_success_response(
            "send-file".to_string(),
            serde_json::json!({"fileId": "file123", "ok": true}),
        );
        let result = mock
            .execute_command(&["send-file", "/path/file.txt", "--chat-id", "chat123"])
            .await;
        assert!(result.is_ok());

        // Test edit message
        mock.add_success_response("edit-message".to_string(), serde_json::json!({"ok": true}));
        let result = mock
            .execute_command(&["edit-message", "msg123", "--new-text", "Updated"])
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_chat_management_commands() {
        let mut mock = create_mock_with_responses();

        // Test chat info
        let result = mock
            .execute_command(&["chat-info", "--chat-id", "chat123"])
            .await;
        assert!(result.is_ok());

        // Test get chat members
        mock.add_success_response(
            "get-chat-members".to_string(),
            serde_json::json!({"members": [], "cursor": null}),
        );
        let result = mock
            .execute_command(&["get-chat-members", "--chat-id", "chat123"])
            .await;
        assert!(result.is_ok());

        // Test set chat title
        mock.add_success_response(
            "set-chat-title".to_string(),
            serde_json::json!({"ok": true}),
        );
        let result = mock
            .execute_command(&["set-chat-title", "New Title", "--chat-id", "chat123"])
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_file_operations() {
        let mut mock = create_mock_with_responses();

        // Test upload base64
        mock.add_success_response(
            "upload-file-base64".to_string(),
            serde_json::json!({"fileId": "file123", "ok": true}),
        );
        let result = mock
            .execute_command(&[
                "upload-file-base64",
                "test.txt",
                "--content",
                "dGVzdA==", // "test" in base64
                "--chat-id",
                "chat123",
            ])
            .await;
        assert!(result.is_ok());

        // Test file info
        mock.add_success_response(
            "file-info".to_string(),
            serde_json::json!({"fileId": "file123", "fileName": "test.txt"}),
        );
        let result = mock.execute_command(&["file-info", "file123"]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_storage_operations() {
        let mut mock = create_mock_with_responses();

        // Test semantic search
        mock.add_success_response(
            "search-semantic".to_string(),
            serde_json::json!({"results": [], "total": 0}),
        );
        let result = mock
            .execute_command(&[
                "search-semantic",
                "query",
                "--chat-id",
                "chat123",
                "--limit",
                "10",
            ])
            .await;
        assert!(result.is_ok());

        // Test database stats
        mock.add_success_response(
            "get-database-stats".to_string(),
            serde_json::json!({"totalMessages": 1000, "totalChats": 5}),
        );
        let result = mock
            .execute_command(&["get-database-stats", "--chat-id", "chat123"])
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_diagnostic_operations() {
        let mut mock = create_mock_with_responses();

        // Test self get
        let result = mock.execute_command(&["self-get"]).await;
        assert!(result.is_ok());

        // Test events get
        mock.add_success_response(
            "events-get".to_string(),
            serde_json::json!({"events": [], "lastEventId": "123"}),
        );
        let result = mock
            .execute_command(&["events-get", "--last-event-id", "last123"])
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_error_scenarios() {
        let mut mock = MockCliBridge::new();

        // Test various error types
        mock.add_error_response(
            "invalid-command".to_string(),
            "Command not found".to_string(),
        );
        let result = mock.execute_command(&["invalid-command"]).await;
        assert!(result.is_err());

        // Test rate limit error handling
        mock.add_error_response("rate-limited".to_string(), "Too many requests".to_string());
        let result = mock.execute_command(&["rate-limited"]).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_result_type_operations() {
        // Test MCPResult type operations for better coverage
        use crate::server::MCPResult;
        use rmcp::model::{CallToolResult, Content, ErrorCode, ErrorData};

        // Test success case
        let success_result: MCPResult = Ok(CallToolResult::success(vec![Content::text(
            "test content".to_string(),
        )]));
        assert!(success_result.is_ok());

        if let Ok(result) = success_result {
            assert!(!result.content.expect("content").is_empty());
            // Just verify that content is not empty, without pattern matching on enum variants
            // since Content enum structure may vary between rmcp versions
        }

        // Test error case
        let error_result: MCPResult = Err(ErrorData {
            code: ErrorCode(-400),
            message: "Test error".into(),
            data: Some(serde_json::json!({"detail": "error details"})),
        });
        assert!(error_result.is_err());

        if let Err(error) = error_result {
            assert_eq!(error.code.0, -400);
            assert_eq!(error.message, "Test error");
            assert!(error.data.is_some());
        }
    }

    #[test]
    fn test_convert_bridge_result_edge_cases() {
        // Test empty JSON object
        let empty_json = serde_json::json!({});
        let result = convert_bridge_result(Ok(empty_json));
        assert!(result.is_ok());

        // Test null JSON
        let null_json = serde_json::Value::Null;
        let result = convert_bridge_result(Ok(null_json));
        assert!(result.is_ok());

        // Test array JSON
        let array_json = serde_json::json!([1, 2, 3]);
        let result = convert_bridge_result(Ok(array_json));
        assert!(result.is_ok());

        // Test string JSON
        let string_json = serde_json::json!("simple string");
        let result = convert_bridge_result(Ok(string_json));
        assert!(result.is_ok());
    }

    #[test]
    fn test_bridge_error_types_coverage() {
        // Test ProcessTerminated error
        let error = BridgeError::ProcessTerminated("Signal 9".to_string());
        let result = convert_bridge_result(Err(error));
        assert!(result.is_err());

        if let Err(error_data) = result {
            assert_eq!(error_data.code.0, -500);
            assert!(error_data.message.contains("Internal error"));
        }

        // Test IO error
        let error = BridgeError::Io("Connection failed".to_string());
        let result = convert_bridge_result(Err(error));
        assert!(result.is_err());

        if let Err(error_data) = result {
            assert_eq!(error_data.code.0, -500);
            assert!(error_data.message.contains("Internal error"));
        }
    }

    #[test]
    fn test_server_info_implementation() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_server_info");
        }

        match Server::try_new() {
            Ok(server) => {
                use rmcp::ServerHandler;
                let info = server.get_info();

                // Test server capabilities
                assert!(info.capabilities.tools.is_some());
                // Prompts are currently disabled in server info
                // assert!(info.capabilities.prompts.is_some());

                // Test instructions content
                assert!(info.instructions.is_some());
                let instructions = info.instructions.unwrap();
                assert!(instructions.contains("VKTeams MCP Server"));
                assert!(instructions.contains("send_text"));
                assert!(instructions.contains("chat_info"));
                assert!(instructions.contains("file_info"));
                assert!(instructions.contains("CLI-as-backend architecture"));
            }
            Err(_) => {
                println!("Server info test skipped - CLI binary not available");
            }
        }

        unsafe {
            std::env::remove_var("VKTEAMS_BOT_CHAT_ID");
        }
    }

    #[test]
    fn test_chat_id_fallback_logic() {
        // Test that target_chat_id uses config fallback when chat_id is None
        let config_chat_id = Some("config_chat_123".to_string());
        let param_chat_id: Option<String> = None;

        let target_chat_id = param_chat_id.as_deref().or(config_chat_id.as_deref());
        assert_eq!(target_chat_id, Some("config_chat_123"));

        // Test that parameter chat_id takes precedence over config
        let param_chat_id = Some("param_chat_456".to_string());
        let target_chat_id = param_chat_id.as_deref().or(config_chat_id.as_deref());
        assert_eq!(target_chat_id, Some("param_chat_456"));

        // Test when both are None
        let param_chat_id: Option<String> = None;
        let config_chat_id: Option<String> = None;
        let target_chat_id = param_chat_id.as_deref().or(config_chat_id.as_deref());
        assert_eq!(target_chat_id, None);
    }

    #[test]
    fn test_various_json_response_conversions() {
        // Test different types of JSON responses that would come from CLI commands

        // Test messaging response
        let msg_response = serde_json::json!({"msgId": "msg123", "ok": true});
        let result = convert_bridge_result(Ok(msg_response));
        assert!(result.is_ok());

        // Test file upload response
        let file_response =
            serde_json::json!({"fileId": "file123", "fileName": "test.txt", "size": 1024});
        let result = convert_bridge_result(Ok(file_response));
        assert!(result.is_ok());

        // Test chat info response
        let chat_response = serde_json::json!({
            "chatId": "chat123",
            "title": "Test Chat",
            "members": 5,
            "type": "group"
        });
        let result = convert_bridge_result(Ok(chat_response));
        assert!(result.is_ok());

        // Test search response
        let search_response = serde_json::json!({
            "results": [
                {"id": "1", "text": "message 1"},
                {"id": "2", "text": "message 2"}
            ],
            "total": 2,
            "hasMore": false
        });
        let result = convert_bridge_result(Ok(search_response));
        assert!(result.is_ok());

        // Test daemon status response
        let daemon_response = serde_json::json!({
            "status": "running",
            "uptime": 3600,
            "processId": 12345,
            "memory": "50MB"
        });
        let result = convert_bridge_result(Ok(daemon_response));
        assert!(result.is_ok());

        // Test events response
        let events_response = serde_json::json!({
            "events": [
                {"type": "message", "id": "event1"},
                {"type": "file", "id": "event2"}
            ],
            "lastEventId": "event2",
            "hasMore": true
        });
        let result = convert_bridge_result(Ok(events_response));
        assert!(result.is_ok());

        // Verify all results have content
        for response in [
            serde_json::json!({"test": "value"}),
            serde_json::json!([1, 2, 3]),
            serde_json::json!("simple string"),
            serde_json::Value::Null,
            serde_json::json!({"complex": {"nested": {"deep": true}}}),
        ] {
            let result = convert_bridge_result(Ok(response));
            assert!(result.is_ok());

            if let Ok(call_result) = result {
                assert!(!call_result.content.expect("Expected content").is_empty());
            }
        }
    }

    #[test]
    fn test_json_serialization_fallback() {
        // Test the JSON serialization fallback path that was improved in PR #49
        // Create a Value that should serialize successfully
        let normal_json = serde_json::json!({"test": "normal"});
        let result = convert_bridge_result(Ok(normal_json));
        assert!(result.is_ok());

        // Test with complex JSON structure
        let complex_json = serde_json::json!({
            "nested": {
                "array": [1, 2, 3],
                "string": "test",
                "null": null,
                "bool": true
            }
        });
        let result = convert_bridge_result(Ok(complex_json));
        assert!(result.is_ok());

        if let Ok(call_result) = result {
            assert!(!call_result.content.expect("Expected content").is_empty());
        }
    }

    // Server tests from types.rs
    #[test]
    fn test_server_default_and_bridge() {
        // Set required env vars for default
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat_id");
        }

        // Test server creation with graceful error handling
        match Server::try_new() {
            Ok(server) => {
                println!("✓ Server created successfully");
                assert_eq!(server.config.mcp.chat_id, Some("test_chat_id".to_string()));
                // Don't assert exact API URL as user might have custom config
                assert!(!server.config.api.url.is_empty());
                let bridge = server.bridge();
                assert!(Arc::strong_count(&bridge) >= 1);
            }
            Err(e) => {
                println!("⚠ Expected failure in test environment without CLI binary: {e}");
                // This is acceptable in test environment where CLI binary might not be available
                assert!(e.to_string().contains("CLI") || e.to_string().contains("bridge"));
            }
        }

        // Clean up environment variable for next test
        unsafe {
            std::env::remove_var("VKTEAMS_BOT_CHAT_ID");
        }
    }

    #[test]
    fn test_server_with_config() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "config_test_chat");
        }

        let mut config = UnifiedConfig::default();
        config.mcp.chat_id = Some("custom_chat_id".to_string());
        config.api.url = "https://custom.api.com".to_string();

        match Server::try_with_config(config.clone()) {
            Ok(server) => {
                println!("✓ Server with config created successfully");
                assert_eq!(
                    server.config.mcp.chat_id,
                    Some("config_test_chat".to_string())
                ); // env override
                assert_eq!(server.config.api.url, "https://custom.api.com");
            }
            Err(e) => {
                println!("⚠ Expected failure in test environment without CLI binary: {e}");
                assert!(e.to_string().contains("CLI") || e.to_string().contains("not found"));
            }
        }

        // Clean up environment variable
        unsafe {
            std::env::remove_var("VKTEAMS_BOT_CHAT_ID");
        }
    }

    #[test]
    fn test_config_loading_scenarios() {
        // Test config loading with environment variable
        let original_config = std::env::var("VKTEAMS_BOT_CONFIG").ok();

        unsafe {
            std::env::set_var("VKTEAMS_BOT_CONFIG", "/nonexistent/config.toml");
        }

        // This should handle the error gracefully and fall back to available config
        let config = Server::load_config();
        assert!(!config.api.url.is_empty()); // Should have some URL

        // Restore original state
        unsafe {
            match original_config {
                Some(config) => std::env::set_var("VKTEAMS_BOT_CONFIG", config),
                None => std::env::remove_var("VKTEAMS_BOT_CONFIG"),
            }
        }
    }

    #[test]
    fn test_user_config_directory_resolution() {
        // Test config loading when home directory is available
        if let Some(home_dir) = dirs::home_dir() {
            let user_config_path = home_dir.join(".config/vkteams-bot/config.toml");
            println!("Testing user config path: {}", user_config_path.display());

            // This tests the path resolution logic
            let config = Server::load_config();
            assert!(!config.api.url.is_empty());
            // Don't assert specific URL as user might have custom config
        }
    }

    #[test]
    fn test_bridge_reference_counting() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_bridge_ref");
        }

        match Server::try_new() {
            Ok(server) => {
                let bridge1 = server.bridge();
                assert!(Arc::strong_count(&bridge1) >= 1);

                // Test multiple references
                let bridge2 = server.bridge();
                assert!(Arc::strong_count(&bridge2) >= 2);

                // Both should point to the same instance
                assert!(Arc::ptr_eq(&bridge1, &bridge2));
            }
            Err(_) => {
                println!("Bridge test skipped - CLI binary not available in test environment");
            }
        }

        // Clean up environment variable
        unsafe {
            std::env::remove_var("VKTEAMS_BOT_CHAT_ID");
        }
    }
}
