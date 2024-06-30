use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsGetMembers`]
///
/// [`SendMessagesAPIMethods::ChatsGetMembers`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetMembers
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetMembers {
    pub chat_id: ChatId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<u32>,
}
/// Response for method [`SendMessagesAPIMethods::ChatsGetMembers`]
///
/// [`SendMessagesAPIMethods::ChatsGetMembers`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetMembers
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGetMembers {
    #[serde(default)]
    pub members: Vec<Member>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<u32>,
}
impl BotRequest for RequestChatsGetMembers {
    const METHOD: &'static str = "chats/getMembers";
    type RequestType = Self;
    type ResponseType = ResponseChatsGetMembers;
}
impl RequestChatsGetMembers {
    /// Create a new RequestChatsGetMembers with the chat_id
    /// - `chat_id` - [`ChatId`]
    pub fn new(chat_id: ChatId) -> Self {
        Self {
            chat_id,
            ..Default::default()
        }
    }
    /// Set cursor for the request
    /// - `cursor` - `u32`
    pub fn set_cursor(&mut self, cursor: u32) -> &mut Self {
        self.cursor = Some(cursor);
        self
    }
}
