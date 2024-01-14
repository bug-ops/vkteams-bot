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
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGetInfo {
    #[serde(rename = "type")]
    pub chat_type: ChatType,
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nick: Option<String>,
    pub about: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_bot: Option<bool>,
    pub language: Languages,
    // FIXME: Separate this struct for different chat types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invite_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join_moderation: Option<bool>,
}
impl BotRequest for RequestChatsGetInfo {
    const METHOD: &'static str = "chats/getInfo";
    type RequestType = Self;
    type ResponseType = ResponseChatsGetInfo;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::ChatsGetInfo(chat_id) => Self {
                chat_id: chat_id.to_owned(),
            },
            _ => panic!("Wrong API method for RequestChatsGetInfo"),
        }
    }
}
