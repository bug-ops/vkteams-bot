//! Unblock User in chat method `chats/unblockUser`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_unblockUser)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
bot_api_method! {
    method   = "chats/unblockUser",
    request  = RequestChatsUnblockUser {
        required {
            chat_id: ChatId,
            user_id: UserId,
        },
        optional {}
    },
    response = ResponseChatsUnblockUser {},
}
