//! Send voice messages to a chat method `messages/sendVoice`
//! [More info](https://teams.vk.com/botapi/#/messages/get_messages_sendVoice)
use crate::prelude::*;
use serde::{Deserialize, Serialize};

bot_api_method! {
    method = "messages/sendVoice",
    request = RequestMessagesSendVoice {
        required {
            chat_id: ChatId,
            multipart: MultipartName,
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
    response = ResponseMessagesSendVoice {
        msg_id: Option<MsgId>,
        file_id: Option<String>,
        ok: bool,
    },
}

impl MessageTextSetters for RequestMessagesSendVoice {
    /// Set the text of the message
    /// ## Parameters
    /// - `parser`: [`MessageTextParser`]
    fn set_text(self, parser: MessageTextParser) -> Self {
        let (text, parse_mode) = parser.parse();
        self.with_text(text).with_parse_mode(parse_mode)
    }
    /// Set the forward message id
    /// ## Parameters
    /// - `chat_id`: [`ChatId`]
    /// - `msg_id`: [`MsgId`]
    fn set_forward_msg_id(self, chat_id: ChatId, msg_id: MsgId) -> Self {
        self.with_forward_chat_id(chat_id)
            .with_forward_msg_id(msg_id)
    }
    /// Set the keyboard
    /// ## Parameters
    /// - `keyboard`: [`Keyboard`]
    fn set_keyboard(self, keyboard: Keyboard) -> Self {
        self.with_inline_keyboard_markup(keyboard.into())
    }
}
