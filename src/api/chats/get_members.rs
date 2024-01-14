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
    pub cursor: Option<u64>,
}
/// Response for method [`SendMessagesAPIMethods::ChatsGetMembers`]
///
/// [`SendMessagesAPIMethods::ChatsGetMembers`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetMembers
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGetMembers {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<Member>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<u64>,
}
impl BotRequest for RequestChatsGetMembers {
    const METHOD: &'static str = "chats/getMembers";
    type RequestType = Self;
    type ResponseType = ResponseChatsGetMembers;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::ChatsGetMembers(chat_id) => Self {
                chat_id: chat_id.to_owned(),
                ..Default::default()
            },
            _ => panic!("Wrong API method for RequestChatsGetMembers"),
        }
    }
}
