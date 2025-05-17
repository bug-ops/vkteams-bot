//! Resolve pendings in chat method `chats/resolvePending`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_resolvePending)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/resolvePending",
    request  = RequestChatsResolvePending {
        required {
            chat_id: ChatId,
            approve: bool,
        },
        optional {
            user_id: UserId,
            everyone: bool,
        }
    },
    response = ResponseChatsResolvePending {},
}
