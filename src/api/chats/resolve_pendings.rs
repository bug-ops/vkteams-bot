//! Resolve pendings in chat method `chats/resolvePending`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_resolvePending)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
bot_api_method! {
    method   = "chats/resolvePending",
    request  = RequestChatsResolvePending {
        required {
            chat_id: ChatId,
            approve: bool,
        },
        optional {
            #[serde(skip_serializing_if = "Option::is_none")]
            user_id: Option<UserId>,
            #[serde(skip_serializing_if = "Option::is_none")]
            everyone: Option<bool>,
        }
    },
    response = ResponseChatsResolvePending {},
}
