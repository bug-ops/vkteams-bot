use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsSetRules`]
///
/// [`SendMessagesAPIMethods::ChatsSetRules`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetRules
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsSetRules {
    pub chat_id: ChatId,
    pub rules: String,
}
/// Response for method [`SendMessagesAPIMethods::ChatsSetRules`]
///
/// [`SendMessagesAPIMethods::ChatsSetRules`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetRules
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsSetRules {
    pub ok: bool,
}
impl BotRequest for RequestChatsSetRules {
    const METHOD: &'static str = "chats/setRules";
    type RequestType = Self;
    type ResponseType = ResponseChatsSetRules;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::ChatsSetRules(chat_id, rules) => Self {
                chat_id: chat_id.to_owned(),
                rules: rules.to_owned(),
            },
            _ => panic!("Wrong API method for RequestChatsSetRules"),
        }
    }
}
