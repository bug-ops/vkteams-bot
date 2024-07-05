//! Block User method `chats/blockUser`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_blockUser)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// # Chats block user request
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsBlockUser {
    pub chat_id: ChatId,
    pub user_id: UserId,
    pub del_last_messages: bool,
}
/// # Chats block user response
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
    /// # Create a new [`RequestChatsBlockUser`]
    /// ## Parameters
    /// - `chat_id`: [`ChatId`]
    /// - `user_id`: [`UserId`]
    pub fn new(chat_id: ChatId, user_id: UserId) -> Self {
        Self {
            chat_id,
            user_id,
            ..Default::default()
        }
    }
    /// # Set del_last_messages for [`RequestChatsBlockUser`]
    /// ## Parameters
    /// - `del_last_messages`: [`bool`]
    /// - `true` - delete all messages from the user in the chat
    /// - `false` - do not delete messages
    pub fn del_last_messages(mut self, del_last_messages: bool) -> Self {
        self.del_last_messages = del_last_messages;
        self
    }
}
