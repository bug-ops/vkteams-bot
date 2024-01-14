use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsPinMessage`]
///
/// [`SendMessagesAPIMethods::ChatsPinMessage`]: enum.SendMessagesAPIMethods.html#variant.ChatsPinMessage
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsPinMessage {
    pub chat_id: ChatId,
    pub msg_id: MsgId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsPinMessage`]
///
/// [`SendMessagesAPIMethods::ChatsPinMessage`]: enum.SendMessagesAPIMethods.html#variant.ChatsPinMessage
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsPinMessage {
    pub ok: bool,
}
impl BotRequest for RequestChatsPinMessage {
    const METHOD: &'static str = "chats/pinMessage";
    type RequestType = Self;
    type ResponseType = ResponseChatsPinMessage;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::ChatsPinMessage(chat_id, msg_id) => Self {
                chat_id: chat_id.to_owned(),
                msg_id: msg_id.to_owned(),
            },
            _ => panic!("Wrong API method for RequestChatsPinMessage"),
        }
    }
}
