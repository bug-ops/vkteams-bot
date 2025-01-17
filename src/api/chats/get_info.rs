//! Get information about a chat method `chats/getInfo`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_getInfo)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// # Chats get info request method `chats/getInfo`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetInfo {
    pub chat_id: ChatId,
}
/// # Chats get info response method `chats/getInfo`
/// Response can be one of the following types:
/// - `private`: [`ResponseChatsPrivateGetInfo`]
/// - `group`: [`ResponseChatsGroupGetInfo`]
/// - `channel`: [`ResponseChatsChannelGetInfo`]
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
/// # Chats get info response method `chats/getInfo` for private chat
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsPrivateGetInfo {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub nick: Option<String>,
    pub about: Option<String>,
    pub is_bot: Option<bool>,
    pub language: Option<Languages>,
}
/// # Chats get info response method `chats/getInfo` for group chat
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGroupGetInfo {
    pub title: Option<String>,
    pub about: Option<String>,
    pub rules: Option<String>,
    pub invite_link: Option<String>,
    pub public: Option<bool>,
    pub join_moderation: Option<bool>,
}
/// # Chats get info response method `chats/getInfo` for channel chat
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsChannelGetInfo {
    pub title: Option<String>,
    pub about: Option<String>,
    pub rules: Option<String>,
    pub invite_link: Option<String>,
    pub public: Option<bool>,
    pub join_moderation: Option<bool>,
}
impl BotRequest for RequestChatsGetInfo {
    const METHOD: &'static str = "chats/getInfo";
    type RequestType = Self;
    type ResponseType = ResponseChatsGetInfo;
}
impl RequestChatsGetInfo {
    /// Create a new [`RequestChatsGetInfo`] with the chat_id
    /// ## Parameters
    /// - `chat_id` - [`ChatId`]
    pub fn new(chat_id: ChatId) -> Self {
        Self { chat_id }
    }
}
