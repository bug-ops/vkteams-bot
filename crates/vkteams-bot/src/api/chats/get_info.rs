#![allow(unused_parens)]
//! Get information about a chat method `chats/getInfo`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_getInfo)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/getInfo",
    request  = RequestChatsGetInfo {
        required {
            chat_id: ChatId,
        },
        optional {}
    },
    response = ResponseChatsGetInfo {
        #[serde(flatten)]
        types: EnumChatsGetInfo
    },
}

/// # Chats get info response method `chats/getInfo`
/// Response can be one of the following types:
/// - `private`: [`ResponseChatsPrivateGetInfo`]
/// - `group`: [`ResponseChatsGroupGetInfo`]
/// - `channel`: [`ResponseChatsChannelGetInfo`]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EnumChatsGetInfo {
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
