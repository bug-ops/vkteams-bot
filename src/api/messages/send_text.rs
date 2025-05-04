#![allow(unused_parens)]
//! Send text messages to the chat method `messages/sendText`
//! [More info](https://teams.vk.com/botapi/#/messages/get_messages_sendText)
use crate::prelude::*;
use serde::{Deserialize, Serialize};

bot_api_method! {
    method = "messages/sendText",
    request = RequestMessagesSendText {
        required {
            chat_id: ChatId,
        },
        optional {
            text: String,
            reply_msg_id: MsgId,
            forward_chat_id: ChatId,
            forward_msg_id: MsgId,
            inline_keyboard_markup: String,
            format: MessageFormat,
            parse_mode: ParseMode,
        }
    },
    response = ResponseMessagesSendText {
        msg_id: MsgId,
        description: String,
        ok: bool,
    },
}

impl MessageTextSetters for RequestMessagesSendText {
    /// Set text and parse_mode
    /// ## Parameters
    /// - `parser`: [`MessageTextParser`]
    fn set_text(self, parser: MessageTextParser) -> Self {
        let (text, parse_mode) = parser.parse();
        trace!("Text: {}", text);
        self.with_text(text).with_parse_mode(parse_mode)
    }
    /// Set keyboard
    /// ## Parameters
    /// - `keyboard`: [`Keyboard`]
    fn set_keyboard(self, keyboard: Keyboard) -> Self {
        self.with_inline_keyboard_markup(keyboard.into())
    }
}
