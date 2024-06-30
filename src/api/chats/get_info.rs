use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsGetInfo`]
///
/// [`SendMessagesAPIMethods::ChatsGetInfo`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetInfo
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetInfo {
    pub chat_id: ChatId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsGetInfo`]
///
/// [`SendMessagesAPIMethods::ChatsGetInfo`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetInfo
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ResponseChatsGetInfo {
    /// Private chat
    Private(ResponseChatsPrivateGetInfo),
    /// Group chat
    Group(ResponseChatsGroupGetInfo),
    /// Channel chat
    Channel(ResponseChatsChannelGetInfo),
    #[default]
    None,
}
/// Response for method [`SendMessagesAPIMethods::ChatsGetInfo`]
///
/// [`SendMessagesAPIMethods::ChatsGetInfo`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetInfo
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsPrivateGetInfo {
    pub first_name: String,
    pub last_name: String,
    pub nick: String,
    pub about: String,
    pub is_bot: bool,
    pub language: Languages,
}
/// Response for method [`SendMessagesAPIMethods::ChatsGetInfo`]
///
/// [`SendMessagesAPIMethods::ChatsGetInfo`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetInfo
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGroupGetInfo {
    pub title: String,
    pub about: String,
    pub rules: String,
    pub invite_link: String,
    pub public: bool,
    pub join_moderation: bool,
}
/// Response for method [`SendMessagesAPIMethods::ChatsGetInfo`]
///
/// [`SendMessagesAPIMethods::ChatsGetInfo`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetInfo
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsChannelGetInfo {
    pub title: String,
    pub about: String,
    pub rules: String,
    pub invite_link: String,
    pub public: bool,
    pub join_moderation: bool,
}

impl BotRequest for RequestChatsGetInfo {
    const METHOD: &'static str = "chats/getInfo";
    type RequestType = Self;
    type ResponseType = ResponseChatsGetInfo;
}
impl RequestChatsGetInfo {
    /// Create a new RequestChatsGetInfo with the chat_id
    /// - `chat_id` - [`ChatId`]
    pub fn new(chat_id: ChatId) -> Self {
        Self { chat_id }
    }
}
