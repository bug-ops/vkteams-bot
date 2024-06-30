use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsSetAbout`]
///
/// [`SendMessagesAPIMethods::ChatsSetAbout`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetAbout
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsSetAbout {
    pub chat_id: ChatId,
    pub about: String,
}
/// Response for method [`SendMessagesAPIMethods::ChatsSetAbout`]
///
/// [`SendMessagesAPIMethods::ChatsSetAbout`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetAbout
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsSetAbout {
    pub ok: bool,
}
impl BotRequest for RequestChatsSetAbout {
    const METHOD: &'static str = "chats/setAbout";
    type RequestType = Self;
    type ResponseType = ResponseChatsSetAbout;
}
impl RequestChatsSetAbout {
    /// Create a new RequestChatsSetAbout with the chat_id and about
    /// - `chat_id` - [`ChatId`]
    /// - `about` - [`String`]
    pub fn new(chat_id: ChatId, about: String) -> Self {
        Self { chat_id, about }
    }
}
