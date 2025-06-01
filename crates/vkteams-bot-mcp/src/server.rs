use crate::errors::McpError;
use crate::types::Server;
use rmcp::tool_box;
use rmcp::{
    ServerHandler,
    model::{CallToolResult, Content, ErrorData, ServerInfo},
    tool,
};
use serde::Serialize;
use std::result::Result;
use vkteams_bot::prelude::*;

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
        ServerInfo {
            instructions: Some("VKTeams MCP Server — a server for managing a VK Teams bot via MCP (Machine Control Protocol).\n\nFeatures:\n- Send text messages to chat (send_text)\n- Get bot information (self_get)\n- Get chat information (chat_info)\n- Get file information (file_info)\n- Get events (events_get)\n\nTo send a message, use the send_text tool with the text parameter.\nExample: {\"tool\": \"send_text\", \"params\": {\"text\": \"Hello!\"}}\n\nYou must pre-configure the environment variables VKTEAMS_BOT_API_TOKEN, VKTEAMS_BOT_API_URL, VKTEAMS_BOT_CHAT_ID.\n\nDocumentation: https://teams.vk.com/botapi/?lang=en".into()),
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
        #[schemars(description = "Text to send in chat")]
        text: String,
    ) -> MCPResult {
        let req = RequestMessagesSendText::new(ChatId(self.chat_id.clone())).with_text(text);
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
    async fn send_file(
        &self,
        #[tool(param)]
        #[schemars(description = "Path to file")]
        file_path: String,
    ) -> MCPResult {
        let req = RequestMessagesSendFile::new((
            ChatId(self.chat_id.clone()),
            MultipartName::File(file_path),
        ));
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
        #[schemars(description = "Path to voice file")]
        file_path: String,
    ) -> MCPResult {
        let req = RequestMessagesSendVoice::new((
            ChatId(self.chat_id.clone()),
            MultipartName::File(file_path),
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
        #[schemars(description = "New text")]
        text: String,
    ) -> MCPResult {
        let req = RequestMessagesEditText::new((ChatId(self.chat_id.clone()), MsgId(msg_id)))
            .set_text(MessageTextParser::new().add(MessageTextFormat::Plain(text)))
            .map_err(McpError::from)?;
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
        #[schemars(description = "Path to image file")]
        file_path: String,
    ) -> MCPResult {
        let req = RequestChatsAvatarSet::new((
            ChatId(self.chat_id.clone()),
            MultipartName::Image(file_path),
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
    });
}
