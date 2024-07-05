//! Unblock User in chat method `chats/unblockUser`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_unblockUser)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// # Unblock User in chat method `chats/unblockUser`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsUnblockUser {
    pub chat_id: ChatId,
    pub user_id: UserId,
}
/// # Unblock User in chat method `chats/unblockUser`
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsUnblockUser {
    pub ok: bool,
}
impl BotRequest for RequestChatsUnblockUser {
    const METHOD: &'static str = "chats/unblockUser";
    type RequestType = Self;
    type ResponseType = ResponseChatsUnblockUser;
}
impl RequestChatsUnblockUser {
    /// Create a new [`RequestChatsUnblockUser`]
    /// ## Parameters
    /// - `chat_id` - [`ChatId`]
    /// - `user_id` - [`UserId`]
    pub fn new(chat_id: ChatId, user_id: UserId) -> Self {
        Self { chat_id, user_id }
    }
}
