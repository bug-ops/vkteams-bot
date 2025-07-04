use crate::cli_bridge::CliBridge;
use rmcp::{handler::server::router::tool::ToolRouter, schemars};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use vkteams_bot::config::UnifiedConfig;

#[derive(Debug)]
pub struct Server {
    pub cli: Arc<CliBridge>,
    pub config: UnifiedConfig,
    pub tool_router: ToolRouter<Self>,
}

impl Clone for Server {
    fn clone(&self) -> Self {
        Self {
            cli: Arc::clone(&self.cli),
            config: self.config.clone(),
            tool_router: self.tool_router.clone(),
        }
    }
}

// Parameter structures for tool calls

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SendTextParams {
    #[schemars(description = r#"Text message to send.
        You can use HTML formatting:
            <b>bold</b>, <i>italic</i>, <u>underline</u>, <s>strikethrough</s>
            <a href="http://www.example.com/">inline URL</a>
            <code>inline code</code>
            <pre>pre-formatted code block</pre>
        "#)]
    pub text: String,
    #[schemars(description = "Chat ID (optional, uses default if not provided)")]
    pub chat_id: Option<String>,
    #[schemars(description = "Reply to message ID (optional)")]
    pub reply_msg_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SendFileParams {
    #[schemars(description = "Path to file to send")]
    pub file_path: String,
    #[schemars(description = "Chat ID (optional, uses default if not provided)")]
    pub chat_id: Option<String>,
    #[schemars(description = "Caption for the file (optional)")]
    pub caption: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SendVoiceParams {
    #[schemars(description = "Path to voice file to send (.ogg, .mp3, .wav, .m4a)")]
    pub file_path: String,
    #[schemars(description = "Chat ID (optional, uses default if not provided)")]
    pub chat_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EditMessageParams {
    #[schemars(description = "Message ID to edit")]
    pub message_id: String,
    #[schemars(description = "New text for the message")]
    pub new_text: String,
    #[schemars(description = "Chat ID (optional, uses default if not provided)")]
    pub chat_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeleteMessageParams {
    #[schemars(description = "Message ID to delete")]
    pub message_id: String,
    #[schemars(description = "Chat ID (optional, uses default if not provided)")]
    pub chat_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PinMessageParams {
    #[schemars(description = "Message ID to pin")]
    pub message_id: String,
    #[schemars(description = "Chat ID (optional, uses default if not provided)")]
    pub chat_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UnpinMessageParams {
    #[schemars(description = "Message ID to unpin")]
    pub message_id: String,
    #[schemars(description = "Chat ID (optional, uses default if not provided)")]
    pub chat_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ChatInfoParams {
    #[schemars(description = "Chat ID (optional, uses default if not provided)")]
    pub chat_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetProfileParams {
    #[schemars(description = "User ID to get profile for")]
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetChatMembersParams {
    #[schemars(description = "Chat ID (optional, uses default if not provided)")]
    pub chat_id: Option<String>,
    #[schemars(description = "Cursor for pagination (optional)")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetChatAdminsParams {
    #[schemars(description = "Chat ID (optional, uses default if not provided)")]
    pub chat_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SetChatTitleParams {
    #[schemars(description = "New chat title")]
    pub title: String,
    #[schemars(description = "Chat ID (optional, uses default if not provided)")]
    pub chat_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SetChatAboutParams {
    #[schemars(description = "New chat description")]
    pub about: String,
    #[schemars(description = "Chat ID (optional, uses default if not provided)")]
    pub chat_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SendActionParams {
    #[schemars(description = "Action to send: 'typing' or 'looking'")]
    pub action: String,
    #[schemars(description = "Chat ID (optional, uses default if not provided)")]
    pub chat_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UploadFileFromBase64Params {
    #[schemars(description = "File name with extension")]
    pub file_name: String,
    #[schemars(description = "Base64 encoded file content")]
    pub base64_content: String,
    #[schemars(description = "Chat ID (optional, uses default if not provided)")]
    pub chat_id: Option<String>,
    #[schemars(description = "Caption for the file (optional)")]
    pub caption: Option<String>,
    #[schemars(description = "Reply to message ID (optional)")]
    pub reply_msg_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UploadTextAsFileParams {
    #[schemars(description = "File name with extension")]
    pub file_name: String,
    #[schemars(description = "Text content to save as file")]
    pub content: String,
    #[schemars(description = "Chat ID (optional, uses default if not provided)")]
    pub chat_id: Option<String>,
    #[schemars(description = "Caption for the file (optional)")]
    pub caption: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UploadJsonFileParams {
    #[schemars(description = "File name (will add .json extension if missing)")]
    pub file_name: String,
    #[schemars(description = "JSON data as string")]
    pub json_data: String,
    #[schemars(description = "Pretty print JSON (default: true)")]
    pub pretty: Option<bool>,
    #[schemars(description = "Chat ID (optional, uses default if not provided)")]
    pub chat_id: Option<String>,
    #[schemars(description = "Caption for the file (optional)")]
    pub caption: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FileInfoParams {
    #[schemars(description = "File ID to get information about")]
    pub file_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SearchSemanticParams {
    #[schemars(description = "Search query")]
    pub query: String,
    #[schemars(description = "Chat ID to search in (optional, searches all if not provided)")]
    pub chat_id: Option<String>,
    #[schemars(description = "Maximum number of results (default: 10)")]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SearchTextParams {
    #[schemars(description = "Search query")]
    pub query: String,
    #[schemars(description = "Chat ID to search in (optional, searches all if not provided)")]
    pub chat_id: Option<String>,
    #[schemars(description = "Maximum number of results (default: 10)")]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetDatabaseStatsParams {
    #[schemars(
        description = "Chat ID for specific chat stats (optional, all chats if not provided)"
    )]
    pub chat_id: Option<String>,
    #[schemars(description = "Date since when to count (optional)")]
    pub since: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetContextParams {
    #[schemars(description = "Chat ID (optional, uses default if not provided)")]
    pub chat_id: Option<String>,
    #[schemars(
        description = "Context type: 'recent', 'summary', or 'keywords' (default: 'recent')"
    )]
    pub context_type: Option<String>,
    #[schemars(description = "Timeframe for context (optional)")]
    pub timeframe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetRecentMessagesParams {
    #[schemars(description = "Chat ID (optional, gets from all chats if not provided)")]
    pub chat_id: Option<String>,
    #[schemars(description = "Maximum number of messages to return (default: 50)")]
    pub limit: Option<usize>,
    #[schemars(description = "Get messages since this timestamp (ISO 8601 format)")]
    pub since: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EventsGetParams {
    #[schemars(description = "Last event ID for pagination (optional)")]
    pub last_event_id: Option<String>,
    #[schemars(description = "Poll time in seconds (optional)")]
    pub poll_time: Option<u64>,
}
