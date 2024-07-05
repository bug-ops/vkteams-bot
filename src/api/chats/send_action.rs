//! Send chat actions method `chats/sendActions`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_sendActions)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// # Chat actions method `chats/sendActions`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsSendAction {
    pub chat_id: ChatId,
    pub actions: ChatActions,
}
/// # Chat actions response method `chats/sendActions`
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ResponseChatsSendAction {
    pub ok: bool,
}
impl BotRequest for RequestChatsSendAction {
    const METHOD: &'static str = "chats/sendActions";
    type RequestType = Self;
    type ResponseType = ResponseChatsSendAction;
}
impl RequestChatsSendAction {
    /// Create a new [`RequestChatsSendAction`]
    /// ## Parameters
    /// - `chat_id` - [`ChatId`]
    /// - `actions` - [`ChatActions`]
    pub fn new(chat_id: ChatId, actions: ChatActions) -> Self {
        Self { chat_id, actions }
    }
}
