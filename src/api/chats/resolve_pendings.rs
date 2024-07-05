//! Resolve pendings in chat method `chats/resolvePending`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_resolvePending)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// #Resolve pendings request method `chats/resolvePending`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsResolvePending {
    pub chat_id: ChatId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<UserId>,
    pub approve: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub everyone: Option<bool>,
}
/// # Resolve pendings response method `chats/resolvePending`
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsResolvePending {
    pub ok: bool,
}
impl BotRequest for RequestChatsResolvePending {
    const METHOD: &'static str = "chats/resolvePending";
    type RequestType = Self;
    type ResponseType = ResponseChatsResolvePending;
}
impl RequestChatsResolvePending {
    /// Create a new [`RequestChatsResolvePending`]
    /// ## Parameters
    /// - `chat_id` - [`ChatId`]
    /// - `approve` - [`bool`]
    pub fn new(chat_id: ChatId, approve: bool) -> Self {
        Self {
            chat_id,
            approve,
            ..Default::default()
        }
    }
    /// Set user_id for the request [`RequestChatsResolvePending`]
    /// ## Parameters
    /// - `user_id` - [`UserId`]
    pub fn set_user(&mut self, user_id: UserId) -> &mut Self {
        self.user_id = Some(user_id);
        self
    }
    /// Set everyone for the request [`RequestChatsResolvePending`]
    /// ## Parameters
    /// - `everyone` - `bool`
    pub fn set_everyone(&mut self, everyone: bool) -> &mut Self {
        self.everyone = Some(everyone);
        self
    }
}
