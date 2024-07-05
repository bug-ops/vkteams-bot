//! Delete messages method `messages/deleteMessages`
//! [More info](https://teams.vk.com/botapi/#/messages/get_messages_deleteMessages)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Delete messages method `messages/deleteMessages`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesDeleteMessages {
    pub chat_id: ChatId,
    pub msg_id: MsgId,
}
/// Delete messages method `messages/deleteMessages`
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
    /// Create a new [`RequestMessagesDeleteMessages`]
    /// ## Parameters
    /// - `chat_id`: [`ChatId`]
    /// - `msg_id`: [`MsgId`]
    pub fn new(chat_id: ChatId, msg_id: MsgId) -> Self {
        Self { chat_id, msg_id }
    }
}
