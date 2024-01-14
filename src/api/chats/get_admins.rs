use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsGetAdmins`]
///
/// [`SendMessagesAPIMethods::ChatsGetAdmins`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetAdmins
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetAdmins {
    pub chat_id: ChatId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsGetAdmins`]
///
/// [`SendMessagesAPIMethods::ChatsGetAdmins`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetAdmins
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGetAdmins {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admins: Option<Vec<Admin>>,
}
impl BotRequest for RequestChatsGetAdmins {
    const METHOD: &'static str = "chats/getAdmins";
    type RequestType = Self;
    type ResponseType = ResponseChatsGetAdmins;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::ChatsGetAdmins(chat_id) => Self {
                chat_id: chat_id.to_owned(),
            },
            _ => panic!("Wrong API method for RequestChatsGetAdmins"),
        }
    }
}
