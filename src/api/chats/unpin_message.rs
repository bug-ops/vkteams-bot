//! Unpin Message method `chats/unpinMessage`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_unpinMessage)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
bot_api_method! {
    method   = "chats/unpinMessage",
    request  = RequestChatsUnpinMessage {
        required {
            chat_id: ChatId,
            msg_id: MsgId,
        },
        optional {}
    },
    response = ResponseChatsUnpinMessage {
        ok: bool,
    },
}
