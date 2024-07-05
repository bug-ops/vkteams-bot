//! Get pending users method `chats/getPendingUsers`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_getPendingUsers)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// # Chats get pending users request method `chats/getPendingUsers`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetPendingUsers {
    pub chat_id: ChatId,
}
/// # Chats get pending users response method `chats/getPendingUsers`
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGetPendingUsers {
    #[serde(default)]
    pub users: Vec<Users>,
}
impl BotRequest for RequestChatsGetPendingUsers {
    const METHOD: &'static str = "chats/getPendingUsers";
    type RequestType = Self;
    type ResponseType = ResponseChatsGetPendingUsers;
}
impl RequestChatsGetPendingUsers {
    /// Create a new [`RequestChatsGetPendingUsers`]
    /// ## Parameters
    /// - `chat_id` - [`ChatId`]
    pub fn new(chat_id: ChatId) -> Self {
        Self { chat_id }
    }
}
