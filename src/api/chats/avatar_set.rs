//! Avatar set method `chats/avatar/set`
//! [More info](https://teams.vk.com/botapi/#/chats/post_chats_avatar_set)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// # Chat avatar set request
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsAvatarSet {
    pub chat_id: ChatId,
    #[serde(skip)]
    pub multipart: MultipartName,
}
/// # Chat avatar set response
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
    /// Create a new [`RequestChatsAvatarSet`]
    /// ## Parameters
    /// - `chat_id`: [`ChatId`]
    /// ## Body
    /// - `multipart`: [`MultipartName`]
    pub fn new(chat_id: ChatId, multipart: MultipartName) -> Self {
        Self { chat_id, multipart }
    }
}
