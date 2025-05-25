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
        let req = RequestEventsGet::new(last_event_id.unwrap_or(0));
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
    });
}
