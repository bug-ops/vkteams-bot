//! Edit text messages method `messages/editText`
//! [More info](https://teams.vk.com/botapi/#/messages/get_messages_editText)
use crate::prelude::*;
use serde::{Deserialize, Serialize};

bot_api_method! {
    method = "messages/editText",
    request = RequestMessagesEditText {
        required {
            chat_id: ChatId,
            msg_id: MsgId,
        },
        optional {
            text: String,
            inline_keyboard_markup: String,
            format: MessageFormat,
            parse_mode: ParseMode,
        }
    },
    response = ResponseMessagesEditText {
        ok: bool,
    },
}

impl MessageTextSetters for RequestMessagesEditText {
    /// Set text
    /// ## Parameters
    /// - `parser`: [`MessageTextParser`]
    fn set_text(self, parser: MessageTextParser) -> Self {
        let (text, parse_mode) = parser.parse();
        self.with_text(text).with_parse_mode(parse_mode)
    }
    /// Set format
    /// ## Parameters
    /// - `format`: [`MessageFormat`]
    fn set_keyboard(self, keyboard: Keyboard) -> Self {
        self.with_inline_keyboard_markup(keyboard.into())
    }
}
