use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsGetBlockedUsers`]
///
/// [`SendMessagesAPIMethods::ChatsGetBlockedUsers`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetBlockedUsers
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetBlockedUsers {
    pub chat_id: ChatId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsGetBlockedUsers`]
///
/// [`SendMessagesAPIMethods::ChatsGetBlockedUsers`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetBlockedUsers
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
    /// Create a new RequestChatsGetBlockedUsers with the chat_id
    /// - `chat_id` - [`ChatId`]
    pub fn new(chat_id: ChatId) -> Self {
        Self { chat_id }
    }
}
