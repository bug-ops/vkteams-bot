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
    #[serde(default)]
    pub description: String,
}
impl BotRequest for RequestChatsAvatarSet {
    const METHOD: &'static str = "chats/avatar/set";
    const HTTP_METHOD: HTTPMethod = HTTPMethod::POST;
    type RequestType = Self;
    type ResponseType = ResponseChatsAvatarSet;
}
impl RequestChatsAvatarSet {
    /// Create a new RequestChatsAvatarSet with the chat_id and multipart
    /// - `chat_id` - [`ChatId`]
    /// - `multipart` - [`MultipartName`]
    pub fn new(chat_id: ChatId, multipart: MultipartName) -> Self {
        Self { chat_id, multipart }
    }
}
