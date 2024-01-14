use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsResolvePending`]
///
/// [`SendMessagesAPIMethods::ChatsResolvePending`]: enum.SendMessagesAPIMethods.html#variant.ChatsResolvePending
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsResolvePending {
    pub chat_id: ChatId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<UserId>,
    pub approve: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub everyone: Option<bool>,
}
/// Response for method [`SendMessagesAPIMethods::ChatsResolvePending`]
///
/// [`SendMessagesAPIMethods::ChatsResolvePending`]: enum.SendMessagesAPIMethods.html#variant.ChatsResolvePending
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsResolvePending {
    pub ok: bool,
}
impl BotRequest for RequestChatsResolvePending {
    const METHOD: &'static str = "chats/resolvePending";
    type RequestType = Self;
    type ResponseType = ResponseChatsResolvePending;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::ChatsResolvePending(chat_id, approve, user_id, everyone) => Self {
                chat_id: chat_id.to_owned(),
                approve: approve.to_owned(),
                user_id: user_id.to_owned(),
                everyone: everyone.to_owned(),
            },
            _ => panic!("Wrong API method for RequestChatsResolvePending"),
        }
    }
}
