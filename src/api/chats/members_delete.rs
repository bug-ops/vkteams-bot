use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsMembersDelete`]
///
/// [`SendMessagesAPIMethods::ChatsMembersDelete`]: enum.SendMessagesAPIMethods.html#variant.ChatsMembersDelete
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsMembersDelete {
    pub chat_id: ChatId,
    pub user_id: UserId,
    pub members: Vec<Sn>,
}
/// Response for method [`SendMessagesAPIMethods::ChatsMembersDelete`]
///
/// [`SendMessagesAPIMethods::ChatsMembersDelete`]: enum.SendMessagesAPIMethods.html#variant.ChatsMembersDelete
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ResponseChatsMembersDelete {
    pub ok: bool,
}
impl BotRequest for RequestChatsMembersDelete {
    const METHOD: &'static str = "chats/members/delete";
    type RequestType = Self;
    type ResponseType = ResponseChatsMembersDelete;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::ChatsMembersDelete(chat_id, user_id) => Self {
                chat_id: chat_id.to_owned(),
                user_id: user_id.to_owned(),
                ..Default::default()
            },
            _ => panic!("Wrong API method for RequestChatsMembersDelete"),
        }
    }
}
