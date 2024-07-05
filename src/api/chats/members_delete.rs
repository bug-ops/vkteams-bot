//! Delete members from the chat method `chats/members/delete`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_members_delete)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// # Delete members from the chat request method `chats/members/delete`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsMembersDelete {
    pub chat_id: ChatId,
    pub user_id: UserId,
    pub members: Vec<Sn>,
}
/// # Delete members from the chat response method `chats/members/delete`
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ResponseChatsMembersDelete {
    pub ok: bool,
}
impl BotRequest for RequestChatsMembersDelete {
    const METHOD: &'static str = "chats/members/delete";
    type RequestType = Self;
    type ResponseType = ResponseChatsMembersDelete;
}
impl RequestChatsMembersDelete {
    /// Create a new [`RequestChatsMembersDelete`]
    /// ## Parameters
    /// - `chat_id` - [`ChatId`]
    /// - `user_id` - [`UserId`]
    pub fn new(chat_id: ChatId, user_id: UserId) -> Self {
        Self {
            chat_id,
            user_id,
            ..Default::default()
        }
    }
    /// Create a new [`RequestChatsMembersDelete`]
    /// ## Parameters
    /// - `Sn` - [`Sn`]
    pub fn add_member(&mut self, member: Sn) -> &mut Self {
        self.members.push(member);
        self
    }
}
