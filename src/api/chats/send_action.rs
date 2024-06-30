use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsSendActions`]
///
/// [`SendMessagesAPIMethods::ChatsSendActions`]: enum.SendMessagesAPIMethods.html#variant.ChatsSendActions
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsSendAction {
    pub chat_id: ChatId,
    pub actions: ChatActions,
}
/// Response for method [`SendMessagesAPIMethods::ChatsSendActions`]
///
/// [`SendMessagesAPIMethods::ChatsSendActions`]: enum.SendMessagesAPIMethods.html#variant.ChatsSendActions
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ResponseChatsSendAction {
    pub ok: bool,
}
impl BotRequest for RequestChatsSendAction {
    const METHOD: &'static str = "chats/sendActions";
    type RequestType = Self;
    type ResponseType = ResponseChatsSendAction;
}
impl RequestChatsSendAction {
    /// Create a new RequestChatsSendAction with the chat_id and actions
    /// - `chat_id` - [`ChatId`]
    /// - `actions` - [`ChatActions`]
    pub fn new(chat_id: ChatId, actions: ChatActions) -> Self {
        Self { chat_id, actions }
    }
}
