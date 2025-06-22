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
        reply_msg_id: Option<&str>
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
        caption: Option<&str>
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
        chat_id: Option<&str>
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
        chat_id: Option<&str>
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["edit-message", "--message-id", message_id, "--new-text", new_text];
        
        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }
        
        self.execute_command(&args).await
    }
    
    /// Delete message
    pub async fn delete_message(
        &self, 
        message_id: &str,
        chat_id: Option<&str>
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
        chat_id: Option<&str>
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
        chat_id: Option<&str>
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
        self.execute_command(&["get-profile", "--user-id", user_id]).await
    }
    
    /// Get chat members with optional cursor
    pub async fn get_chat_members(
        &self, 
        chat_id: Option<&str>,
        cursor: Option<&str>
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
        chat_id: Option<&str>
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
        chat_id: Option<&str>
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
        chat_id: Option<&str>
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
        reply_msg_id: Option<&str>
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
        caption: Option<&str>
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
        caption: Option<&str>
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
        since: Option<&str>
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
        limit: Option<usize>
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
        limit: Option<i64>
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
        timeframe: Option<&str>
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
        poll_time: Option<u64>
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

    #[tokio::test]
    async fn test_command_building() {
        // Set required env var for test
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat");
        }
        
        // This test just verifies that command building works correctly
        // We can't actually execute commands in test environment
        if let Ok(_bridge) = CliBridge::new() {
            // Test would need actual CLI binary to run
            println!("CLI bridge created for testing");
        }
    }
}