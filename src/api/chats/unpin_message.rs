use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsUnpinMessage`]
///
/// [`SendMessagesAPIMethods::ChatsUnpinMessage`]: enum.SendMessagesAPIMethods.html#variant.ChatsUnpinMessage
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsUnpinMessage {
    pub chat_id: ChatId,
    pub msg_id: MsgId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsUnpinMessage`]
///
/// [`SendMessagesAPIMethods::ChatsUnpinMessage`]: enum.SendMessagesAPIMethods.html#variant.ChatsUnpinMessage
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsUnpinMessage {
    pub ok: bool,
}
impl BotRequest for RequestChatsUnpinMessage {
    const METHOD: &'static str = "chats/unpinMessage";
    type RequestType = Self;
    type ResponseType = ResponseChatsUnpinMessage;
}
impl RequestChatsUnpinMessage {
    /// Create a new RequestChatsUnpinMessage with the chat_id and msg_id
    /// - `chat_id` - [`ChatId`]
    /// - `msg_id` - [`MsgId`]
    pub fn new(chat_id: ChatId, msg_id: MsgId) -> Self {
        Self { chat_id, msg_id }
    }
}
