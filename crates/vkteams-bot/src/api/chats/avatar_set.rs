//! Avatar set method `chats/avatar/set`
//! [More info](https://teams.vk.com/botapi/#/chats/post_chats_avatar_set)
use crate::api::types::*;
bot_api_method! {
    method      = "chats/avatar/set",
    http_method = HTTPMethod::POST,
    request     = RequestChatsAvatarSet {
        required {
            chat_id: ChatId,
            multipart: MultipartName,
        },
        optional {}
    },
    response = ResponseChatsAvatarSet {},
}
