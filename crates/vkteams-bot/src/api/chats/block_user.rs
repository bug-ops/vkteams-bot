//! Block User method `chats/blockUser`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_blockUser)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/blockUser",
    request  = RequestChatsBlockUser {
        required {
            chat_id: ChatId,
            user_id: UserId,
        },
        optional {
            del_last_messages: bool,
        }
    },
    response = ResponseChatsBlockUser {},
}
