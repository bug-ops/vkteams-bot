//! Set chat title method `chats/setTitle`
//! [More Info](https://teams.vk.com/botapi/#/chats/get_chats_setTitle)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/setTitle",
    request  = RequestChatsSetTitle {
        required {
            chat_id: ChatId,
            title: String,
        },
        optional {}
    },
    response = ResponseChatsSetTitle {},
}
