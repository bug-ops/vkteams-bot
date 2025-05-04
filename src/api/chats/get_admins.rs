#![allow(unused_parens)]
//! Get chat admins method `chats/getAdmins`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_getAdmins)
use crate::api::types::*;
use serde::{Deserialize, Serialize};

bot_api_method! {
    method   = "chats/getAdmins",
    request  = RequestChatsGetAdmins {
        required {
            chat_id: ChatId,
        },
        optional {}
    },
    response = ResponseChatsGetAdmins {
        admins: Vec<Admin>,
    },
}
