//! Send a file to a chat method `messages/sendFile`
//! [More info](https://teams.vk.com/botapi/#/messages/get_messages_sendFile)
use crate::prelude::*;
use serde::{Deserialize, Serialize};

bot_api_method! {
    method = "messages/sendFile",
    http_method = HTTPMethod::POST,
    request = RequestMessagesSendFile {
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
    response = ResponseMessagesSendFile {
        msg_id: Option<MsgId>,
        file_id: Option<String>,
    },
}

impl MessageTextSetters for RequestMessagesSendFile {
    /// Set the text of the message
    /// ## Parameters
    /// - `parser`: [`MessageTextParser`]
    fn set_text(self, parser: MessageTextParser) -> Self {
        let (text, parse_mode) = parser.parse();
        self.with_text(text).with_parse_mode(parse_mode)
    }
    /// Set forward message id
    /// ## Parameters
    /// - `chat_id`: [`ChatId`]
    /// - `msg_id`: [`MsgId`]
    fn set_forward_msg_id(self, chat_id: ChatId, msg_id: MsgId) -> Self {
        self.with_forward_chat_id(chat_id)
            .with_forward_msg_id(msg_id)
    }
    /// Set keyboard for the message
    /// ## Parameters
    /// - `keyboard`: [`Keyboard`]
    fn set_keyboard(self, keyboard: Keyboard) -> Self {
        self.with_inline_keyboard_markup(keyboard.into())
    }
}
