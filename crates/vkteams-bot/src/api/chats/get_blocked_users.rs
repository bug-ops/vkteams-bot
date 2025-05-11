#![allow(unused_parens)]
//! Get blocked users method `chats/getBlockedUsers`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_getBlockedUsers)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/getBlockedUsers",
    request  = RequestChatsGetBlockedUsers {
        required {
            chat_id: ChatId,
        },
        optional {}
    },
    response = ResponseChatsGetBlockedUsers {
        users: Vec<Users>,
    },
}
