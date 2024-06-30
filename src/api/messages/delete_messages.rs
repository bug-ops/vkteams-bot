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
}
impl RequestMessagesDeleteMessages {
    /// Create a new RequestMessagesDeleteMessages with the chat_id and msg_id
    /// - `chat_id` - [`ChatId`]
    /// - `msg_id` - [`MsgId`]
    pub fn new(chat_id: ChatId, msg_id: MsgId) -> Self {
        Self { chat_id, msg_id }
    }
}
