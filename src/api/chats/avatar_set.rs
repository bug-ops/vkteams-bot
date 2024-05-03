use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::ChatsAvatarSet`]
///
/// [`SendMessagesAPIMethods::ChatsAvatarSet`]: enum.SendMessagesAPIMethods.html#variant.ChatsAvatarSet
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsAvatarSet {
    pub chat_id: ChatId,
    #[serde(skip)]
    pub multipart: MultipartName,
}
/// Response for method [`SendMessagesAPIMethods::ChatsAvatarSet`]
///
/// [`SendMessagesAPIMethods::ChatsAvatarSet`]: enum.SendMessagesAPIMethods.html#variant.ChatsAvatarSet
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsAvatarSet {
    pub ok: bool,
    pub description: Option<String>,
}
impl BotRequest for RequestChatsAvatarSet {
    const METHOD: &'static str = "chats/avatar/set";
    const HTTP_METHOD: HTTPMethod = HTTPMethod::POST;
    type RequestType = Self;
    type ResponseType = ResponseChatsAvatarSet;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::ChatsAvatarSet(chat_id, multipart) => Self {
                chat_id: chat_id.to_owned(),
                multipart: multipart.to_owned(),
            },
            _ => panic!("Wrong API method for RequestChatsAvatarSet"),
        }
    }
    fn get_file(&self) -> Option<MultipartName> {
        match self.multipart {
            MultipartName::File(..) | MultipartName::Image(..) => Some(self.multipart.to_owned()),
            _ => None,
        }
    }
}
