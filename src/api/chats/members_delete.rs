use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsMembersDelete`]
///
/// [`SendMessagesAPIMethods::ChatsMembersDelete`]: enum.SendMessagesAPIMethods.html#variant.ChatsMembersDelete
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsMembersDelete {
    pub chat_id: ChatId,
    pub user_id: UserId,
    pub members: Vec<Sn>,
}
/// Response for method [`SendMessagesAPIMethods::ChatsMembersDelete`]
///
/// [`SendMessagesAPIMethods::ChatsMembersDelete`]: enum.SendMessagesAPIMethods.html#variant.ChatsMembersDelete
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ResponseChatsMembersDelete {
    pub ok: bool,
}
impl BotRequest for RequestChatsMembersDelete {
    const METHOD: &'static str = "chats/members/delete";
    type RequestType = Self;
    type ResponseType = ResponseChatsMembersDelete;
}
/// Add members to the chat
impl RequestChatsMembersDelete {
    /// Create a new RequestChatsMembersDelete with the chat_id and user_id
    /// - `chat_id` - [`ChatId`]
    /// - `user_id` - [`UserId`]
    pub fn new(chat_id: ChatId, user_id: UserId) -> Self {
        Self {
            chat_id,
            user_id,
            ..Default::default()
        }
    }
    /// Create a new RequestChatsMembersDelete with the chat_id and user_id
    /// - `chat_id` - [`ChatId`]
    /// - `user_id` - [`UserId`]
    pub fn add_member(&mut self, member: Sn) -> &mut Self {
        self.members.push(member);
        self
    }
}
