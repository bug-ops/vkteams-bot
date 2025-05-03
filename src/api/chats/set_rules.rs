//! Set rules for a chat method `chats/setRules`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_setRules)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
bot_api_method! {
    method   = "chats/setRules",
    request  = RequestChatsSetRules {
        required {
            chat_id: ChatId,
            rules: String,
        },
        optional {}
    },
    response = ResponseChatsSetRules {
        ok: bool,
    },
}
