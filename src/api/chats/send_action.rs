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
    fn new(method: &Methods) -> Self {
        match method {
            Methods::ChatsSendAction(chat_id, actions) => Self {
                chat_id: chat_id.to_owned(),
                actions: actions.to_owned(),
            },
            _ => panic!("Wrong API method for RequestChatsSendAction"),
        }
    }
}
