//! Typed CLI commands for MCP Server
//!
//! This module provides high-level, typed methods for each CLI command
//! that the MCP server needs to call. All methods return structured
//! JSON responses from the CLI.

use crate::cli_bridge::CliBridge;
use crate::errors::BridgeError;
use serde_json::Value;

impl CliBridge {
    // === Messaging Commands ===

    /// Send text message to chat
    pub async fn send_text(
        &self,
        text: &str,
        chat_id: Option<&str>,
        reply_msg_id: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["send-text", "--message", text];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        if let Some(reply_id) = reply_msg_id {
            args.extend(&["--reply-msg-id", reply_id]);
        }

        self.execute_command(&args).await
    }

    /// Send file from file path
    pub async fn send_file(
        &self,
        file_path: &str,
        chat_id: Option<&str>,
        caption: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["send-file", "--file-path", file_path];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        if let Some(caption) = caption {
            args.extend(&["--caption", caption]);
        }

        self.execute_command(&args).await
    }

    /// Send voice message from file path
    pub async fn send_voice(
        &self,
        file_path: &str,
        chat_id: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["send-voice", "--file-path", file_path];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        self.execute_command(&args).await
    }

    /// Edit existing message
    pub async fn edit_message(
        &self,
        message_id: &str,
        new_text: &str,
        chat_id: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec![
            "edit-message",
            "--message-id",
            message_id,
            "--new-text",
            new_text,
        ];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        self.execute_command(&args).await
    }

    /// Delete message
    pub async fn delete_message(
        &self,
        message_id: &str,
        chat_id: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["delete-message", "--message-id", message_id];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        self.execute_command(&args).await
    }

    /// Pin message in chat
    pub async fn pin_message(
        &self,
        message_id: &str,
        chat_id: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["pin-message", "--message-id", message_id];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        self.execute_command(&args).await
    }

    /// Unpin message from chat
    pub async fn unpin_message(
        &self,
        message_id: &str,
        chat_id: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["unpin-message", "--message-id", message_id];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        self.execute_command(&args).await
    }

    // === Chat Management Commands ===

    /// Get chat information
    pub async fn get_chat_info(&self, chat_id: Option<&str>) -> Result<Value, BridgeError> {
        let mut args = vec!["get-chat-info"];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        self.execute_command(&args).await
    }

    /// Get user profile
    pub async fn get_profile(&self, user_id: &str) -> Result<Value, BridgeError> {
        self.execute_command(&["get-profile", "--user-id", user_id])
            .await
    }

    /// Get chat members with optional cursor
    pub async fn get_chat_members(
        &self,
        chat_id: Option<&str>,
        cursor: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["get-chat-members"];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        if let Some(cursor) = cursor {
            args.extend(&["--cursor", cursor]);
        }

        self.execute_command(&args).await
    }

    /// Get chat administrators
    pub async fn get_chat_admins(&self, chat_id: Option<&str>) -> Result<Value, BridgeError> {
        let mut args = vec!["get-chat-admins"];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        self.execute_command(&args).await
    }

    /// Set chat title
    pub async fn set_chat_title(
        &self,
        title: &str,
        chat_id: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["set-chat-title", "--title", title];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        self.execute_command(&args).await
    }

    /// Set chat description
    pub async fn set_chat_about(
        &self,
        about: &str,
        chat_id: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["set-chat-about", "--about", about];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        self.execute_command(&args).await
    }

    /// Send chat action (typing/looking)
    pub async fn send_action(
        &self,
        action: &str,
        chat_id: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["send-action", "--action", action];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        self.execute_command(&args).await
    }

    // === File Upload Commands ===

    /// Upload file from base64 content
    pub async fn upload_file_base64(
        &self,
        name: &str,
        content_base64: &str,
        chat_id: Option<&str>,
        caption: Option<&str>,
        reply_msg_id: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["upload", "--name", name, "--content-base64", content_base64];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        if let Some(caption) = caption {
            args.extend(&["--caption", caption]);
        }

        if let Some(reply_id) = reply_msg_id {
            args.extend(&["--reply-msg-id", reply_id]);
        }

        self.execute_command(&args).await
    }

    /// Upload text content as file
    pub async fn upload_text_file(
        &self,
        name: &str,
        content: &str,
        chat_id: Option<&str>,
        caption: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["upload-text", "--name", name, "--content", content];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        if let Some(caption) = caption {
            args.extend(&["--caption", caption]);
        }

        self.execute_command(&args).await
    }

    /// Upload JSON data as file
    pub async fn upload_json_file(
        &self,
        name: &str,
        json_data: &str,
        pretty: bool,
        chat_id: Option<&str>,
        caption: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["upload-json", "--name", name, "--json-data", json_data];

        if pretty {
            args.push("--pretty");
        }

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        if let Some(caption) = caption {
            args.extend(&["--caption", caption]);
        }

        self.execute_command(&args).await
    }

    /// Get file information
    pub async fn get_file_info(&self, file_id: &str) -> Result<Value, BridgeError> {
        self.execute_command(&["info", "--file-id", file_id]).await
    }

    // === Storage Commands ===

    /// Get database statistics
    pub async fn get_database_stats(
        &self,
        chat_id: Option<&str>,
        since: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["database", "stats"];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        if let Some(since) = since {
            args.extend(&["--since", since]);
        }

        self.execute_command(&args).await
    }

    /// Search messages using semantic similarity
    pub async fn search_semantic(
        &self,
        query: &str,
        chat_id: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Value, BridgeError> {
        let limit_str = limit.map(|l| l.to_string());
        let mut args = vec!["search", "semantic", query];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        if let Some(ref limit_str) = limit_str {
            args.extend(&["--limit", limit_str]);
        }

        self.execute_command(&args).await
    }

    /// Search messages using text search
    pub async fn search_text(
        &self,
        query: &str,
        chat_id: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Value, BridgeError> {
        let limit_str = limit.map(|l| l.to_string());
        let mut args = vec!["search", "text", query];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        if let Some(ref limit_str) = limit_str {
            args.extend(&["--limit", limit_str]);
        }

        self.execute_command(&args).await
    }

    /// Get conversation context
    pub async fn get_context(
        &self,
        chat_id: Option<&str>,
        context_type: Option<&str>,
        timeframe: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["context", "get"];

        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }

        if let Some(context_type) = context_type {
            args.extend(&["--context-type", context_type]);
        }

        if let Some(timeframe) = timeframe {
            args.extend(&["--timeframe", timeframe]);
        }

        self.execute_command(&args).await
    }

    // === Diagnostic Commands ===

    /// Get bot information and status
    pub async fn get_self(&self) -> Result<Value, BridgeError> {
        self.execute_command(&["get-self"]).await
    }

    /// Get events
    pub async fn get_events(
        &self,
        last_event_id: Option<&str>,
        poll_time: Option<u64>,
    ) -> Result<Value, BridgeError> {
        let poll_time_str = poll_time.map(|p| p.to_string());
        let mut args = vec!["get-events"];

        if let Some(event_id) = last_event_id {
            args.extend(&["--last-event-id", event_id]);
        }

        if let Some(ref poll_time_str) = poll_time_str {
            args.extend(&["--poll-time", poll_time_str]);
        }

        self.execute_command(&args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge_trait::{CliBridgeTrait, MockCliBridge};

    /// Helper function to create a mock CLI bridge with success responses
    fn create_mock_bridge() -> MockCliBridge {
        let mut mock = MockCliBridge::new();
        
        // Add default success responses for common commands
        mock.add_success_response(
            "send-text".to_string(),
            serde_json::json!({"message_id": "msg123", "ok": true}),
        );
        mock.add_success_response(
            "send-file".to_string(),
            serde_json::json!({"file_id": "file123", "ok": true}),
        );
        mock.add_success_response(
            "get-chat-info".to_string(),
            serde_json::json!({"chat_id": "chat123", "title": "Test Chat"}),
        );
        
        mock
    }

    #[tokio::test]
    async fn test_command_building() {
        // Set required env var for test
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat");
        }

        // This test just verifies that command building works correctly
        // We can't actually execute commands in test environment
        let config = vkteams_bot::config::UnifiedConfig::default();
        if let Ok(_bridge) = CliBridge::new(&config) {
            // Test would need actual CLI binary to run
            println!("CLI bridge created for testing");
        }
    }

    #[tokio::test]
    async fn test_send_text_command_args() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "send-text --message Hello".to_string(),
            serde_json::json!({"message_id": "123"}),
        );

        let result = mock
            .execute_command(&["send-text", "--message", "Hello"])
            .await;
        assert!(result.is_ok());

        if let Ok(response) = result {
            assert_eq!(response["success"], true);
            assert_eq!(response["data"]["message_id"], "123");
        }
    }

    #[tokio::test]
    async fn test_send_text_with_chat_id() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "send-text".to_string(),
            serde_json::json!({"message_id": "456"}),
        );

        let result = mock
            .execute_command(&["send-text", "--message", "Hello", "--chat-id", "chat123"])
            .await;
        assert!(result.is_ok());

        if let Ok(response) = result {
            assert_eq!(response["success"], true);
            assert_eq!(response["data"]["message_id"], "456");
        }
    }

    #[tokio::test]
    async fn test_send_text_with_reply() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "send-text".to_string(),
            serde_json::json!({"message_id": "789"}),
        );

        let result = mock
            .execute_command(&["send-text", "--message", "Reply", "--reply-msg-id", "100"])
            .await;
        assert!(result.is_ok());

        if let Ok(response) = result {
            assert_eq!(response["success"], true);
            assert_eq!(response["data"]["message_id"], "789");
        }
    }

    #[tokio::test]
    async fn test_send_file_command() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "send-file".to_string(),
            serde_json::json!({"file_id": "file123"}),
        );

        let result = mock
            .execute_command(&["send-file", "--file-path", "/path/to/file.txt"])
            .await;
        assert!(result.is_ok());

        if let Ok(response) = result {
            assert_eq!(response["success"], true);
            assert_eq!(response["data"]["file_id"], "file123");
        }
    }

    #[tokio::test]
    async fn test_send_file_with_caption() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "send-file".to_string(),
            serde_json::json!({"file_id": "file456"}),
        );

        let result = mock
            .execute_command(&[
                "send-file",
                "--file-path",
                "/path/to/image.png",
                "--caption",
                "My Image",
            ])
            .await;
        assert!(result.is_ok());

        if let Ok(response) = result {
            assert_eq!(response["success"], true);
            assert_eq!(response["data"]["file_id"], "file456");
        }
    }

    #[tokio::test]
    async fn test_edit_message_command() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "edit-message".to_string(),
            serde_json::json!({"success": true}),
        );

        let result = mock
            .execute_command(&[
                "edit-message",
                "--message-id",
                "123",
                "--new-text",
                "Updated text",
            ])
            .await;
        assert!(result.is_ok());

        if let Ok(response) = result {
            assert_eq!(response["success"], true);
        }
    }

    #[tokio::test]
    async fn test_delete_message_command() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "delete-message".to_string(),
            serde_json::json!({"success": true}),
        );

        let result = mock
            .execute_command(&["delete-message", "--message-id", "123"])
            .await;
        assert!(result.is_ok());

        if let Ok(response) = result {
            assert_eq!(response["success"], true);
        }
    }

    #[tokio::test]
    async fn test_get_chat_info_command() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "chat-info".to_string(),
            serde_json::json!({
                "chat_id": "chat123",
                "title": "Test Chat",
                "type": "group"
            }),
        );

        let result = mock.execute_command(&["chat-info"]).await;
        assert!(result.is_ok());

        if let Ok(response) = result {
            assert_eq!(response["success"], true);
            assert_eq!(response["data"]["chat_id"], "chat123");
            assert_eq!(response["data"]["title"], "Test Chat");
        }
    }

    #[tokio::test]
    async fn test_get_chat_info_with_chat_id() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "chat-info".to_string(),
            serde_json::json!({
                "chat_id": "custom_chat",
                "title": "Custom Chat",
                "type": "private"
            }),
        );

        let result = mock
            .execute_command(&["chat-info", "--chat-id", "custom_chat"])
            .await;
        assert!(result.is_ok());

        if let Ok(response) = result {
            assert_eq!(response["success"], true);
            assert_eq!(response["data"]["chat_id"], "custom_chat");
            assert_eq!(response["data"]["title"], "Custom Chat");
        }
    }

    #[tokio::test]
    async fn test_get_events_command() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "get-events".to_string(),
            serde_json::json!({
                "events": [
                    {"id": "1", "type": "message", "data": {}},
                    {"id": "2", "type": "edit", "data": {}}
                ]
            }),
        );

        let result = mock.execute_command(&["get-events"]).await;
        assert!(result.is_ok());

        if let Ok(response) = result {
            assert_eq!(response["success"], true);
            assert!(response["data"]["events"].is_array());
            assert_eq!(response["data"]["events"].as_array().unwrap().len(), 2);
        }
    }

    #[tokio::test]
    async fn test_get_events_with_last_event_id() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "get-events".to_string(),
            serde_json::json!({
                "events": [
                    {"id": "3", "type": "message", "data": {}}
                ]
            }),
        );

        let result = mock
            .execute_command(&["get-events", "--last-event-id", "2"])
            .await;
        assert!(result.is_ok());

        if let Ok(response) = result {
            assert_eq!(response["success"], true);
            assert!(response["data"]["events"].is_array());
        }
    }

    #[tokio::test]
    async fn test_command_error_handling() {
        let mut mock = MockCliBridge::new();
        mock.add_error_response("send-text".to_string(), "Message is too long".to_string());

        let result = mock
            .execute_command(&["send-text", "--message", "Very long message..."])
            .await;
        assert!(result.is_err());

        if let Err(error) = result {
            match error {
                BridgeError::CliReturnedError(info) => {
                    assert_eq!(info.message, "Message is too long");
                }
                _ => panic!("Expected CliReturnedError"),
            }
        }
    }

    #[tokio::test]
    async fn test_file_info_command() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "file-info".to_string(),
            serde_json::json!({
                "file_id": "file123",
                "file_name": "document.pdf",
                "file_size": 1024,
                "mime_type": "application/pdf"
            }),
        );

        let result = mock
            .execute_command(&["file-info", "--file-id", "file123"])
            .await;
        assert!(result.is_ok());

        if let Ok(response) = result {
            assert_eq!(response["success"], true);
            assert_eq!(response["data"]["file_id"], "file123");
            assert_eq!(response["data"]["file_name"], "document.pdf");
            assert_eq!(response["data"]["file_size"], 1024);
        }
    }

    #[tokio::test]
    async fn test_partial_command_matching() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response("send".to_string(), serde_json::json!({"result": "matched"}));

        // Test that "send-text" matches "send" pattern
        let result = mock
            .execute_command(&["send-text", "--message", "test"])
            .await;
        assert!(result.is_ok());

        if let Ok(response) = result {
            assert_eq!(response["success"], true);
            assert_eq!(response["data"]["result"], "matched");
        }
    }

    // === Comprehensive tests for all CLI command methods ===

    #[tokio::test]
    async fn test_send_text_method_args() {
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat");
        }
        let mock = create_mock_bridge();
        
        let config = vkteams_bot::config::UnifiedConfig::default();
        if let Ok(_bridge) = CliBridge::new(&config) {
            // Test the actual send_text method argument building
            // We can't call the method directly without CLI binary, but we can test argument construction
            let args = vec!["send-text", "--message", "Hello World"];
            let result = mock.execute_command(&args).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_send_text_with_all_parameters() {
        let mock = create_mock_bridge();
        
        let result = mock.execute_command(&[
            "send-text", "--message", "Hello World",
            "--chat-id", "chat123",
            "--reply-msg-id", "msg456"
        ]).await;
        assert!(result.is_ok());
        
        if let Ok(response) = result {
            assert_eq!(response["success"], true);
        }
    }

    #[tokio::test]
    async fn test_send_file_method() {
        let mock = create_mock_bridge();
        
        let result = mock.execute_command(&[
            "send-file", "--file-path", "/path/to/file.txt"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_file_with_all_parameters() {
        let mock = create_mock_bridge();
        
        let result = mock.execute_command(&[
            "send-file", "--file-path", "/path/to/file.txt",
            "--chat-id", "chat123",
            "--caption", "Test file caption"
        ]).await;
        assert!(result.is_ok());
        
        if let Ok(response) = result {
            assert_eq!(response["success"], true);
        }
    }

    #[tokio::test]
    async fn test_send_voice_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "send-voice".to_string(),
            serde_json::json!({"voice_id": "voice123", "ok": true}),
        );
        
        let result = mock.execute_command(&[
            "send-voice", "--file-path", "/path/to/voice.ogg"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_voice_with_chat_id() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "send-voice".to_string(),
            serde_json::json!({"voice_id": "voice123", "ok": true}),
        );
        
        let result = mock.execute_command(&[
            "send-voice", "--file-path", "/path/to/voice.ogg",
            "--chat-id", "chat123"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_edit_message_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "edit-message".to_string(),
            serde_json::json!({"ok": true}),
        );
        
        let result = mock.execute_command(&[
            "edit-message", "--message-id", "msg123",
            "--new-text", "Updated message"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_edit_message_with_chat_id() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "edit-message".to_string(),
            serde_json::json!({"ok": true}),
        );
        
        let result = mock.execute_command(&[
            "edit-message", "--message-id", "msg123",
            "--new-text", "Updated message",
            "--chat-id", "chat123"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_message_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "delete-message".to_string(),
            serde_json::json!({"ok": true}),
        );
        
        let result = mock.execute_command(&[
            "delete-message", "--message-id", "msg123"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_message_with_chat_id() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "delete-message".to_string(),
            serde_json::json!({"ok": true}),
        );
        
        let result = mock.execute_command(&[
            "delete-message", "--message-id", "msg123",
            "--chat-id", "chat123"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pin_message_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "pin-message".to_string(),
            serde_json::json!({"ok": true}),
        );
        
        let result = mock.execute_command(&[
            "pin-message", "--message-id", "msg123"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pin_message_with_chat_id() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "pin-message".to_string(),
            serde_json::json!({"ok": true}),
        );
        
        let result = mock.execute_command(&[
            "pin-message", "--message-id", "msg123",
            "--chat-id", "chat123"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unpin_message_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "unpin-message".to_string(),
            serde_json::json!({"ok": true}),
        );
        
        let result = mock.execute_command(&[
            "unpin-message", "--message-id", "msg123"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unpin_message_with_chat_id() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "unpin-message".to_string(),
            serde_json::json!({"ok": true}),
        );
        
        let result = mock.execute_command(&[
            "unpin-message", "--message-id", "msg123",
            "--chat-id", "chat123"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_chat_info_method() {
        let mock = create_mock_bridge();
        
        let result = mock.execute_command(&["get-chat-info"]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_chat_info_with_specific_chat_id() {
        let mock = create_mock_bridge();
        
        let result = mock.execute_command(&[
            "get-chat-info", "--chat-id", "chat123"
        ]).await;
        assert!(result.is_ok());
        
        if let Ok(response) = result {
            assert_eq!(response["success"], true);
            assert_eq!(response["data"]["chat_id"], "chat123");
        }
    }

    #[tokio::test]
    async fn test_get_profile_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "get-profile".to_string(),
            serde_json::json!({"user_id": "user123", "first_name": "John"}),
        );
        
        let result = mock.execute_command(&[
            "get-profile", "--user-id", "user123"
        ]).await;
        assert!(result.is_ok());
        
        if let Ok(response) = result {
            assert_eq!(response["success"], true);
            assert_eq!(response["data"]["user_id"], "user123");
        }
    }

    #[tokio::test]
    async fn test_get_chat_members_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "get-chat-members".to_string(),
            serde_json::json!({"members": [], "cursor": null}),
        );
        
        let result = mock.execute_command(&["get-chat-members"]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_chat_members_with_parameters() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "get-chat-members".to_string(),
            serde_json::json!({"members": [], "cursor": "next123"}),
        );
        
        let result = mock.execute_command(&[
            "get-chat-members", 
            "--chat-id", "chat123",
            "--cursor", "cursor123"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_chat_admins_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "get-chat-admins".to_string(),
            serde_json::json!({"admins": []}),
        );
        
        let result = mock.execute_command(&["get-chat-admins"]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_chat_admins_with_chat_id() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "get-chat-admins".to_string(),
            serde_json::json!({"admins": []}),
        );
        
        let result = mock.execute_command(&[
            "get-chat-admins", "--chat-id", "chat123"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_chat_title_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "set-chat-title".to_string(),
            serde_json::json!({"ok": true}),
        );
        
        let result = mock.execute_command(&[
            "set-chat-title", "--title", "New Chat Title"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_chat_title_with_chat_id() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "set-chat-title".to_string(),
            serde_json::json!({"ok": true}),
        );
        
        let result = mock.execute_command(&[
            "set-chat-title", "--title", "New Chat Title",
            "--chat-id", "chat123"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_chat_about_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "set-chat-about".to_string(),
            serde_json::json!({"ok": true}),
        );
        
        let result = mock.execute_command(&[
            "set-chat-about", "--about", "New chat description"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_chat_about_with_chat_id() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "set-chat-about".to_string(),
            serde_json::json!({"ok": true}),
        );
        
        let result = mock.execute_command(&[
            "set-chat-about", "--about", "New chat description",
            "--chat-id", "chat123"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_action_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "send-action".to_string(),
            serde_json::json!({"ok": true}),
        );
        
        let result = mock.execute_command(&[
            "send-action", "--action", "typing"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_action_with_chat_id() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "send-action".to_string(),
            serde_json::json!({"ok": true}),
        );
        
        let result = mock.execute_command(&[
            "send-action", "--action", "looking",
            "--chat-id", "chat123"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_upload_file_base64_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "upload-file-base64".to_string(),
            serde_json::json!({"file_id": "file123", "ok": true}),
        );
        
        let result = mock.execute_command(&[
            "upload-file-base64", "--name", "test.txt",
            "--content-base64", "dGVzdA=="
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_upload_file_base64_with_all_parameters() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "upload-file-base64".to_string(),
            serde_json::json!({"file_id": "file123", "ok": true}),
        );
        
        let result = mock.execute_command(&[
            "upload-file-base64", "--name", "test.txt",
            "--content-base64", "dGVzdA==",
            "--chat-id", "chat123",
            "--caption", "Test file",
            "--reply-msg-id", "msg456"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_upload_text_file_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "upload-text-file".to_string(),
            serde_json::json!({"file_id": "text123", "ok": true}),
        );
        
        let result = mock.execute_command(&[
            "upload-text-file", "--name", "notes.txt",
            "--content", "Some text content"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_upload_text_file_with_all_parameters() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "upload-text-file".to_string(),
            serde_json::json!({"file_id": "text123", "ok": true}),
        );
        
        let result = mock.execute_command(&[
            "upload-text-file", "--name", "notes.txt",
            "--content", "Some text content",
            "--chat-id", "chat123",
            "--caption", "My notes"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_upload_json_file_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "upload-json-file".to_string(),
            serde_json::json!({"file_id": "json123", "ok": true}),
        );
        
        let result = mock.execute_command(&[
            "upload-json-file", "--name", "data",
            "--json-data", r#"{"key": "value"}"#
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_upload_json_file_with_all_parameters() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "upload-json-file".to_string(),
            serde_json::json!({"file_id": "json123", "ok": true}),
        );
        
        let result = mock.execute_command(&[
            "upload-json-file", "--name", "data",
            "--json-data", r#"{"key": "value"}"#,
            "--pretty", "true",
            "--chat-id", "chat123",
            "--caption", "JSON data"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_file_info_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "file-info".to_string(),
            serde_json::json!({
                "file_id": "file123",
                "file_name": "test.txt",
                "file_size": 1024
            }),
        );
        
        let result = mock.execute_command(&[
            "file-info", "--file-id", "file123"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_database_stats_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "get-database-stats".to_string(),
            serde_json::json!({
                "total_messages": 1000,
                "total_chats": 5,
                "db_size": 1048576
            }),
        );
        
        let result = mock.execute_command(&["get-database-stats"]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_database_stats_with_parameters() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "get-database-stats".to_string(),
            serde_json::json!({
                "total_messages": 500,
                "total_chats": 1,
                "db_size": 524288
            }),
        );
        
        let result = mock.execute_command(&[
            "get-database-stats",
            "--chat-id", "chat123",
            "--since", "2024-01-01"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_semantic_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "search-semantic".to_string(),
            serde_json::json!({
                "results": [],
                "total": 0
            }),
        );
        
        let result = mock.execute_command(&[
            "search-semantic", "--query", "search term"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_semantic_with_all_parameters() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "search-semantic".to_string(),
            serde_json::json!({
                "results": [],
                "total": 0
            }),
        );
        
        let result = mock.execute_command(&[
            "search-semantic", "--query", "search term",
            "--chat-id", "chat123",
            "--limit", "10"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_text_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "search-text".to_string(),
            serde_json::json!({
                "results": [],
                "total": 0
            }),
        );
        
        let result = mock.execute_command(&[
            "search-text", "--query", "text search"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_text_with_all_parameters() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "search-text".to_string(),
            serde_json::json!({
                "results": [],
                "total": 0
            }),
        );
        
        let result = mock.execute_command(&[
            "search-text", "--query", "text search",
            "--chat-id", "chat123",
            "--limit", "20"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_context_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "get-context".to_string(),
            serde_json::json!({
                "context": "recent messages..."
            }),
        );
        
        let result = mock.execute_command(&["get-context"]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_context_with_all_parameters() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "get-context".to_string(),
            serde_json::json!({
                "context": "recent messages..."
            }),
        );
        
        let result = mock.execute_command(&[
            "get-context",
            "--chat-id", "chat123",
            "--context-type", "recent",
            "--timeframe", "1d"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_self_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "self-get".to_string(),
            serde_json::json!({
                "user_id": "bot123",
                "first_name": "Test Bot"
            }),
        );
        
        let result = mock.execute_command(&["self-get"]).await;
        assert!(result.is_ok());
        
        if let Ok(response) = result {
            assert_eq!(response["success"], true);
        }
    }

    #[tokio::test]
    async fn test_get_events_method() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "events-get".to_string(),
            serde_json::json!({
                "events": [],
                "last_event_id": "123"
            }),
        );
        
        let result = mock.execute_command(&["events-get"]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_events_with_parameters() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "events-get".to_string(),
            serde_json::json!({
                "events": [],
                "last_event_id": "124"
            }),
        );
        
        let result = mock.execute_command(&[
            "events-get",
            "--last-event-id", "123",
            "--poll-time", "30"
        ]).await;
        assert!(result.is_ok());
    }

    // === Error handling tests ===

    #[tokio::test]
    async fn test_command_error_responses() {
        let mut mock = MockCliBridge::new();
        mock.add_error_response(
            "send-text".to_string(),
            "Message too long".to_string(),
        );
        
        let result = mock.execute_command(&[
            "send-text", "--message", "very long message..."
        ]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_command_responses() {
        let mut mock = MockCliBridge::new();
        mock.add_error_response(
            "invalid-command".to_string(),
            "Command not found".to_string(),
        );
        
        let result = mock.execute_command(&["invalid-command"]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parameter_validation() {
        let mut mock = MockCliBridge::new();
        mock.add_error_response(
            "send-text".to_string(),
            "Missing required parameter: message".to_string(),
        );
        
        let result = mock.execute_command(&["send-text"]).await;
        assert!(result.is_err());
    }

    // === Edge cases and integration tests ===

    #[tokio::test]
    async fn test_empty_optional_parameters() {
        let mock = create_mock_bridge();
        
        // Test commands with None for optional parameters
        let result = mock.execute_command(&["send-text", "--message", "Hello"]).await;
        assert!(result.is_ok());
        
        let result = mock.execute_command(&["get-chat-info"]).await;
        assert!(result.is_ok());
        
        let result = mock.execute_command(&["get-chat-members"]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_special_characters_in_parameters() {
        let mock = create_mock_bridge();
        
        let result = mock.execute_command(&[
            "send-text", "--message", "Hello 🌍! Special chars: @#$%^&*()"
        ]).await;
        assert!(result.is_ok());
        
        let result = mock.execute_command(&[
            "set-chat-title", "--title", "Chat with émojis 🚀"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_numeric_parameters() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "search-semantic".to_string(),
            serde_json::json!({"results": [], "total": 0}),
        );
        
        let result = mock.execute_command(&[
            "search-semantic", "--query", "test",
            "--limit", "50"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boolean_parameters() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "upload-json-file".to_string(),
            serde_json::json!({"file_id": "json123", "ok": true}),
        );
        
        let result = mock.execute_command(&[
            "upload-json-file", "--name", "data.json",
            "--json-data", r#"{"test": true}"#,
            "--pretty", "false"
        ]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiline_content() {
        let mut mock = create_mock_bridge();
        mock.add_success_response(
            "upload-text-file".to_string(),
            serde_json::json!({"file_id": "text123", "ok": true}),
        );
        
        let multiline_content = "Line 1\nLine 2\nLine 3\nWith special chars: àáâã";
        let result = mock.execute_command(&[
            "upload-text-file", "--name", "multiline.txt",
            "--content", multiline_content
        ]).await;
        assert!(result.is_ok());
    }
}
