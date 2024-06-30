use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsGetAdmins`]
///
/// [`SendMessagesAPIMethods::ChatsGetAdmins`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetAdmins
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetAdmins {
    pub chat_id: ChatId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsGetAdmins`]
///
/// [`SendMessagesAPIMethods::ChatsGetAdmins`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetAdmins
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
    /// Create a new RequestChatsGetAdmins with the chat_id
    /// - `chat_id` - [`ChatId`]
    pub fn new(chat_id: ChatId) -> Self {
        Self { chat_id }
    }
}
