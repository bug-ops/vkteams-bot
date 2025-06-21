//! New MCP Server implementation using CLI Bridge
//!
//! This module contains the new MCP server implementation that uses the CLI
//! bridge instead of direct library calls. This ensures single source of truth
//! for all business logic in the CLI.

use crate::cli_bridge::BridgeError;
use crate::types::Server;
use rmcp::tool_box;
use rmcp::{
    ServerHandler,
    model::{CallToolResult, Content, ErrorData, ServerCapabilities, ServerInfo},
    tool,
};
use serde_json::Value;
use std::result::Result;

pub type MCPResult = Result<CallToolResult, ErrorData>;

/// Convert CLI bridge result to MCP result
#[allow(dead_code)]
fn convert_bridge_result(result: Result<Value, BridgeError>) -> MCPResult {
    match result {
        Ok(json_response) => {
            // CLI already returns structured JSON, just pass it through
            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string(&json_response)
                    .unwrap_or_else(|_| "{}".to_string())
            )]))
        }
        Err(e) => Err(ErrorData {
            code: rmcp::model::ErrorCode(-1),
            message: format!("CLI command failed: {}", e).into(),
            data: None,
        }),
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
        let result = self.cli.send_text(
            &text,
            chat_id.as_deref(),
            reply_msg_id.as_deref(),
        ).await;
        
        convert_bridge_result(result)
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
        let result = self.cli.send_file(
            &file_path,
            chat_id.as_deref(),
            caption.as_deref(),
        ).await;
        
        convert_bridge_result(result)
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
        let result = self.cli.send_voice(
            &file_path,
            chat_id.as_deref(),
        ).await;
        
        convert_bridge_result(result)
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
        let result = self.cli.edit_message(
            &message_id,
            &new_text,
            chat_id.as_deref(),
        ).await;
        
        convert_bridge_result(result)
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
        let result = self.cli.delete_message(
            &message_id,
            chat_id.as_deref(),
        ).await;
        
        convert_bridge_result(result)
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
        let result = self.cli.pin_message(
            &message_id,
            chat_id.as_deref(),
        ).await;
        
        convert_bridge_result(result)
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
        let result = self.cli.unpin_message(
            &message_id,
            chat_id.as_deref(),
        ).await;
        
        convert_bridge_result(result)
    }

    // === Chat Management Commands ===
    
    #[tool(description = "Get information about the chat")]
    async fn chat_info(
        &self,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
    ) -> MCPResult {
        let result = self.cli.get_chat_info(chat_id.as_deref()).await;
        convert_bridge_result(result)
    }
    
    #[tool(description = "Get user profile information")]
    async fn get_profile(
        &self,
        #[tool(param)]
        #[schemars(description = "User ID to get profile for")]
        user_id: String,
    ) -> MCPResult {
        let result = self.cli.get_profile(&user_id).await;
        convert_bridge_result(result)
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
        let result = self.cli.get_chat_members(
            chat_id.as_deref(),
            cursor.as_deref(),
        ).await;
        convert_bridge_result(result)
    }
    
    #[tool(description = "Get chat administrators")]
    async fn get_chat_admins(
        &self,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
    ) -> MCPResult {
        let result = self.cli.get_chat_admins(chat_id.as_deref()).await;
        convert_bridge_result(result)
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
        let result = self.cli.set_chat_title(
            &title,
            chat_id.as_deref(),
        ).await;
        convert_bridge_result(result)
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
        let result = self.cli.set_chat_about(
            &about,
            chat_id.as_deref(),
        ).await;
        convert_bridge_result(result)
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
        let result = self.cli.send_action(
            &action,
            chat_id.as_deref(),
        ).await;
        convert_bridge_result(result)
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
        let result = self.cli.upload_file_base64(
            &file_name,
            &base64_content,
            chat_id.as_deref(),
            caption.as_deref(),
            reply_msg_id.as_deref(),
        ).await;
        convert_bridge_result(result)
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
        let result = self.cli.upload_text_file(
            &file_name,
            &content,
            chat_id.as_deref(),
            caption.as_deref(),
        ).await;
        convert_bridge_result(result)
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
        let result = self.cli.upload_json_file(
            &file_name,
            &json_data,
            pretty.unwrap_or(true),
            chat_id.as_deref(),
            caption.as_deref(),
        ).await;
        convert_bridge_result(result)
    }
    
    #[tool(description = "Get file information")]
    async fn file_info(
        &self,
        #[tool(param)]
        #[schemars(description = "File ID to get information about")]
        file_id: String,
    ) -> MCPResult {
        let result = self.cli.get_file_info(&file_id).await;
        convert_bridge_result(result)
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
        let result = self.cli.search_semantic(
            &query,
            chat_id.as_deref(),
            limit,
        ).await;
        convert_bridge_result(result)
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
        let result = self.cli.search_text(
            &query,
            chat_id.as_deref(),
            limit,
        ).await;
        convert_bridge_result(result)
    }
    
    #[tool(description = "Get database statistics")]
    async fn get_database_stats(
        &self,
        #[tool(param)]
        #[schemars(description = "Chat ID for specific chat stats (optional, all chats if not provided)")]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Date since when to count (optional)")]
        since: Option<String>,
    ) -> MCPResult {
        let result = self.cli.get_database_stats(
            chat_id.as_deref(),
            since.as_deref(),
        ).await;
        convert_bridge_result(result)
    }
    
    #[tool(description = "Get conversation context")]
    async fn get_context(
        &self,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, uses default if not provided)")]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Context type: 'recent', 'summary', or 'keywords' (default: 'recent')")]
        context_type: Option<String>,
        #[tool(param)]
        #[schemars(description = "Timeframe for context (optional)")]
        timeframe: Option<String>,
    ) -> MCPResult {
        let result = self.cli.get_context(
            chat_id.as_deref(),
            context_type.as_deref(),
            timeframe.as_deref(),
        ).await;
        convert_bridge_result(result)
    }

    // === Diagnostic Commands ===
    
    #[tool(description = "Get information about the bot")]
    async fn self_get(&self) -> MCPResult {
        let result = self.cli.get_self().await;
        convert_bridge_result(result)
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
        let result = self.cli.get_events(
            last_event_id.as_deref(),
            poll_time,
        ).await;
        convert_bridge_result(result)
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
        events_get
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_bridge_result_success() {
        let json_val = serde_json::json!({"success": true, "data": "test"});
        let result = convert_bridge_result(Ok(json_val));
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_bridge_result_error() {
        let error = BridgeError::CliError("test error".to_string());
        let result = convert_bridge_result(Err(error));
        assert!(result.is_err());
    }
}