#![allow(unused_parens)]
//! Send text messages to the chat method `messages/sendText`
//! [More info](https://teams.vk.com/botapi/#/messages/get_messages_sendText)
use crate::prelude::*;
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
    },
}

impl MessageTextSetters for RequestMessagesSendText {
    /// Set text and parse_mode
    /// ## Parameters
    /// - `parser`: [`MessageTextParser`]
    fn set_text(self, parser: MessageTextParser) -> Result<Self> {
        let (text, parse_mode) = parser.parse()?;
        Ok(self.with_text(text).with_parse_mode(parse_mode))
    }
    /// Set keyboard
    /// ## Parameters
    /// - `keyboard`: [`Keyboard`]
    fn set_keyboard(self, keyboard: Keyboard) -> Result<Self> {
        Ok(self.with_inline_keyboard_markup(keyboard.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::*;

    #[test]
    fn test_set_text_valid() {
        let req = RequestMessagesSendText::new(ChatId("c1".to_string()));
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
        let req = RequestMessagesSendText::new(ChatId("c1".to_string()));
        // Парсер с невалидным URL (ошибка парсинга)
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
        let req = RequestMessagesSendText::new(ChatId("c1".to_string()));
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
}
