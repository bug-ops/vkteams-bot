//! Set a chat's about text method `chats/setAbout`
//! [Nore info](https://teams.vk.com/botapi/#/chats/get_chats_setAbout)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// # Chat's about text method `chats/setAbout`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsSetAbout {
    pub chat_id: ChatId,
    pub about: String,
}
/// # Chat's about text method `chats/setAbout`
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
    /// Create a new [`RequestChatsSetAbout`]
    /// ## Parameters
    /// - `chat_id` - [`ChatId`]
    /// - `about` - [`String`]
    pub fn new(chat_id: ChatId, about: String) -> Self {
        Self { chat_id, about }
    }
}
