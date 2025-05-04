#![allow(unused_parens)]
//! Answer callback query method `messages/answerCallbackQuery`
//! [More info](https://teams.vk.com/botapi/#/messages/get_messages_answerCallbackQuery)
use crate::prelude::*;
use serde::{Deserialize, Serialize};

bot_api_method! {
    method = "messages/answerCallbackQuery",
    request = RequestMessagesAnswerCallbackQuery {
        required {
            query_id: QueryId,
        },
        optional {
            text: String,
            show_alert: bool,
            url: String,
        }
    },
    response = ResponseMessagesAnswerCallbackQuery {
        ok: bool,
    },
}
