use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsUnblockUser`]
///
/// [`SendMessagesAPIMethods::ChatsResolvePending`]: enum.SendMessagesAPIMethods.html#variant.ChatsUnblockUser
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsUnblockUser {
    pub chat_id: ChatId,
    pub user_id: UserId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsUnblockUser`]
///
/// [`SendMessagesAPIMethods::ChatsResolvePending`]: enum.SendMessagesAPIMethods.html#variant.ChatsUnblockUser
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsUnblockUser {
    pub ok: bool,
}
impl BotRequest for RequestChatsUnblockUser {
    const METHOD: &'static str = "chats/unblockUser";
    type RequestType = Self;
    type ResponseType = ResponseChatsUnblockUser;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::ChatsUnblockUser(chat_id, user_id) => Self {
                chat_id: chat_id.to_owned(),
                user_id: user_id.to_owned(),
            },
            _ => panic!("Wrong API method for RequestChatsUnblockUser"),
        }
    }
}
