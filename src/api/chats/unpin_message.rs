//! Unpin Message method `chats/unpinMessage`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_unpinMessage)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// # Unpin Message method `chats/unpinMessage`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsUnpinMessage {
    pub chat_id: ChatId,
    pub msg_id: MsgId,
}
/// # Unpin Message method `chats/unpinMessage`
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
    /// Create a new [`RequestChatsUnpinMessage`]
    /// ## Parameters
    /// - `chat_id` - [`ChatId`]
    /// - `msg_id` - [`MsgId`]
    pub fn new(chat_id: ChatId, msg_id: MsgId) -> Self {
        Self { chat_id, msg_id }
    }
}
