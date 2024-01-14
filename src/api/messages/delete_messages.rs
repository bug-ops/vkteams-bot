use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::MessagesDeleteMessages`]
///
/// [`SendMessagesAPIMethods::MessagesDeleteMessages`]: enum.SendMessagesAPIMethods.html#variant.MessagesDeleteMessages
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesDeleteMessages {
    pub chat_id: ChatId,
    pub msg_id: MsgId,
}
/// Response for method [`SendMessagesAPIMethods::MessagesDeleteMessages`]
///
/// [`SendMessagesAPIMethods::MessagesDeleteMessages`]: enum.SendMessagesAPIMethods.html#variant.MessagesDeleteMessages
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ResponseMessagesDeleteMessages {
    pub ok: bool,
}
impl BotRequest for RequestMessagesDeleteMessages {
    const METHOD: &'static str = "messages/deleteMessages";
    type RequestType = Self;
    type ResponseType = ResponseMessagesDeleteMessages;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::MessagesDeleteMessages(chat_id, msg_id) => Self {
                chat_id: chat_id.to_owned(),
                msg_id: msg_id.to_owned(),
            },
            _ => panic!("Wrong API method for RequestMessagesDeleteMessages"),
        }
    }
}
