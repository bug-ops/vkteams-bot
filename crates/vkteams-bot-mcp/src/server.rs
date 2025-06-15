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
use vkteams_bot::prelude::ParseMode;
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
        let mut req = RequestMessagesSendText::new(ChatId(self.chat_id.clone()))
            .with_text(text)
            .with_parse_mode(ParseMode::HTML);
        if let Some(reply_to_msg_id) = reply_msg_id {
            req = req.with_reply_msg_id(MsgId(reply_to_msg_id));
        }
        if let Some(forward_msg_id) = forward_msg_id {
            req = req.with_forward_msg_id(MsgId(forward_msg_id));
        }
        if let Some(forward_from_chat_id) = forward_from_chat_id {
            req = req.with_forward_chat_id(ChatId(forward_from_chat_id));
        }
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }
    #[tool(description = "Get information about the chat")]
    async fn chat_info(&self) -> MCPResult {
        let req = RequestChatsGetInfo::new(ChatId(self.chat_id.clone()));
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
            ChatId(self.chat_id.clone()),
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
            req = req.with_forward_chat_id(ChatId(forward_chat_id));
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
            ChatId(self.chat_id.clone()),
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
        let req = RequestMessagesEditText::new((ChatId(self.chat_id.clone()), MsgId(msg_id)))
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
        let req = RequestMessagesDeleteMessages::new((ChatId(self.chat_id.clone()), MsgId(msg_id)));
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
        let req = RequestChatsPinMessage::new((ChatId(self.chat_id.clone()), MsgId(msg_id)));
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
        let req = RequestChatsUnpinMessage::new((ChatId(self.chat_id.clone()), MsgId(msg_id)));
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
        let mut req = RequestChatsGetMembers::new(ChatId(self.chat_id.clone()));
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
        let req = RequestChatsGetAdmins::new(ChatId(self.chat_id.clone()));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Get pending users in chat")]
    async fn get_chat_pending_users(&self) -> MCPResult {
        let req = RequestChatsGetPendingUsers::new(ChatId(self.chat_id.clone()));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Get blocked users in chat")]
    async fn get_chat_blocked_users(&self) -> MCPResult {
        let req = RequestChatsGetBlockedUsers::new(ChatId(self.chat_id.clone()));
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
        let req = RequestChatsSetTitle::new((ChatId(self.chat_id.clone()), title));
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
        let req = RequestChatsSetAbout::new((ChatId(self.chat_id.clone()), about));
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
        let req = RequestChatsSetRules::new((ChatId(self.chat_id.clone()), rules));
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
        let req = RequestChatsSendAction::new((ChatId(self.chat_id.clone()), chat_action));
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
        let mut req = RequestChatsBlockUser::new((ChatId(self.chat_id.clone()), UserId(user_id)));
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
        let req = RequestChatsUnblockUser::new((ChatId(self.chat_id.clone()), UserId(user_id)));
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
        let mut req = RequestChatsResolvePending::new((ChatId(self.chat_id.clone()), approve));
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
            ChatId(self.chat_id.clone()),
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
            ChatId(self.chat_id.clone()),
            UserId(user_id),
            members_vec,
        ));
        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Upload file from base64 encoded content")]
    async fn upload_file_from_base64(
        &self,
        #[tool(param)]
        #[schemars(description = "File name with extension")]
        file_name: String,
        #[tool(param)]
        #[schemars(description = "Base64 encoded file content")]
        base64_content: String,
        #[tool(param)]
        #[schemars(description = "Optional caption/text message")]
        caption: Option<String>,
        #[tool(param)]
        #[schemars(description = "Reply to message ID (optional)")]
        reply_msg_id: Option<String>,
    ) -> MCPResult {
        // Validate filename
        validate_filename(&file_name).or_else(|_| {
            let sanitized = sanitize_filename(&file_name);
            validate_filename(&sanitized)
        })?;

        // Decode base64 content
        let file_content = base64::engine::general_purpose::STANDARD
            .decode(&base64_content)
            .map_err(|e| McpError::Other(format!("Invalid base64 content: {}", e)))?;

        // Validate file size
        validate_file_size(&file_content)?;

        let mut req = RequestMessagesSendFile::new((
            ChatId(self.chat_id.clone()),
            MultipartName::FileContent {
                filename: file_name.clone(),
                content: file_content,
            },
        ));

        if let Some(caption) = caption {
            req = req.with_text(caption);
        }
        if let Some(reply_msg_id) = reply_msg_id {
            req = req.with_reply_msg_id(MsgId(reply_msg_id));
        }

        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Upload text content as a file")]
    async fn upload_text_as_file(
        &self,
        #[tool(param)]
        #[schemars(description = "File name with extension (e.g., 'code.py', 'log.txt')")]
        file_name: String,
        #[tool(param)]
        #[schemars(description = "Text content to save as file")]
        text_content: String,
        #[tool(param)]
        #[schemars(description = "Optional caption/description")]
        caption: Option<String>,
        #[tool(param)]
        #[schemars(description = "Reply to message ID (optional)")]
        reply_msg_id: Option<String>,
    ) -> MCPResult {
        // Validate filename
        validate_filename(&file_name).or_else(|_| {
            let sanitized = sanitize_filename(&file_name);
            validate_filename(&sanitized)
        })?;

        let file_content = text_content.into_bytes();

        // Validate file size
        validate_file_size(&file_content)?;

        let mut req = RequestMessagesSendFile::new((
            ChatId(self.chat_id.clone()),
            MultipartName::FileContent {
                filename: file_name.clone(),
                content: file_content,
            },
        ));

        if let Some(caption) = caption {
            req = req.with_text(caption);
        }
        if let Some(reply_msg_id) = reply_msg_id {
            req = req.with_reply_msg_id(MsgId(reply_msg_id));
        }

        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Create and upload a JSON file from structured data")]
    async fn upload_json_file(
        &self,
        #[tool(param)]
        #[schemars(description = "File name (mandatory)")]
        file_name: String,
        #[tool(param)]
        #[schemars(description = "JSON data as string")]
        json_data: String,
        #[tool(param)]
        #[schemars(description = "Pretty print JSON (default: true)")]
        pretty_print: Option<bool>,
        #[tool(param)]
        #[schemars(description = "Optional caption/description")]
        caption: Option<String>,
        #[tool(param)]
        #[schemars(description = "Reply to message ID (optional)")]
        reply_msg_id: Option<String>,
    ) -> MCPResult {
        // Parse and optionally reformat JSON
        let json_value: serde_json::Value = serde_json::from_str(&json_data)
            .map_err(|e| McpError::Other(format!("Invalid JSON data: {}", e)))?;

        let formatted_json = if pretty_print.unwrap_or(true) {
            serde_json::to_string_pretty(&json_value)
        } else {
            serde_json::to_string(&json_value)
        }
        .map_err(|e| McpError::Other(format!("Failed to serialize JSON: {}", e)))?;

        // Ensure .json extension
        let final_filename = if file_name.ends_with(".json") {
            file_name
        } else {
            format!("{}.json", file_name)
        };

        let mut req = RequestMessagesSendFile::new((
            ChatId(self.chat_id.clone()),
            MultipartName::FileContent {
                filename: final_filename,
                content: formatted_json.into_bytes(),
            },
        ));

        if let Some(caption) = caption {
            req = req.with_text(caption);
        }
        if let Some(reply_msg_id) = reply_msg_id {
            req = req.with_reply_msg_id(MsgId(reply_msg_id));
        }

        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Upload multiple files at once")]
    async fn upload_multiple_files(
        &self,
        #[tool(param)]
        #[schemars(
            description = "JSON array of file objects with 'name' and 'content' (base64) fields"
        )]
        files_json: String,
        #[tool(param)]
        #[schemars(description = "Optional caption for the file batch")]
        caption: Option<String>,
    ) -> MCPResult {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct FileUpload {
            name: String,
            content: String, // base64 encoded
        }

        let files: Vec<FileUpload> = serde_json::from_str(&files_json)
            .map_err(|e| McpError::Other(format!("Invalid files JSON: {}", e)))?;

        if files.is_empty() {
            return Err(McpError::Other("No files provided".to_string()).into());
        }

        if files.len() > 10 {
            return Err(McpError::Other("Too many files (max 10 allowed)".to_string()).into());
        }

        let mut results = Vec::new();

        for file in files {
            // Validate filename
            #[allow(clippy::redundant_pattern_matching)]
            if let Err(_) = validate_filename(&file.name) {
                results.push(format!("❌ {}: invalid filename", file.name));
                continue;
            }

            let file_content = match base64::engine::general_purpose::STANDARD.decode(&file.content)
            {
                Ok(content) => content,
                Err(e) => {
                    results.push(format!("❌ {}: invalid base64 content - {}", file.name, e));
                    continue;
                }
            };

            // Validate file size
            if let Err(e) = validate_file_size(&file_content) {
                results.push(format!("❌ {}: {}", file.name, e));
                continue;
            }

            let req = RequestMessagesSendFile::new((
                ChatId(self.chat_id.clone()),
                MultipartName::FileContent {
                    filename: file.name.clone(),
                    content: file_content.clone(),
                },
            ));

            match self.client().send_api_request(req).await {
                Ok(_response) => results.push(format!(
                    "✅ {}: uploaded successfully ({})",
                    file.name,
                    format_file_size(file_content.len())
                )),
                Err(e) => results.push(format!("❌ {}: failed to upload - {}", file.name, e)),
            }
        }

        let summary = format!(
            "Upload Summary:\n{}\n\nTotal: {} files processed",
            results.join("\n"),
            results.len()
        );

        // Send summary message if caption provided
        if let Some(caption) = caption {
            let summary_text = format!("{}\n\n{}", caption, summary);
            let summary_req = RequestMessagesSendText::new(ChatId(self.chat_id.clone()))
                .with_text(summary_text)
                .with_parse_mode(ParseMode::HTML);

            self.client()
                .send_api_request(summary_req)
                .await
                .map_err(McpError::from)
                .into_mcp_result()
        } else {
            convert_result(summary).map_err(ErrorData::from)
        }
    }

    #[tool(description = "Create and upload CSV file from structured data")]
    async fn upload_csv_file(
        &self,
        #[tool(param)]
        #[schemars(description = "File name (will add .csv extension if missing)")]
        file_name: String,
        #[tool(param)]
        #[schemars(description = "CSV headers as comma-separated string")]
        headers: String,
        #[tool(param)]
        #[schemars(description = "CSV rows as JSON array of arrays")]
        rows_json: String,
        #[tool(param)]
        #[schemars(description = "Optional caption/description")]
        caption: Option<String>,
        #[tool(param)]
        #[schemars(description = "Reply to message ID (optional)")]
        reply_msg_id: Option<String>,
    ) -> MCPResult {
        use serde_json::Value;

        // Parse rows data
        let rows: Vec<Vec<Value>> = serde_json::from_str(&rows_json)
            .map_err(|e| McpError::Other(format!("Invalid rows JSON: {}", e)))?;

        // Build CSV content
        let mut csv_content = String::new();

        // Add headers
        csv_content.push_str(&headers);
        csv_content.push('\n');

        // Add rows
        for row in rows {
            let row_strings: Vec<String> = row
                .into_iter()
                .map(|v| match v {
                    Value::String(s) => s,
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => String::new(),
                    _ => v.to_string(),
                })
                .collect();
            csv_content.push_str(&row_strings.join(","));
            csv_content.push('\n');
        }

        // Ensure .csv extension
        let final_filename = if file_name.ends_with(".csv") {
            file_name
        } else {
            format!("{}.csv", file_name)
        };

        validate_filename(&final_filename)?;
        let file_content = csv_content.into_bytes();
        validate_file_size(&file_content)?;

        let mut req = RequestMessagesSendFile::new((
            ChatId(self.chat_id.clone()),
            MultipartName::FileContent {
                filename: final_filename,
                content: file_content,
            },
        ));

        if let Some(caption) = caption {
            req = req.with_text(caption);
        }
        if let Some(reply_msg_id) = reply_msg_id {
            req = req.with_reply_msg_id(MsgId(reply_msg_id));
        }

        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Upload code file with syntax highlighting")]
    async fn upload_code_file(
        &self,
        #[tool(param)]
        #[schemars(
            description = "File name with appropriate extension (e.g., 'script.py', 'code.rs')"
        )]
        file_name: String,
        #[tool(param)]
        #[schemars(description = "Source code content")]
        code_content: String,
        #[tool(param)]
        #[schemars(description = "Programming language (optional, auto-detected from extension)")]
        language: Option<String>,
        #[tool(param)]
        #[schemars(description = "Optional description/comments")]
        description: Option<String>,
        #[tool(param)]
        #[schemars(description = "Reply to message ID (optional)")]
        reply_msg_id: Option<String>,
    ) -> MCPResult {
        validate_filename(&file_name)?;

        let file_content = code_content.into_bytes();
        validate_file_size(&file_content)?;

        // Detect language from extension if not provided
        let detected_lang = language.unwrap_or_else(|| {
            crate::file_utils::get_file_extension(&file_name)
                .map(|ext| match ext.to_lowercase().as_str() {
                    "rs" => "Rust",
                    "py" => "Python",
                    "js" => "JavaScript",
                    "ts" => "TypeScript",
                    "go" => "Go",
                    "java" => "Java",
                    "cpp" | "cc" | "cxx" => "C++",
                    "c" => "C",
                    "cs" => "C#",
                    "php" => "PHP",
                    "rb" => "Ruby",
                    "swift" => "Swift",
                    "kt" => "Kotlin",
                    "scala" => "Scala",
                    "sql" => "SQL",
                    "sh" | "bash" => "Shell",
                    _ => "Code",
                })
                .unwrap_or("Code")
                .to_string()
        });

        let caption_text = if let Some(desc) = description {
            format!(
                "📄 <b>{}</b> ({} file)\n\n{}",
                file_name, detected_lang, desc
            )
        } else {
            format!("📄 <b>{}</b> ({} file)", file_name, detected_lang)
        };

        let mut req = RequestMessagesSendFile::new((
            ChatId(self.chat_id.clone()),
            MultipartName::FileContent {
                filename: file_name,
                content: file_content,
            },
        ));

        req = req.with_text(caption_text).with_parse_mode(ParseMode::HTML);

        if let Some(reply_msg_id) = reply_msg_id {
            req = req.with_reply_msg_id(MsgId(reply_msg_id));
        }

        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
    }

    #[tool(description = "Validate file content and metadata before uploading")]
    async fn validate_file_before_upload(
        &self,
        #[tool(param)]
        #[schemars(description = "File name")]
        file_name: String,
        #[tool(param)]
        #[schemars(description = "Base64 encoded file content")]
        base64_content: String,
    ) -> MCPResult {
        use serde_json::json;

        // Validate filename
        let filename_valid = validate_filename(&file_name).is_ok();
        let sanitized_name = if !filename_valid {
            Some(sanitize_filename(&file_name))
        } else {
            None
        };

        // Decode and validate content
        let file_content = match base64::engine::general_purpose::STANDARD.decode(&base64_content) {
            Ok(content) => content,
            Err(e) => {
                return convert_result(json!({
                    "valid": false,
                    "errors": [format!("Invalid base64 content: {}", e)],
                    "filename": file_name
                }))
                .map_err(ErrorData::from);
            }
        };

        let size_valid = validate_file_size(&file_content).is_ok();
        let file_size = file_content.len();
        let mime_type = get_mime_type(&file_name);
        let is_text = crate::file_utils::is_text_file(&file_name);
        let is_image = crate::file_utils::is_image_file(&file_name);
        let is_audio = crate::file_utils::is_audio_file(&file_name);

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if !filename_valid {
            errors.push("Invalid filename".to_string());
        }
        if !size_valid {
            errors.push(format!("File too large ({})", format_file_size(file_size)));
        }
        if file_size == 0 {
            warnings.push("File is empty".to_string());
        }

        let validation_result = json!({
            "valid": errors.is_empty(),
            "filename": file_name,
            "sanitized_filename": sanitized_name,
            "file_size": file_size,
            "file_size_formatted": format_file_size(file_size),
            "mime_type": mime_type,
            "is_text": is_text,
            "is_image": is_image,
            "is_audio": is_audio,
            "errors": errors,
            "warnings": warnings,
            "can_upload": errors.is_empty()
        });

        convert_result(validation_result).map_err(ErrorData::from)
    }

    #[tool(description = "Upload file with automatic format conversion")]
    async fn upload_with_conversion(
        &self,
        #[tool(param)]
        #[schemars(description = "Original file name")]
        file_name: String,
        #[tool(param)]
        #[schemars(description = "Base64 encoded file content")]
        base64_content: String,
        #[tool(param)]
        #[schemars(description = "Target format (txt, json, csv, etc.)")]
        target_format: String,
        #[tool(param)]
        #[schemars(description = "Optional caption")]
        caption: Option<String>,
    ) -> MCPResult {
        // Decode content
        let file_content = base64::engine::general_purpose::STANDARD
            .decode(&base64_content)
            .map_err(|e| McpError::Other(format!("Invalid base64 content: {}", e)))?;

        validate_file_size(&file_content)?;

        // Try to convert content based on target format
        let (converted_content, new_filename) = match target_format.to_lowercase().as_str() {
            "txt" => {
                // Convert to plain text
                let text_content = if crate::file_utils::is_likely_text(&file_content) {
                    String::from_utf8_lossy(&file_content).to_string()
                } else {
                    format!(
                        "Binary file converted to text representation:\nFile: {}\nSize: {}\nContent: [Binary data - {} bytes]",
                        file_name,
                        format_file_size(file_content.len()),
                        file_content.len()
                    )
                };
                (
                    text_content.into_bytes(),
                    format!("{}.txt", file_name.split('.').next().unwrap_or("file")),
                )
            }
            "json" => {
                // Try to parse and reformat as JSON if it's already JSON, otherwise create JSON metadata
                let json_content = if let Ok(text) = String::from_utf8(file_content.clone()) {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                        serde_json::to_string_pretty(&value)
                            .map_err(|e| McpError::Other(format!("JSON formatting error: {}", e)))?
                    } else {
                        // Create JSON metadata for non-JSON files
                        let metadata = serde_json::json!({
                            "original_filename": file_name,
                            "file_size": file_content.len(),
                            "mime_type": get_mime_type(&file_name),
                            "is_text": crate::file_utils::is_likely_text(&file_content),
                            "content_preview": if crate::file_utils::is_likely_text(&file_content) {
                                Some(String::from_utf8_lossy(&file_content[..std::cmp::min(200, file_content.len())]).to_string())
                            } else {
                                None
                            }
                        });
                        serde_json::to_string_pretty(&metadata)
                            .map_err(|e| McpError::Other(format!("JSON creation error: {}", e)))?
                    }
                } else {
                    return Err(
                        McpError::Other("Cannot convert binary file to JSON".to_string()).into(),
                    );
                };
                (
                    json_content.into_bytes(),
                    format!("{}.json", file_name.split('.').next().unwrap_or("file")),
                )
            }
            _ => {
                return Err(McpError::Other(format!(
                    "Unsupported conversion format: {}",
                    target_format
                ))
                .into());
            }
        };

        validate_filename(&new_filename)?;
        validate_file_size(&converted_content)?;

        let mut req = RequestMessagesSendFile::new((
            ChatId(self.chat_id.clone()),
            MultipartName::FileContent {
                filename: new_filename.clone(),
                content: converted_content,
            },
        ));

        let final_caption = if let Some(cap) = caption {
            format!("🔄 Converted {} to {}\n\n{}", file_name, new_filename, cap)
        } else {
            format!("🔄 Converted {} to {}", file_name, new_filename)
        };

        req = req.with_text(final_caption);

        self.client()
            .send_api_request(req)
            .await
            .map_err(McpError::from)
            .into_mcp_result()
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
        upload_file_from_base64,
        upload_text_as_file,
        upload_json_file,
        upload_multiple_files,
        upload_csv_file,
        upload_code_file,
        validate_file_before_upload,
        upload_with_conversion,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;

    // Mock server for testing
    fn create_test_server() -> Server {
        // Set test environment variables
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat_123");
            std::env::set_var("VKTEAMS_BOT_API_TOKEN", "test_token");
            std::env::set_var("VKTEAMS_BOT_API_URL", "https://test.api.com");
        }

        Server {
            bot: Arc::new(Bot::default()),
            chat_id: "test_chat_123".to_string(),
        }
    }

    #[test]
    fn test_into_call_tool_result_success() {
        let test_data = json!({"message": "test success"});
        let result: std::result::Result<serde_json::Value, McpError> = Ok(test_data);

        let call_result = result.into_mcp_result();
        assert!(call_result.is_ok());
    }

    #[test]
    fn test_into_call_tool_result_error() {
        let error = McpError::Other("test error".to_string());
        let result: std::result::Result<serde_json::Value, McpError> = Err(error);

        let call_result = result.into_mcp_result();
        assert!(call_result.is_err());
    }

    #[test]
    fn test_convert_result_with_json() {
        let test_data = json!({"key": "value", "number": 42});
        let result = convert_result(test_data);

        assert!(result.is_ok());
    }

    #[test]
    fn test_server_get_info() {
        let server = create_test_server();
        let info = server.get_info();

        assert!(info.capabilities.tools.is_some());
        assert!(info.capabilities.prompts.is_some());
        assert!(info.instructions.is_some());

        let instructions = info.instructions.unwrap();
        assert!(instructions.contains("VKTeams MCP Server"));
        assert!(instructions.contains("send_text"));
        assert!(instructions.contains("upload_file_from_base64"));
    }

    #[tokio::test]
    async fn test_validate_file_before_upload_valid_file() {
        let server = create_test_server();
        let base64_content = base64::engine::general_purpose::STANDARD.encode(b"Hello, world!");

        let result = server
            .validate_file_before_upload("test.txt".to_string(), base64_content)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_file_before_upload_invalid_base64() {
        let server = create_test_server();

        let result = server
            .validate_file_before_upload("test.txt".to_string(), "invalid-base64!".to_string())
            .await;

        assert!(result.is_ok()); // Function handles invalid base64 gracefully
    }

    #[test]
    fn test_upload_json_file_invalid_json() {
        use tokio::runtime::Runtime;
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            let server = create_test_server();
            let invalid_json = r#"{"name": "test", "value":}"#; // Invalid JSON

            let result = server
                .upload_json_file(
                    "invalid_data".to_string(),
                    invalid_json.to_string(),
                    Some(true),
                    None,
                    None,
                )
                .await;

            assert!(result.is_err());
        });
    }

    #[test]
    fn test_upload_csv_file_invalid_rows_json() {
        use tokio::runtime::Runtime;
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            let server = create_test_server();
            let headers = "Name,Age";
            let invalid_rows_json = r#"[["John", 25,]]"#; // Invalid JSON

            let result = server
                .upload_csv_file(
                    "test".to_string(),
                    headers.to_string(),
                    invalid_rows_json.to_string(),
                    None,
                    None,
                )
                .await;

            assert!(result.is_err());
        });
    }

    #[test]
    fn test_upload_multiple_files_validation() {
        use tokio::runtime::Runtime;
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            let server = create_test_server();

            // Test with too many files (> 10)
            let files_json = serde_json::to_string(&vec![
                json!({"name": "file1.txt", "content": base64::engine::general_purpose::STANDARD.encode(b"content1")}); 11
            ]).unwrap();

            let result = server.upload_multiple_files(
                files_json,
                Some("Too many files test".to_string()),
            ).await;

            assert!(result.is_err());
        });
    }

    #[test]
    fn test_upload_multiple_files_empty_list() {
        use tokio::runtime::Runtime;
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            let server = create_test_server();
            let files_json = "[]";

            let result = server
                .upload_multiple_files(files_json.to_string(), None)
                .await;

            assert!(result.is_err());
        });
    }

    #[test]
    fn test_upload_with_conversion_unsupported_format() {
        use tokio::runtime::Runtime;
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            let server = create_test_server();
            let base64_content = base64::engine::general_purpose::STANDARD.encode(b"test content");

            let result = server
                .upload_with_conversion(
                    "test.txt".to_string(),
                    base64_content,
                    "unsupported_format".to_string(),
                    None,
                )
                .await;

            assert!(result.is_err());
        });
    }

    #[test]
    fn test_base64_decoding_error_handling() {
        use tokio::runtime::Runtime;
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            let server = create_test_server();

            let result = server
                .upload_file_from_base64(
                    "test.txt".to_string(),
                    "invalid-base64-content!@#".to_string(),
                    None,
                    None,
                )
                .await;

            assert!(result.is_err());
        });
    }
}
