//! Set rules for a chat method `chats/setRules`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_setRules)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// # Set rules for a chat method `chats/setRules`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsSetRules {
    pub chat_id: ChatId,
    pub rules: String,
}
/// # Set rules for a chat method `chats/setRules`
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsSetRules {
    pub ok: bool,
}
impl BotRequest for RequestChatsSetRules {
    const METHOD: &'static str = "chats/setRules";
    type RequestType = Self;
    type ResponseType = ResponseChatsSetRules;
}
impl RequestChatsSetRules {
    /// Create a new [`RequestChatsSetRules`]
    /// ## Parameters
    /// - `chat_id` - [`ChatId`]
    /// - `rules` - [`String`]
    pub fn new(chat_id: ChatId, rules: String) -> Self {
        Self { chat_id, rules }
    }
}
