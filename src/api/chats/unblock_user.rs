use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsUnblockUser`]
///
/// [`SendMessagesAPIMethods::ChatsResolvePending`]: enum.SendMessagesAPIMethods.html#variant.ChatsUnblockUser
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsUnblockUser {
    pub chat_id: ChatId,
    pub user_id: UserId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsUnblockUser`]
///
/// [`SendMessagesAPIMethods::ChatsResolvePending`]: enum.SendMessagesAPIMethods.html#variant.ChatsUnblockUser
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
    /// Create a new RequestChatsUnblockUser with the chat_id and user_id
    /// - `chat_id` - [`ChatId`]
    /// - `user_id` - [`UserId`]
    pub fn new(chat_id: ChatId, user_id: UserId) -> Self {
        Self { chat_id, user_id }
    }
}
