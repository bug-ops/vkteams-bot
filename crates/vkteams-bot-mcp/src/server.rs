//! New MCP Server implementation using CLI Bridge
//!
//! This module contains the new MCP server implementation that uses the CLI
//! bridge instead of direct library calls. This ensures single source of truth
//! for all business logic in the CLI.

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
    model::{CallToolResult, Content, ErrorCode, ErrorData, ServerCapabilities, ServerInfo},
    tool, tool_handler,
};
use serde_json::Value;
use std::result::Result;
use tracing::{error, warn};

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
                    (
                        -504,
                        format!("Command timed out after {duration:?}"),
                        None,
                    )
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
        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_prompts()
            .build();
        ServerInfo {
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
        }
    }
}

impl Server {
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
        convert_bridge_result(
            self.cli
                .send_text(&text, chat_id.as_deref(), reply_msg_id.as_deref())
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
        convert_bridge_result(
            self.cli
                .send_file(&file_path, chat_id.as_deref(), caption.as_deref())
                .await,
        )
    }

    #[tool(description = "Send voice message to chat")]
    async fn send_voice(
        &self,
        Parameters(SendVoiceParams { file_path, chat_id }): Parameters<SendVoiceParams>,
    ) -> MCPResult {
        convert_bridge_result(self.cli.send_voice(&file_path, chat_id.as_deref()).await)
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
        convert_bridge_result(
            self.cli
                .edit_message(&message_id, &new_text, chat_id.as_deref())
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
        convert_bridge_result(
            self.cli
                .delete_message(&message_id, chat_id.as_deref())
                .await,
        )
    }

    #[tool(description = "Pin message in chat")]
    async fn pin_message(
        &self,
        Parameters(PinMessageParams {
            message_id,
            chat_id,
        }): Parameters<PinMessageParams>,
    ) -> MCPResult {
        convert_bridge_result(self.cli.pin_message(&message_id, chat_id.as_deref()).await)
    }

    #[tool(description = "Unpin message from chat")]
    async fn unpin_message(
        &self,
        Parameters(UnpinMessageParams {
            message_id,
            chat_id,
        }): Parameters<UnpinMessageParams>,
    ) -> MCPResult {
        convert_bridge_result(
            self.cli
                .unpin_message(&message_id, chat_id.as_deref())
                .await,
        )
    }

    // === Chat Management Commands ===

    #[tool(description = "Get information about the chat")]
    async fn chat_info(
        &self,
        Parameters(ChatInfoParams { chat_id }): Parameters<ChatInfoParams>,
    ) -> MCPResult {
        convert_bridge_result(self.cli.get_chat_info(chat_id.as_deref()).await)
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
        convert_bridge_result(
            self.cli
                .get_chat_members(chat_id.as_deref(), cursor.as_deref())
                .await,
        )
    }

    #[tool(description = "Get chat administrators")]
    async fn get_chat_admins(
        &self,
        Parameters(GetChatAdminsParams { chat_id }): Parameters<GetChatAdminsParams>,
    ) -> MCPResult {
        convert_bridge_result(self.cli.get_chat_admins(chat_id.as_deref()).await)
    }

    #[tool(description = "Set chat title")]
    async fn set_chat_title(
        &self,
        Parameters(SetChatTitleParams { title, chat_id }): Parameters<SetChatTitleParams>,
    ) -> MCPResult {
        convert_bridge_result(self.cli.set_chat_title(&title, chat_id.as_deref()).await)
    }

    #[tool(description = "Set chat description")]
    async fn set_chat_about(
        &self,
        Parameters(SetChatAboutParams { about, chat_id }): Parameters<SetChatAboutParams>,
    ) -> MCPResult {
        convert_bridge_result(self.cli.set_chat_about(&about, chat_id.as_deref()).await)
    }

    #[tool(description = "Send typing or looking action to chat")]
    async fn send_action(
        &self,
        Parameters(SendActionParams { action, chat_id }): Parameters<SendActionParams>,
    ) -> MCPResult {
        convert_bridge_result(self.cli.send_action(&action, chat_id.as_deref()).await)
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
        convert_bridge_result(
            self.cli
                .upload_file_base64(
                    &file_name,
                    &base64_content,
                    chat_id.as_deref(),
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
        convert_bridge_result(
            self.cli
                .upload_text_file(&file_name, &content, chat_id.as_deref(), caption.as_deref())
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
        convert_bridge_result(
            self.cli
                .upload_json_file(
                    &file_name,
                    &json_data,
                    pretty.unwrap_or(true),
                    chat_id.as_deref(),
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
        convert_bridge_result(
            self.cli
                .search_semantic(&query, chat_id.as_deref(), limit)
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
        convert_bridge_result(
            self.cli
                .search_text(&query, chat_id.as_deref(), limit)
                .await,
        )
    }

    #[tool(description = "Get database statistics")]
    async fn get_database_stats(
        &self,
        Parameters(GetDatabaseStatsParams { chat_id, since }): Parameters<GetDatabaseStatsParams>,
    ) -> MCPResult {
        convert_bridge_result(
            self.cli
                .get_database_stats(chat_id.as_deref(), since.as_deref())
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
        convert_bridge_result(
            self.cli
                .get_context(
                    chat_id.as_deref(),
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
        convert_bridge_result(
            self.cli
                .get_recent_messages(chat_id.as_deref(), limit, since.as_deref())
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
    use crate::bridge_trait::{CliBridgeTrait, MockCliBridge};
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
            assert!(!call_result.content.is_empty());
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
                assert_eq!(
                    error_data.code.0, expected_code,
                    "Failed for code: {code}"
                );
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
            assert!(!result.content.is_empty());
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
}
