//! Pin Message method in chat `chats/pinMessage`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_pinMessage)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// # Pin Message request method `chats/pinMessage`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsPinMessage {
    pub chat_id: ChatId,
    pub msg_id: MsgId,
}
/// # Pin Message response method `chats/pinMessage`
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
    /// Create a new [`RequestChatsPinMessage`]
    /// ## Parameters
    /// - `chat_id` - [`ChatId`]
    /// - `msg_id` - [`MsgId`]
    pub fn new(chat_id: ChatId, msg_id: MsgId) -> Self {
        Self { chat_id, msg_id }
    }
}
