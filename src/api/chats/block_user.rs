use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsBlockUser`]
///
///
/// [`SendMessagesAPIMethods::ChatsBlockUser`]: enum.SendMessagesAPIMethods.html#variant.ChatsBlockUser
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsBlockUser {
    pub chat_id: ChatId,
    pub user_id: UserId,
    pub del_last_messages: bool,
}
/// Response for method [`SendMessagesAPIMethods::ChatsBlockUser`]
///
/// [`SendMessagesAPIMethods::ChatsBlockUser`]: enum.SendMessagesAPIMethods.html#variant.ChatsBlockUser
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsBlockUser {
    pub ok: bool,
}
impl BotRequest for RequestChatsBlockUser {
    const METHOD: &'static str = "chats/blockUser";
    type RequestType = Self;
    type ResponseType = ResponseChatsBlockUser;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::ChatsBlockUser(chat_id, user_id, del_last_messages) => Self {
                chat_id: chat_id.to_owned(),
                user_id: user_id.to_owned(),
                del_last_messages: *del_last_messages,
            },
            _ => panic!("Wrong API method for RequestChatsBlockUser"),
        }
    }
}
