//! Send a file to a chat method `messages/sendFile`
//! [More info](https://teams.vk.com/botapi/#/messages/get_messages_sendFile)
use crate::prelude::*;

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
    fn set_text(self, parser: MessageTextParser) -> Result<Self> {
        let (text, parse_mode) = parser.parse()?;
        Ok(self.with_text(text).with_parse_mode(parse_mode))
    }
    /// Set forward message id
    /// ## Parameters
    /// - `chat_id`: [`ChatId`]
    /// - `msg_id`: [`MsgId`]
    fn set_forward_msg_id(self, chat_id: ChatId, msg_id: MsgId) -> Result<Self> {
        Ok(self
            .with_forward_chat_id(chat_id)
            .with_forward_msg_id(msg_id))
    }
    /// Set keyboard for the message
    /// ## Parameters
    /// - `keyboard`: [`Keyboard`]
    fn set_keyboard(self, keyboard: Keyboard) -> Result<Self> {
        Ok(self.with_inline_keyboard_markup(keyboard.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_text_valid() {
        let req = RequestMessagesSendFile::new((
            ChatId("c1".to_string()),
            MultipartName::File("f1".to_string()),
        ));
        let mut parser = MessageTextParser::default();
        parser
            .text
            .push(MessageTextFormat::Plain("hello".to_string()));
        let res = req.set_text(parser);
        assert!(res.is_ok());
        let req2 = res.unwrap();
        assert_eq!(req2.text.unwrap(), "hello");
    }

    #[test]
    fn test_set_text_parser_error() {
        let req = RequestMessagesSendFile::new((
            ChatId("c1".to_string()),
            MultipartName::File("f1".to_string()),
        ));
        let mut parser = MessageTextParser::default();
        parser.text.push(MessageTextFormat::Link(
            "not a url".to_string(),
            "text".to_string(),
        ));
        let res = req.set_text(parser);
        assert!(res.is_err());
    }

    #[test]
    fn test_set_keyboard_valid() {
        let req = RequestMessagesSendFile::new((
            ChatId("c1".to_string()),
            MultipartName::File("f1".to_string()),
        ));
        let keyboard = Keyboard {
            buttons: vec![vec![ButtonKeyboard {
                text: "ok".to_string(),
                url: None,
                callback_data: None,
                style: None,
            }]],
        };
        let res = req.set_keyboard(keyboard);
        assert!(res.is_ok());
        let req2 = res.unwrap();
        assert!(req2.inline_keyboard_markup.is_some());
    }

    #[test]
    fn test_set_forward_msg_id_valid() {
        let req = RequestMessagesSendFile::new((
            ChatId("c1".to_string()),
            MultipartName::File("f1".to_string()),
        ));
        let res = req.set_forward_msg_id(ChatId("c2".to_string()), MsgId("m1".to_string()));
        assert!(res.is_ok());
        let req2 = res.unwrap();
        assert_eq!(req2.forward_chat_id.unwrap().0, "c2");
        assert_eq!(req2.forward_msg_id.unwrap().0, "m1");
    }
}
