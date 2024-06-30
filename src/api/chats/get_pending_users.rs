use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsBlockUser`]
///
/// [`SendMessagesAPIMethods::ChatsBlockUser`]: enum.SendMessagesAPIMethods.html#variant.ChatsBlockUser
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetPendingUsers {
    pub chat_id: ChatId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsBlockUser`]
///
/// [`SendMessagesAPIMethods::ChatsBlockUser`]: enum.SendMessagesAPIMethods.html#variant.ChatsBlockUser
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
    /// Create a new RequestChatsGetPendingUsers with the chat_id
    /// - `chat_id` - [`ChatId`]
    pub fn new(chat_id: ChatId) -> Self {
        Self { chat_id }
    }
}
