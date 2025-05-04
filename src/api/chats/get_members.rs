#![allow(unused_parens)]
//! # Get chat members method `chats/getMembers`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_getMembers)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
bot_api_method! {
    method   = "chats/getMembers",
    request  = RequestChatsGetMembers {
        required {
            chat_id: ChatId,
        },
        optional {
            #[serde(skip_serializing_if = "Option::is_none")]
            cursor: Option<u32>,
        }
    },
    response = ResponseChatsGetMembers {
        #[serde(default)]
        members: Vec<Member>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cursor: Option<u32>,
    },
}
