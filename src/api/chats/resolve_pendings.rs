use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsResolvePending`]
///
/// [`SendMessagesAPIMethods::ChatsResolvePending`]: enum.SendMessagesAPIMethods.html#variant.ChatsResolvePending
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
/// Response for method [`SendMessagesAPIMethods::ChatsResolvePending`]
///
/// [`SendMessagesAPIMethods::ChatsResolvePending`]: enum.SendMessagesAPIMethods.html#variant.ChatsResolvePending
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
    /// Create a new RequestChatsResolvePending with the chat_id and approve
    /// - `chat_id` - [`ChatId`]
    /// - `approve` - [`bool`]
    pub fn new(chat_id: ChatId, approve: bool) -> Self {
        Self {
            chat_id,
            approve,
            ..Default::default()
        }
    }
    /// Set user_id for the request
    /// - `user_id` - [`UserId`]
    pub fn set_user(&mut self, user_id: UserId) -> &mut Self {
        self.user_id = Some(user_id);
        self
    }
    /// Set everyone for the request
    /// - `everyone` - `bool`
    pub fn set_everyone(&mut self, everyone: bool) -> &mut Self {
        self.everyone = Some(everyone);
        self
    }
}
