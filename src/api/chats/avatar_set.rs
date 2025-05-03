//! Avatar set method `chats/avatar/set`
//! [More info](https://teams.vk.com/botapi/#/chats/post_chats_avatar_set)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
bot_api_method! {
    method   = "chats/avatar/set",
    request  = RequestChatsAvatarSet {
        required {
        chat_id: ChatId,
        multipart: MultipartName,
        },
        optional {}
    },
    response = ResponseChatsAvatarSet {
        ok: bool,
        description: String,
    },
}
