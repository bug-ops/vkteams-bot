//! MCP CLI Bridge trait for direct MCP result conversion
//!
//! This trait provides methods that return MCPResult directly,
//! avoiding intermediate conversions and simplifying the server code.

use crate::server::MCPResult;
use async_trait::async_trait;

/// Trait for CLI bridge operations that return MCP-compatible results
#[async_trait]
pub trait McpCliBridge {
    // === Messaging Commands ===

    /// Send text message to chat
    async fn send_text_mcp(
        &self,
        text: &str,
        chat_id: Option<&str>,
        reply_msg_id: Option<&str>,
    ) -> MCPResult;

    /// Send file from file path
    async fn send_file_mcp(
        &self,
        file_path: &str,
        chat_id: Option<&str>,
        caption: Option<&str>,
    ) -> MCPResult;

    /// Send voice message from file path
    async fn send_voice_mcp(&self, file_path: &str, chat_id: Option<&str>) -> MCPResult;

    /// Edit existing message
    async fn edit_message_mcp(
        &self,
        message_id: &str,
        new_text: &str,
        chat_id: Option<&str>,
    ) -> MCPResult;

    /// Delete message
    async fn delete_message_mcp(&self, message_id: &str, chat_id: Option<&str>) -> MCPResult;

    /// Pin message in chat
    async fn pin_message_mcp(&self, message_id: &str, chat_id: Option<&str>) -> MCPResult;

    /// Unpin message from chat
    async fn unpin_message_mcp(&self, message_id: &str, chat_id: Option<&str>) -> MCPResult;

    // === Chat Management Commands ===

    /// Get chat information
    async fn get_chat_info_mcp(&self, chat_id: Option<&str>) -> MCPResult;

    /// Get user profile
    async fn get_profile_mcp(&self, user_id: &str) -> MCPResult;

    /// Get chat members with optional cursor
    async fn get_chat_members_mcp(
        &self,
        chat_id: Option<&str>,
        cursor: Option<&str>,
    ) -> MCPResult;

    /// Get chat administrators
    async fn get_chat_admins_mcp(&self, chat_id: Option<&str>) -> MCPResult;

    /// Set chat title
    async fn set_chat_title_mcp(&self, title: &str, chat_id: Option<&str>) -> MCPResult;

    /// Set chat description
    async fn set_chat_about_mcp(&self, about: &str, chat_id: Option<&str>) -> MCPResult;

    /// Send chat action (typing/looking)
    async fn send_action_mcp(&self, action: &str, chat_id: Option<&str>) -> MCPResult;

    // === File Upload Commands ===

    /// Upload file from base64 content
    async fn upload_file_base64_mcp(
        &self,
        name: &str,
        content_base64: &str,
        chat_id: Option<&str>,
        caption: Option<&str>,
        reply_msg_id: Option<&str>,
    ) -> MCPResult;

    /// Upload text content as file
    async fn upload_text_file_mcp(
        &self,
        name: &str,
        content: &str,
        chat_id: Option<&str>,
        caption: Option<&str>,
    ) -> MCPResult;

    /// Upload JSON data as file
    async fn upload_json_file_mcp(
        &self,
        name: &str,
        json_data: &str,
        pretty: bool,
        chat_id: Option<&str>,
        caption: Option<&str>,
    ) -> MCPResult;

    /// Get file information
    async fn get_file_info_mcp(&self, file_id: &str) -> MCPResult;

    // === Storage Commands ===

    /// Get database statistics
    async fn get_database_stats_mcp(
        &self,
        chat_id: Option<&str>,
        since: Option<&str>,
    ) -> MCPResult;

    /// Search messages using semantic similarity
    async fn search_semantic_mcp(
        &self,
        query: &str,
        chat_id: Option<&str>,
        limit: Option<usize>,
    ) -> MCPResult;

    /// Search messages using text search
    async fn search_text_mcp(
        &self,
        query: &str,
        chat_id: Option<&str>,
        limit: Option<i64>,
    ) -> MCPResult;

    /// Get conversation context
    async fn get_context_mcp(
        &self,
        chat_id: Option<&str>,
        context_type: Option<&str>,
        timeframe: Option<&str>,
    ) -> MCPResult;

    // === Daemon Management Commands ===

    /// Get daemon status and statistics
    async fn get_daemon_status_mcp(&self) -> MCPResult;

    /// Get recent messages from storage
    async fn get_recent_messages_mcp(
        &self,
        chat_id: Option<&str>,
        limit: Option<usize>,
        since: Option<&str>,
    ) -> MCPResult;

    // === Diagnostic Commands ===

    /// Get bot information and status
    async fn get_self_mcp(&self) -> MCPResult;

    /// Get events
    async fn get_events_mcp(
        &self,
        last_event_id: Option<&str>,
        poll_time: Option<u64>,
    ) -> MCPResult;
}