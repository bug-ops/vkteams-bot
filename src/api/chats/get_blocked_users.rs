use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsGetBlockedUsers`]
///
/// [`SendMessagesAPIMethods::ChatsGetBlockedUsers`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetBlockedUsers
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetBlockedUsers {
    pub chat_id: ChatId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsGetBlockedUsers`]
///
/// [`SendMessagesAPIMethods::ChatsGetBlockedUsers`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetBlockedUsers
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGetBlockedUsers {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<Users>>,
}
impl BotRequest for RequestChatsGetBlockedUsers {
    const METHOD: &'static str = "chats/getBlockedUsers";
    type RequestType = Self;
    type ResponseType = ResponseChatsGetBlockedUsers;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::ChatsGetBlockedUsers(chat_id) => Self {
                chat_id: chat_id.to_owned(),
            },
            _ => panic!("Wrong API method for RequestChatsGetBlockedUsers"),
        }
    }
}
