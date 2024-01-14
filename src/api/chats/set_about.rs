use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsSetAbout`]
///
/// [`SendMessagesAPIMethods::ChatsSetAbout`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetAbout
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsSetAbout {
    pub chat_id: ChatId,
    pub about: String,
}
/// Response for method [`SendMessagesAPIMethods::ChatsSetAbout`]
///
/// [`SendMessagesAPIMethods::ChatsSetAbout`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetAbout
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsSetAbout {
    pub ok: bool,
}
impl BotRequest for RequestChatsSetAbout {
    const METHOD: &'static str = "chats/setAbout";
    type RequestType = Self;
    type ResponseType = ResponseChatsSetAbout;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::ChatsSetAbout(chat_id, about) => Self {
                chat_id: chat_id.to_owned(),
                about: about.to_owned(),
            },
            _ => panic!("Wrong API method for RequestChatsSetAbout"),
        }
    }
}
