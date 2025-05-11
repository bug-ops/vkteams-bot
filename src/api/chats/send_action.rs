//! Send chat actions method `chats/sendActions`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_sendActions)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/sendActions",
    request  = RequestChatsSendAction {
        required {
            chat_id: ChatId,
            actions: ChatActions,
        },
        optional {}
    },
    response = ResponseChatsSendAction {},
}
