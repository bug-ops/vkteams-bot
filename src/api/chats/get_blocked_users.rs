//! Get blocked users method `chats/getBlockedUsers`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_getBlockedUsers)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// # Chats get blocked users request method `chats/getBlockedUsers`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetBlockedUsers {
    pub chat_id: ChatId,
}
/// # Chats get blocked users response method `chats/getBlockedUsers`
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGetBlockedUsers {
    #[serde(default)]
    pub users: Vec<Users>,
}
impl BotRequest for RequestChatsGetBlockedUsers {
    const METHOD: &'static str = "chats/getBlockedUsers";
    type RequestType = Self;
    type ResponseType = ResponseChatsGetBlockedUsers;
}
impl RequestChatsGetBlockedUsers {
    /// Create a new [`RequestChatsGetBlockedUsers`] with the chat_id
    /// ## Parameters
    /// - `chat_id`: [`ChatId`]
    pub fn new(chat_id: ChatId) -> Self {
        Self { chat_id }
    }
}
