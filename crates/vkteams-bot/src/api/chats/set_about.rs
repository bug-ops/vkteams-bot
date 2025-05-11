//! Set a chat's about text method `chats/setAbout`
//! [Nore info](https://teams.vk.com/botapi/#/chats/get_chats_setAbout)
use crate::api::types::*;
bot_api_method! {
    method = "chats/setAbout",
    request = RequestChatsSetAbout {
        required {
            chat_id: ChatId,
            about: String,
        },
        optional {}
    },
    response = ResponseChatsSetAbout {},
}
