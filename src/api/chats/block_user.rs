use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsBlockUser`]
///
///
/// [`SendMessagesAPIMethods::ChatsBlockUser`]: enum.SendMessagesAPIMethods.html#variant.ChatsBlockUser
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsBlockUser {
    pub chat_id: ChatId,
    pub user_id: UserId,
    pub del_last_messages: bool,
}
/// Response for method [`SendMessagesAPIMethods::ChatsBlockUser`]
///
/// [`SendMessagesAPIMethods::ChatsBlockUser`]: enum.SendMessagesAPIMethods.html#variant.ChatsBlockUser
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsBlockUser {
    pub ok: bool,
    #[serde(default)]
    pub description: String,
}
impl BotRequest for RequestChatsBlockUser {
    const METHOD: &'static str = "chats/blockUser";
    type RequestType = Self;
    type ResponseType = ResponseChatsBlockUser;
}
impl RequestChatsBlockUser {
    /// Create a new RequestChatsBlockUser with the chat_id, user_id and del_last_messages
    /// - `chat_id` - [`ChatId`]
    /// - `user_id` - [`UserId`]
    /// - `del_last_messages` - [`bool`]
    pub fn new(chat_id: ChatId, user_id: UserId, del_last_messages: bool) -> Self {
        Self {
            chat_id,
            user_id,
            del_last_messages,
        }
    }
}
