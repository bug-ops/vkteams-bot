use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsPinMessage`]
///
/// [`SendMessagesAPIMethods::ChatsPinMessage`]: enum.SendMessagesAPIMethods.html#variant.ChatsPinMessage
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsPinMessage {
    pub chat_id: ChatId,
    pub msg_id: MsgId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsPinMessage`]
///
/// [`SendMessagesAPIMethods::ChatsPinMessage`]: enum.SendMessagesAPIMethods.html#variant.ChatsPinMessage
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsPinMessage {
    pub ok: bool,
}
impl BotRequest for RequestChatsPinMessage {
    const METHOD: &'static str = "chats/pinMessage";
    type RequestType = Self;
    type ResponseType = ResponseChatsPinMessage;
}
impl RequestChatsPinMessage {
    /// Create a new RequestChatsPinMessage with the chat_id and msg_id
    /// - `chat_id` - [`ChatId`]
    /// - `msg_id` - [`MsgId`]
    pub fn new(chat_id: ChatId, msg_id: MsgId) -> Self {
        Self { chat_id, msg_id }
    }
}
