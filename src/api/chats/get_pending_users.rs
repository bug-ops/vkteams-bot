use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsBlockUser`]
///
/// [`SendMessagesAPIMethods::ChatsBlockUser`]: enum.SendMessagesAPIMethods.html#variant.ChatsBlockUser
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetPendingUsers {
    pub chat_id: ChatId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsBlockUser`]
///
/// [`SendMessagesAPIMethods::ChatsBlockUser`]: enum.SendMessagesAPIMethods.html#variant.ChatsBlockUser
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGetPendingUsers {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<Users>>,
}
impl BotRequest for RequestChatsGetPendingUsers {
    const METHOD: &'static str = "chats/getPendingUsers";
    type RequestType = Self;
    type ResponseType = ResponseChatsGetPendingUsers;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::ChatsGetPendingUsers(chat_id) => Self {
                chat_id: chat_id.to_owned(),
            },
            _ => panic!("Wrong API method for RequestChatsGetPendingUsers"),
        }
    }
}
