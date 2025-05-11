#![allow(unused_parens)]
//! Get pending users method `chats/getPendingUsers`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_getPendingUsers)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/getPendingUsers",
    request  = RequestChatsGetPendingUsers {
        required {
            chat_id: ChatId,
        },
        optional {}
    },
    response = ResponseChatsGetPendingUsers {
        users: Vec<Users>,
    },
}
