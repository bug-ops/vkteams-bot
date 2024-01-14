use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsUnpinMessage`]
///
/// [`SendMessagesAPIMethods::ChatsUnpinMessage`]: enum.SendMessagesAPIMethods.html#variant.ChatsUnpinMessage
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsUnpinMessage {
    pub chat_id: ChatId,
    pub msg_id: MsgId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsUnpinMessage`]
///
/// [`SendMessagesAPIMethods::ChatsUnpinMessage`]: enum.SendMessagesAPIMethods.html#variant.ChatsUnpinMessage
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsUnpinMessage {
    pub ok: bool,
}
impl BotRequest for RequestChatsUnpinMessage {
    const METHOD: &'static str = "chats/unpinMessage";
    type RequestType = Self;
    type ResponseType = ResponseChatsUnpinMessage;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::ChatsUnpinMessage(chat_id, msg_id) => Self {
                chat_id: chat_id.to_owned(),
                msg_id: msg_id.to_owned(),
            },
            _ => panic!("Wrong API method for RequestChatsUnpinMessage"),
        }
    }
}
