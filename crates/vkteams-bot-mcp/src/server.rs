//! New MCP Server implementation using CLI Bridge
//!
//! This module contains the new MCP server implementation that uses the CLI
//! bridge instead of direct library calls. This ensures single source of truth
//! for all business logic in the CLI.

use crate::errors::BridgeError;
use crate::mcp_bridge_trait::{McpMessaging, McpChatManagement, McpFileOperations, McpStorage, McpDiagnostics};
use crate::types::Server;
use rmcp::tool_box;
use rmcp::{
    ServerHandler,
    model::{CallToolResult, Content, ErrorCode, ErrorData, ServerCapabilities, ServerInfo},
    tool,
};
use serde_json::Value;
use std::result::Result;
use tracing::{error, warn};

pub type MCPResult = Result<CallToolResult, ErrorData>;

/// Convert CLI bridge result to MCP result with enhanced error handling
/// This function is used internally by McpCliBridge implementation
#[inline]
pub(crate) fn convert_bridge_result(result: Result<Value, BridgeError>) -> MCPResult {
    match result {
        Ok(json_response) => {
            // CLI already returns structured JSON, just pass it through
            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string(&json_response).unwrap_or_else(|_| "{}".to_string()),
            )]))
        }
        Err(e) => {
            // Map error types to appropriate error codes
            let (code, message, data) = match &e {
                BridgeError::RateLimit(msg) => {
                    warn!("Rate limit error: {}", msg);
                    (-429, format!("Rate limit exceeded: {}", msg), None)
                }
                BridgeError::Timeout(duration) => {
                    error!("Command timed out after {:?}", duration);
                    (
                        -504,
                        format!("Command timed out after {:?}", duration),
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
                        format!("Service unavailable: CLI not found at {}", path),
                        None,
                    )
                }
                BridgeError::InvalidResponse(err) => {
                    error!("Invalid JSON response: {}", err);
                    (-502, format!("Invalid response from CLI: {}", err), None)
                }
                _ => {
                    error!("CLI bridge error: {}", e);
                    (-500, format!("Internal error: {}", e), None)
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
        #[tool(param)]
        #[schemars(description = r#"Text message to send.
        You can use HTML formatting:
            <b>bold</b>, <i>italic</i>, <u>underline</u>, <s>strikethrough</s>
            <a href="http://www.example.com/">inline URL</a>
            <code>inline code</code>
            <pre>pre-formatted code block</pre>
        "#)]
        text: String,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Reply to message ID (optional)")]
        reply_msg_id: Option<String>,
    ) -> MCPResult {
        self.cli
            .send_text_mcp(&text, chat_id.as_deref(), reply_msg_id.as_deref())
            .await
    }

    #[tool(description = "Send file to chat")]
    async fn send_file(
        &self,
        #[tool(param)]
        #[schemars(description = "Path to file to send")]
        file_path: String,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Caption for the file (optional)")]
        caption: Option<String>,
    ) -> MCPResult {
        self.cli
            .send_file_mcp(&file_path, chat_id.as_deref(), caption.as_deref())
            .await
    }

    #[tool(description = "Send voice message to chat")]
    async fn send_voice(
        &self,
        #[tool(param)]
        #[schemars(description = "Path to voice file to send (.ogg, .mp3, .wav, .m4a)")]
        file_path: String,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
    ) -> MCPResult {
        self.cli.send_voice_mcp(&file_path, chat_id.as_deref()).await
    }

    #[tool(description = "Edit existing message")]
    async fn edit_message(
        &self,
        #[tool(param)]
        #[schemars(description = "Message ID to edit")]
        message_id: String,
        #[tool(param)]
        #[schemars(description = "New text for the message")]
        new_text: String,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
    ) -> MCPResult {
        self.cli
            .edit_message_mcp(&message_id, &new_text, chat_id.as_deref())
            .await
    }

    #[tool(description = "Delete message from chat")]
    async fn delete_message(
        &self,
        #[tool(param)]
        #[schemars(description = "Message ID to delete")]
        message_id: String,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
    ) -> MCPResult {
        self.cli
            .delete_message_mcp(&message_id, chat_id.as_deref())
            .await
    }

    #[tool(description = "Pin message in chat")]
    async fn pin_message(
        &self,
        #[tool(param)]
        #[schemars(description = "Message ID to pin")]
        message_id: String,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
    ) -> MCPResult {
        self.cli.pin_message_mcp(&message_id, chat_id.as_deref()).await
    }

    #[tool(description = "Unpin message from chat")]
    async fn unpin_message(
        &self,
        #[tool(param)]
        #[schemars(description = "Message ID to unpin")]
        message_id: String,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
    ) -> MCPResult {
        self.cli
            .unpin_message_mcp(&message_id, chat_id.as_deref())
            .await
    }

    // === Chat Management Commands ===

    #[tool(description = "Get information about the chat")]
    async fn chat_info(
        &self,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
    ) -> MCPResult {
        self.cli.get_chat_info_mcp(chat_id.as_deref()).await
    }

    #[tool(description = "Get user profile information")]
    async fn get_profile(
        &self,
        #[tool(param)]
        #[schemars(description = "User ID to get profile for")]
        user_id: String,
    ) -> MCPResult {
        self.cli.get_profile_mcp(&user_id).await
    }

    #[tool(description = "Get chat members")]
    async fn get_chat_members(
        &self,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Cursor for pagination (optional)")]
        cursor: Option<String>,
    ) -> MCPResult {
        self.cli
            .get_chat_members_mcp(chat_id.as_deref(), cursor.as_deref())
            .await
    }

    #[tool(description = "Get chat administrators")]
    async fn get_chat_admins(
        &self,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
    ) -> MCPResult {
        self.cli.get_chat_admins_mcp(chat_id.as_deref()).await
    }

    #[tool(description = "Set chat title")]
    async fn set_chat_title(
        &self,
        #[tool(param)]
        #[schemars(description = "New chat title")]
        title: String,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
    ) -> MCPResult {
        self.cli.set_chat_title_mcp(&title, chat_id.as_deref()).await
    }

    #[tool(description = "Set chat description")]
    async fn set_chat_about(
        &self,
        #[tool(param)]
        #[schemars(description = "New chat description")]
        about: String,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
    ) -> MCPResult {
        self.cli.set_chat_about_mcp(&about, chat_id.as_deref()).await
    }

    #[tool(description = "Send typing or looking action to chat")]
    async fn send_action(
        &self,
        #[tool(param)]
        #[schemars(description = "Action to send: 'typing' or 'looking'")]
        action: String,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
    ) -> MCPResult {
        self.cli.send_action_mcp(&action, chat_id.as_deref()).await
    }

    // === File Upload Commands ===

    #[tool(description = "Upload file from base64 content")]
    async fn upload_file_from_base64(
        &self,
        #[tool(param)]
        #[schemars(description = "File name with extension")]
        file_name: String,
        #[tool(param)]
        #[schemars(description = "Base64 encoded file content")]
        base64_content: String,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Caption for the file (optional)")]
        caption: Option<String>,
        #[tool(param)]
        #[schemars(description = "Reply to message ID (optional)")]
        reply_msg_id: Option<String>,
    ) -> MCPResult {
        self.cli
            .upload_file_base64_mcp(
                &file_name,
                &base64_content,
                chat_id.as_deref(),
                caption.as_deref(),
                reply_msg_id.as_deref(),
            )
            .await
    }

    #[tool(description = "Upload text content as file")]
    async fn upload_text_as_file(
        &self,
        #[tool(param)]
        #[schemars(description = "File name with extension")]
        file_name: String,
        #[tool(param)]
        #[schemars(description = "Text content to save as file")]
        content: String,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Caption for the file (optional)")]
        caption: Option<String>,
    ) -> MCPResult {
        self.cli
            .upload_text_file_mcp(&file_name, &content, chat_id.as_deref(), caption.as_deref())
            .await
    }

    #[tool(description = "Upload JSON data as file")]
    async fn upload_json_file(
        &self,
        #[tool(param)]
        #[schemars(description = "File name (will add .json extension if missing)")]
        file_name: String,
        #[tool(param)]
        #[schemars(description = "JSON data as string")]
        json_data: String,
        #[tool(param)]
        #[schemars(description = "Pretty print JSON (default: true)")]
        pretty: Option<bool>,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Caption for the file (optional)")]
        caption: Option<String>,
    ) -> MCPResult {
        self.cli
            .upload_json_file_mcp(
                &file_name,
                &json_data,
                pretty.unwrap_or(true),
                chat_id.as_deref(),
                caption.as_deref(),
            )
            .await
    }

    #[tool(description = "Get file information")]
    async fn file_info(
        &self,
        #[tool(param)]
        #[schemars(description = "File ID to get information about")]
        file_id: String,
    ) -> MCPResult {
        self.cli.get_file_info_mcp(&file_id).await
    }

    // === Storage Commands ===

    #[tool(description = "Search messages using semantic similarity")]
    async fn search_semantic(
        &self,
        #[tool(param)]
        #[schemars(description = "Search query")]
        query: String,
        #[tool(param)]
        #[schemars(description = "Chat ID to search in (optional, searches all if not provided)")]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Maximum number of results (default: 10)")]
        limit: Option<usize>,
    ) -> MCPResult {
        self.cli
            .search_semantic_mcp(&query, chat_id.as_deref(), limit)
            .await
    }

    #[tool(description = "Search messages using text search")]
    async fn search_text(
        &self,
        #[tool(param)]
        #[schemars(description = "Search query")]
        query: String,
        #[tool(param)]
        #[schemars(description = "Chat ID to search in (optional, searches all if not provided)")]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Maximum number of results (default: 10)")]
        limit: Option<i64>,
    ) -> MCPResult {
        self.cli
            .search_text_mcp(&query, chat_id.as_deref(), limit)
            .await
    }

    #[tool(description = "Get database statistics")]
    async fn get_database_stats(
        &self,
        #[tool(param)]
        #[schemars(
            description = "Chat ID for specific chat stats (optional, all chats if not provided)"
        )]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Date since when to count (optional)")]
        since: Option<String>,
    ) -> MCPResult {
        self.cli
            .get_database_stats_mcp(chat_id.as_deref(), since.as_deref())
            .await
    }

    #[tool(description = "Get conversation context")]
    async fn get_context(
        &self,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(
            description = "Context type: 'recent', 'summary', or 'keywords' (default: 'recent')"
        )]
        context_type: Option<String>,
        #[tool(param)]
        #[schemars(description = "Timeframe for context (optional)")]
        timeframe: Option<String>,
    ) -> MCPResult {
        self.cli
            .get_context_mcp(
                chat_id.as_deref(),
                context_type.as_deref(),
                timeframe.as_deref(),
            )
            .await
    }

    // === Daemon Management Commands ===

    #[tool(description = "Get daemon status and statistics")]
    async fn daemon_status(&self) -> MCPResult {
        self.cli.get_daemon_status_mcp().await
    }

    #[tool(description = "Get recent messages from storage")]
    async fn get_recent_messages(
        &self,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, gets from all chats if not provided)")]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Maximum number of messages to return (default: 50)")]
        limit: Option<usize>,
        #[tool(param)]
        #[schemars(description = "Get messages since this timestamp (ISO 8601 format)")]
        since: Option<String>,
    ) -> MCPResult {
        self.cli
            .get_recent_messages_mcp(chat_id.as_deref(), limit, since.as_deref())
            .await
    }

    // === Diagnostic Commands ===

    #[tool(description = "Get information about the bot")]
    async fn self_get(&self) -> MCPResult {
        self.cli.get_self_mcp().await
    }

    #[tool(description = "Get events from the bot")]
    async fn events_get(
        &self,
        #[tool(param)]
        #[schemars(description = "Last event ID for pagination (optional)")]
        last_event_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Poll time in seconds (optional)")]
        poll_time: Option<u64>,
    ) -> MCPResult {
        self.cli
            .get_events_mcp(last_event_id.as_deref(), poll_time)
            .await
    }

    tool_box!(Server {
        self_get,
        send_text,
        send_file,
        send_voice,
        edit_message,
        delete_message,
        pin_message,
        unpin_message,
        chat_info,
        get_profile,
        get_chat_members,
        get_chat_admins,
        set_chat_title,
        set_chat_about,
        send_action,
        upload_file_from_base64,
        upload_text_as_file,
        upload_json_file,
        file_info,
        search_semantic,
        search_text,
        get_database_stats,
        get_context,
        daemon_status,
        get_recent_messages,
        events_get
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge_trait::{CliBridgeTrait, MockCliBridge};
    use crate::errors::CliErrorInfo;
    use std::time::Duration;

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
                    "Failed for code: {}",
                    code
                );
            }
        }
    }
}
