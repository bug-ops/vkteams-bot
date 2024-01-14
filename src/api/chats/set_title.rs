use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsSetTitle`]
///
/// [`SendMessagesAPIMethods::ChatsSetTitle`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetTitle
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsSetTitle {
    pub chat_id: ChatId,
    pub title: String,
}
/// Response for method [`SendMessagesAPIMethods::ChatsSetTitle`]
///
/// [`SendMessagesAPIMethods::ChatsSetTitle`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetTitle
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsSetTitle {
    pub ok: bool,
}
impl BotRequest for RequestChatsSetTitle {
    const METHOD: &'static str = "chats/setTitle";
    type RequestType = Self;
    type ResponseType = ResponseChatsSetTitle;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::ChatsSetTitle(chat_id, title) => Self {
                chat_id: chat_id.to_owned(),
                title: title.to_owned(),
            },
            _ => panic!("Wrong API method for RequestChatsSetTitle"),
        }
    }
}
