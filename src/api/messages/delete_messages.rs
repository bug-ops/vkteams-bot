//! Delete messages method `messages/deleteMessages`
//! [More info](https://teams.vk.com/botapi/#/messages/get_messages_deleteMessages)
use crate::api::types::*;
use serde::{Deserialize, Serialize};

bot_api_method! {
    method = "messages/deleteMessages",
    request = RequestMessagesDeleteMessages {
        required {
            chat_id: ChatId,
            msg_id: MsgId,
        },
        optional {}
    },
    response = ResponseMessagesDeleteMessages {
        ok: bool,
    },
}
