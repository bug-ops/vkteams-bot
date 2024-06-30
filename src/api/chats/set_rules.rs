use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsSetRules`]
///
/// [`SendMessagesAPIMethods::ChatsSetRules`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetRules
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsSetRules {
    pub chat_id: ChatId,
    pub rules: String,
}
/// Response for method [`SendMessagesAPIMethods::ChatsSetRules`]
///
/// [`SendMessagesAPIMethods::ChatsSetRules`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetRules
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
    /// Create a new RequestChatsSetRules with the chat_id and rules
    /// - `chat_id` - [`ChatId`]
    /// - `rules` - [`String`]
    pub fn new(chat_id: ChatId, rules: String) -> Self {
        Self { chat_id, rules }
    }
}
