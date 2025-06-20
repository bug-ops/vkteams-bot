use crate::errors::McpError;
use crate::types::Server;
use rmcp::tool_box;
use rmcp::{
    ServerHandler,
    model::{CallToolResult, Content, ErrorData, ServerCapabilities, ServerInfo},
    tool,
};
use serde::Serialize;
use std::result::Result;
use vkteams_bot::prelude::BotRequest;
use vkteams_bot::prelude::*;

use crate::file_utils::{
    format_file_size, get_mime_type, sanitize_filename, validate_file_size, validate_filename,
};
use base64::Engine;

pub trait IntoCallToolResult<T>
where
    T: Serialize,
{
    fn into_mcp_result(self) -> Result<CallToolResult, ErrorData>;
}

impl<T> IntoCallToolResult<T> for std::result::Result<T, McpError>
where
    T: Serialize,
{
    fn into_mcp_result(self) -> Result<CallToolResult, ErrorData> {
        match self {
            Ok(response) => convert_result(response).map_err(ErrorData::from),
            Err(e) => Err(e.into()),
        }
    }
}
fn convert_result<T>(response: T) -> Result<CallToolResult, McpError>
where
    T: Serialize,
{
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string(&response)?,
    )]))
}

impl ServerHandler for Server {
    tool_box!(@derive);
    fn get_info(&self) -> ServerInfo {
        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_prompts()
            .build();
        ServerInfo {
            capabilities,
            instructions: Some(r#"VKTeams MCP Server — a server for managing a VK Teams bot via MCP (Model Context Protocol).
        Tools:
            - Send text messages to chat (send_text)
            - Get bot information (self_get)
            - Get chat information (chat_info)
            - Get file information (file_info)
            - Get events (events_get)
            - Send files and voice messages (send_file, send_voice)
            - Edit, delete, pin, and unpin messages (edit_message, delete_message, pin_message, unpin_message)
            - Get chat members, admins, pending and blocked users (get_chat_members, get_chat_admins, get_chat_pending_users, get_chat_blocked_users)
            - Set chat title, about, rules, and avatar (set_chat_title, set_chat_about, set_chat_rules, set_chat_avatar)
            - Send chat actions (typing/looking) (send_action)
            - Block, unblock, delete, and resolve pending users (block_user, unblock_user, delete_chat_members, resolve_pending)
            - Enhanced file uploads (upload_file_from_base64, upload_text_as_file, upload_json_file, upload_multiple_files)
            "#.into()),
            ..Default::default()
        }
    }
}

pub type MCPResult = Result<CallToolResult, ErrorData>;

impl Server {
    #[tool(description = "Get information about the bot")]
    async fn self_get(&self) -> MCPResult {
        let req = RequestSelfGet::default();
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }
    #[tool(description = "Send text message to chat")]
    async fn send_text(
        &self,
        #[tool(param)]
        #[schemars(description = r#"New text.
        You must use the formatting template (HTML):
            <b>bold</b>, <strong>bold</strong>
            <i>italic</i>, <em>italic</em>
            <u>underline</u>, <ins>underline</ins>
            <s>strikethrough</s>, <strike>strikethrough</strike>, <del>strikethrough</del>
            <a href="http://www.example.com/">inline URL</a>
            <a>@[{chat_id}]</a> - inline mention of a user where {chat_id} is the chat ID of the user
            <code>inline fixed-width code</code>
            <pre>pre-formatted fixed-width code block</pre>
            <pre><code class="python">pre-formatted fixed-width code block written in the Python programming language</code></pre>
            Ordered list:
            <ol>
                <li>First element</li>
                <li>Second element</li>
            </ol>
            Unordered list:
            <ul>
                <li>First element</li>
                <li>Second element</li>
            </ul>
            Quote:
            <blockquote>
                Begin of quote.
                End of quote.
            </blockquote>
        "#)]
        text: String,
        #[tool(param)]
        #[schemars(description = "Reply to message ID (optional)")]
        reply_msg_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Forward message ID (optional)")]
        forward_msg_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Forward from chat ID (optional)")]
        forward_from_chat_id: Option<String>,
    ) -> MCPResult {
        let mut req = RequestMessagesSendText::new(ChatId::from_borrowed_str(self.chat_id.as_str()))
            .with_text(text)
            .with_parse_mode(ParseMode::HTML);
        if let Some(reply_to_msg_id) = reply_msg_id {
            req = req.with_reply_msg_id(MsgId(reply_to_msg_id));
        }
        if let Some(forward_msg_id) = forward_msg_id {
            req = req.with_forward_msg_id(MsgId(forward_msg_id));
        }
        if let Some(forward_from_chat_id) = forward_from_chat_id {
            req = req.with_forward_chat_id(ChatId::from_borrowed_str(&forward_from_chat_id));
        }
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }
    #[tool(description = "Get information about the chat")]
    async fn chat_info(&self) -> MCPResult {
        let req = RequestChatsGetInfo::new(ChatId::from_borrowed_str(self.chat_id.as_str()));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }
    #[tool(description = "Get information about the file")]
    async fn file_info(
        &self,
        #[tool(param)]
        #[schemars(description = "File ID to get information")]
        file_id: String,
    ) -> MCPResult {
        let req = RequestFilesGetInfo::new(FileId(file_id));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }
    #[tool(description = "Get events from the last event_id")]
    async fn events_get(
        &self,
        #[tool(param)]
        #[schemars(description = "Last event ID to get events")]
        last_event_id: Option<u32>,
    ) -> MCPResult {
        let req = RequestEventsGet::new(last_event_id.unwrap_or(0)).with_poll_time(30);
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }
    #[tool(description = "Send file to chat")]
    #[allow(clippy::too_many_arguments)]
    async fn send_file(
        &self,
        #[tool(param)]
        #[schemars(description = "File name")]
        file_name: String,
        #[tool(param)]
        #[schemars(description = "File content")]
        file_content: Vec<u8>,
        #[tool(param)]
        #[schemars(description = "Text message (optional)")]
        text: Option<String>,
        #[tool(param)]
        #[schemars(description = "Reply to message ID (optional)")]
        reply_msg_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Forward from chat ID (optional)")]
        forward_chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Forward message ID (optional)")]
        forward_msg_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Inline keyboard markup (optional)")]
        inline_keyboard_markup: Option<String>,
    ) -> MCPResult {
        let mut req = RequestMessagesSendFile::new((
            ChatId::from_borrowed_str(self.chat_id.as_str()),
            MultipartName::FileContent {
                filename: file_name.clone(),
                content: file_content.clone(),
            },
        ));
        if let Some(text) = text {
            req = req.with_text(text);
        }
        if let Some(reply_msg_id) = reply_msg_id {
            req = req.with_reply_msg_id(MsgId(reply_msg_id));
        }
        if let Some(forward_chat_id) = forward_chat_id {
            req = req.with_forward_chat_id(ChatId::from_borrowed_str(&forward_chat_id));
        }
        if let Some(forward_msg_id) = forward_msg_id {
            req = req.with_forward_msg_id(MsgId(forward_msg_id));
        }
        if let Some(inline_keyboard_markup) = inline_keyboard_markup {
            req = req.with_inline_keyboard_markup(inline_keyboard_markup);
        }
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Send voice message to chat")]
    async fn send_voice(
        &self,
        #[tool(param)]
        #[schemars(description = "File name")]
        file_name: String,
        #[tool(param)]
        #[schemars(description = "File content")]
        file_content: Vec<u8>,
    ) -> MCPResult {
        let req = RequestMessagesSendVoice::new((
            ChatId::from_borrowed_str(self.chat_id.as_str()),
            MultipartName::FileContent {
                filename: file_name.clone(),
                content: file_content.clone(),
            },
        ));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Edit message in chat")]
    async fn edit_message(
        &self,
        #[tool(param)]
        #[schemars(description = "Message ID")]
        msg_id: String,
        #[tool(param)]
        #[schemars(description = r#"New text.
        You must use the formatting template (HTML):
            <b>bold</b>, <strong>bold</strong>
            <i>italic</i>, <em>italic</em>
            <u>underline</u>, <ins>underline</ins>
            <s>strikethrough</s>, <strike>strikethrough</strike>, <del>strikethrough</del>
            <a href="http://www.example.com/">inline URL</a>
            <a>@[{chat_id}]</a> - inline mention of a user where {chat_id} is the chat ID of the user
            <code>inline fixed-width code</code>
            <pre>pre-formatted fixed-width code block</pre>
            <pre><code class="python">pre-formatted fixed-width code block written in the Python programming language</code></pre>
            Ordered list:
            <ol>
                <li>First element</li>
                <li>Second element</li>
            </ol>
            Unordered list:
            <ul>
                <li>First element</li>
                <li>Second element</li>
            </ul>
            Quote:
            <blockquote>
                Begin of quote.
                End of quote.
            </blockquote>
        "#)]
        text: String,
    ) -> MCPResult {
        let req = RequestMessagesEditText::new((ChatId::from_borrowed_str(self.chat_id.as_str()), MsgId(msg_id)))
            .with_text(text)
            .with_parse_mode(ParseMode::HTML);
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Delete message from chat")]
    async fn delete_message(
        &self,
        #[tool(param)]
        #[schemars(description = "Message ID")]
        msg_id: String,
    ) -> MCPResult {
        let req = RequestMessagesDeleteMessages::new((ChatId::from_borrowed_str(self.chat_id.as_str()), MsgId(msg_id)));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Pin message in chat")]
    async fn pin_message(
        &self,
        #[tool(param)]
        #[schemars(description = "Message ID")]
        msg_id: String,
    ) -> MCPResult {
        let req = RequestChatsPinMessage::new((ChatId::from_borrowed_str(self.chat_id.as_str()), MsgId(msg_id)));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Unpin message in chat")]
    async fn unpin_message(
        &self,
        #[tool(param)]
        #[schemars(description = "Message ID")]
        msg_id: String,
    ) -> MCPResult {
        let req = RequestChatsUnpinMessage::new((ChatId::from_borrowed_str(self.chat_id.as_str()), MsgId(msg_id)));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Get chat members")]
    async fn get_chat_members(
        &self,
        #[tool(param)]
        #[schemars(description = "Cursor for pagination")]
        cursor: Option<u32>,
    ) -> MCPResult {
        let mut req = RequestChatsGetMembers::new(ChatId::from_borrowed_str(self.chat_id.as_str()));
        if let Some(cursor) = cursor {
            req = req.with_cursor(cursor);
        }
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Get chat admins")]
    async fn get_chat_admins(&self) -> MCPResult {
        let req = RequestChatsGetAdmins::new(ChatId::from_borrowed_str(self.chat_id.as_str()));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Get pending users in chat")]
    async fn get_chat_pending_users(&self) -> MCPResult {
        let req = RequestChatsGetPendingUsers::new(ChatId::from_borrowed_str(self.chat_id.as_str()));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Get blocked users in chat")]
    async fn get_chat_blocked_users(&self) -> MCPResult {
        let req = RequestChatsGetBlockedUsers::new(ChatId::from_borrowed_str(self.chat_id.as_str()));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Set chat title")]
    async fn set_chat_title(
        &self,
        #[tool(param)]
        #[schemars(description = "New chat title")]
        title: String,
    ) -> MCPResult {
        let req = RequestChatsSetTitle::new((ChatId::from_borrowed_str(self.chat_id.as_str()), title));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Set chat about/description")]
    async fn set_chat_about(
        &self,
        #[tool(param)]
        #[schemars(description = "New chat about/description")]
        about: String,
    ) -> MCPResult {
        let req = RequestChatsSetAbout::new((ChatId::from_borrowed_str(self.chat_id.as_str()), about));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Set chat rules")]
    async fn set_chat_rules(
        &self,
        #[tool(param)]
        #[schemars(description = "New chat rules")]
        rules: String,
    ) -> MCPResult {
        let req = RequestChatsSetRules::new((ChatId::from_borrowed_str(self.chat_id.as_str()), rules));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Send chat action (typing/looking)")]
    async fn send_action(
        &self,
        #[tool(param)]
        #[schemars(description = "Action (Typing/Looking)")]
        action: String,
    ) -> MCPResult {
        let chat_action = match action.to_lowercase().as_str() {
            "typing" => ChatActions::Typing,
            "looking" => ChatActions::Looking,
            _ => return Err(McpError::Other("Unknown action".to_string()).into()),
        };
        let req = RequestChatsSendAction::new((ChatId::from_borrowed_str(self.chat_id.as_str()), chat_action));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Block user in chat")]
    async fn block_user(
        &self,
        #[tool(param)]
        #[schemars(description = "User ID")]
        user_id: String,
        #[tool(param)]
        #[schemars(description = "Delete last messages")]
        del_last_messages: Option<bool>,
    ) -> MCPResult {
        let mut req = RequestChatsBlockUser::new((ChatId::from_borrowed_str(self.chat_id.as_str()), UserId(user_id)));
        if let Some(del) = del_last_messages {
            req = req.with_del_last_messages(del);
        }
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Unblock user in chat")]
    async fn unblock_user(
        &self,
        #[tool(param)]
        #[schemars(description = "User ID")]
        user_id: String,
    ) -> MCPResult {
        let req = RequestChatsUnblockUser::new((ChatId::from_borrowed_str(self.chat_id.as_str()), UserId(user_id)));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Resolve pendings in chat")]
    async fn resolve_pending(
        &self,
        #[tool(param)]
        #[schemars(description = "Approve (true/false)")]
        approve: bool,
        #[tool(param)]
        #[schemars(description = "User ID (optional)")]
        user_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Everyone (optional)")]
        everyone: Option<bool>,
    ) -> MCPResult {
        let mut req = RequestChatsResolvePending::new((ChatId::from_borrowed_str(self.chat_id.as_str()), approve));
        if let Some(uid) = user_id {
            req = req.with_user_id(UserId(uid));
        }
        if let Some(everyone) = everyone {
            req = req.with_everyone(everyone);
        }
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Set chat avatar")]
    async fn set_chat_avatar(
        &self,
        #[tool(param)]
        #[schemars(description = "File name")]
        file_name: String,
        #[tool(param)]
        #[schemars(description = "File content")]
        file_content: Vec<u8>,
    ) -> MCPResult {
        let req = RequestChatsAvatarSet::new((
            ChatId::from_borrowed_str(self.chat_id.as_str()),
            MultipartName::ImageContent {
                filename: file_name.clone(),
                content: file_content.clone(),
            },
        ));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Delete members from chat")]
    async fn delete_chat_members(
        &self,
        #[tool(param)]
        #[schemars(description = "User ID")]
        user_id: String,
        #[tool(param)]
        #[schemars(description = "Members (comma separated)")]
        members: String,
    ) -> MCPResult {
        let members_vec: Vec<Sn> = members
            .split(',')
            .map(|s| {
                let id = s.trim().to_string();
                Sn {
                    sn: id.clone(),
                    user_id: UserId(id),
                }
            })
            .collect();
        let req = RequestChatsMembersDelete::new((
            ChatId::from_borrowed_str(self.chat_id.as_str()),
            UserId(user_id),
            members_vec,
        ));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    // === Database and History Tools ===

    #[tool(description = "Search messages by content using full-text search")]
    async fn search_messages(
        &self,
        #[tool(param)]
        #[schemars(description = "Search query")]
        query: String,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, defaults to current chat)")]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Maximum number of results (default: 20)")]
        limit: Option<i32>,
    ) -> MCPResult {
        let chat = chat_id.unwrap_or_else(|| self.chat_id.clone());
        let limit = limit.unwrap_or(20);

        match self.event_processor.db.search_messages(&chat, &query, limit).await {
            Ok(messages) => {
                let result = serde_json::json!({
                    "messages": messages,
                    "query": query,
                    "chat_id": chat,
                    "count": messages.len()
                });
                convert_result(result).map_err(ErrorData::from)
            }
            Err(e) => Err(ErrorData::internal_error(format!("Search failed: {}", e), None)),
        }
    }

    #[tool(description = "Get conversation history for context")]
    async fn get_conversation_context(
        &self,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, defaults to current chat)")]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Hours back to look (default: 24)")]
        hours_back: Option<u64>,
        #[tool(param)]
        #[schemars(description = "Maximum number of messages (default: 100)")]
        message_limit: Option<i32>,
    ) -> MCPResult {
        let chat = chat_id.unwrap_or_else(|| self.chat_id.clone());
        let hours = hours_back.unwrap_or(24);
        let limit = message_limit.unwrap_or(100);

        match self.event_processor.db.get_conversation_context(&chat, hours, limit).await {
            Ok(messages) => {
                let result = serde_json::json!({
                    "messages": messages,
                    "chat_id": chat,
                    "period_hours": hours,
                    "count": messages.len()
                });
                convert_result(result).map_err(ErrorData::from)
            }
            Err(e) => Err(ErrorData::internal_error(format!("Failed to get conversation context: {}", e), None)),
        }
    }

    #[tool(description = "Get event analytics and statistics")]
    async fn get_event_analytics(
        &self,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, defaults to current chat)")]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Days back to analyze (default: 7)")]
        days_back: Option<u64>,
    ) -> MCPResult {
        let chat = chat_id.unwrap_or_else(|| self.chat_id.clone());
        let days = days_back.unwrap_or(7);

        match self.event_processor.db.get_event_analytics(&chat, days).await {
            Ok(analytics) => {
                let result = serde_json::json!({
                    "analytics": analytics,
                    "chat_id": chat,
                    "period_days": days
                });
                convert_result(result).map_err(ErrorData::from)
            }
            Err(e) => Err(ErrorData::internal_error(format!("Analytics query failed: {}", e), None)),
        }
    }

    #[tool(description = "Process and store new events manually")]
    async fn process_events(
        &self,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, defaults to current chat)")]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Last event ID to start from (optional)")]
        last_event_id: Option<u32>,
    ) -> MCPResult {
        let chat = chat_id.unwrap_or_else(|| self.chat_id.clone());
        let last_id = if let Some(id) = last_event_id {
            id
        } else {
            // Получаем последний обработанный event_id из БД
            match self.event_processor.get_latest_event_id(&chat).await {
                Ok(id) => id,
                Err(e) => {
                    return Err(ErrorData::internal_error(format!("Failed to get latest event ID: {}", e), None));
                }
            }
        };

        // Получаем новые события
        let req = RequestEventsGet::new(last_id).with_poll_time(5); // Короткий таймаут для ручной обработки
        match self.client().send_api_request(req).await {
            Ok(events_response) => {
                let mut processed_count = 0;
                let mut errors = Vec::new();

                for event in &events_response.events {
                    match self.event_processor.process_event(event, &chat).await {
                        Ok(()) => processed_count += 1,
                        Err(e) => {
                            errors.push(format!("Event {}: {}", event.event_id, e));
                        }
                    }
                }

                let result = serde_json::json!({
                    "processed_events": processed_count,
                    "total_events": events_response.events.len(),
                    "errors": errors,
                    "chat_id": chat,
                    "last_event_id": events_response.events.last().map(|e| e.event_id).unwrap_or(last_id)
                });

                convert_result(result).map_err(ErrorData::from)
            }
            Err(e) => Err(ErrorData::internal_error(format!("Failed to fetch events: {}", e), None)),
        }
    }

    #[tool(description = "Get recent events from database")]
    async fn get_recent_events(
        &self,
        #[tool(param)]
        #[schemars(description = "Chat ID (optional, defaults to current chat)")]
        chat_id: Option<String>,
        #[tool(param)]
        #[schemars(description = "Hours back to look (default: 24)")]
        hours_back: Option<u64>,
        #[tool(param)]
        #[schemars(description = "Maximum number of events (default: 50)")]
        limit: Option<i32>,
    ) -> MCPResult {
        let chat = chat_id.unwrap_or_else(|| self.chat_id.clone());
        let hours = hours_back.unwrap_or(24);
        let limit = limit.unwrap_or(50);

        let since = chrono::Utc::now() - chrono::Duration::hours(hours as i64);

        match self.event_processor.db.get_events_since(&chat, since, limit).await {
            Ok(events) => {
                let result = serde_json::json!({
                    "events": events,
                    "chat_id": chat,
                    "period_hours": hours,
                    "count": events.len()
                });
                convert_result(result).map_err(ErrorData::from)
            }
            Err(e) => Err(ErrorData::internal_error(format!("Failed to get recent events: {}", e), None)),
        }
    }

    tool_box!(Server {
        self_get,
        send_text,
        chat_info,
        file_info,
        events_get,
        send_file,
        send_voice,
        edit_message,
        delete_message,
        pin_message,
        unpin_message,
        get_chat_members,
        get_chat_admins,
        get_chat_pending_users,
        get_chat_blocked_users,
        set_chat_title,
        set_chat_about,
        set_chat_rules,
        send_action,
        block_user,
        unblock_user,
        resolve_pending,
        set_chat_avatar,
        delete_chat_members,
        search_messages,
        get_conversation_context,
        get_event_analytics,
        process_events,
        get_recent_events,
    });
}
