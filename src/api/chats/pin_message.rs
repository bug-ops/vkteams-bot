//! Pin Message method in chat `chats/pinMessage`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_pinMessage)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/pinMessage",
    request  = RequestChatsPinMessage {
        required {
            chat_id: ChatId,
            msg_id: MsgId,
        },
        optional {}
    },
    response = ResponseChatsPinMessage {},
}
