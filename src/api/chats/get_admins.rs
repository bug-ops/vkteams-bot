//! Get chat admins method `chats/getAdmins`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_getAdmins)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// # Chats get admins request method `chats/getAdmins`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetAdmins {
    pub chat_id: ChatId,
}
/// # Chats get admins response method `chats/getAdmins`
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGetAdmins {
    #[serde(default)]
    pub admins: Vec<Admin>,
}
impl BotRequest for RequestChatsGetAdmins {
    const METHOD: &'static str = "chats/getAdmins";
    type RequestType = Self;
    type ResponseType = ResponseChatsGetAdmins;
}
impl RequestChatsGetAdmins {
    /// # Create a new [`RequestChatsGetAdmins`]
    /// ## Parameters
    /// - `chat_id`: [`ChatId`]
    pub fn new(chat_id: ChatId) -> Self {
        Self { chat_id }
    }
}
