use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsSetTitle`]
///
/// [`SendMessagesAPIMethods::ChatsSetTitle`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetTitle
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsSetTitle {
    pub chat_id: ChatId,
    pub title: String,
}
/// Response for method [`SendMessagesAPIMethods::ChatsSetTitle`]
///
/// [`SendMessagesAPIMethods::ChatsSetTitle`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetTitle
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsSetTitle {
    pub ok: bool,
}
impl BotRequest for RequestChatsSetTitle {
    const METHOD: &'static str = "chats/setTitle";
    type RequestType = Self;
    type ResponseType = ResponseChatsSetTitle;
}
impl RequestChatsSetTitle {
    /// Create a new RequestChatsSetTitle with the chat_id and title
    /// - `chat_id` - [`ChatId`]
    /// - `title` - [`String`]
    pub fn new(chat_id: ChatId, title: String) -> Self {
        Self { chat_id, title }
    }
}
