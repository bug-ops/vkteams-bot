//! Set chat title method `chats/setTitle`
//! [More Info](https://teams.vk.com/botapi/#/chats/get_chats_setTitle)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// # Set chat title method `chats/setTitle`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsSetTitle {
    pub chat_id: ChatId,
    pub title: String,
}
/// # Set chat title method `chats/setTitle`
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
    /// Create a new [`RequestChatsSetTitle`]
    /// ## Parameters
    /// - `chat_id` - [`ChatId`]
    /// - `title` - [`String`]
    pub fn new(chat_id: ChatId, title: String) -> Self {
        Self { chat_id, title }
    }
}
