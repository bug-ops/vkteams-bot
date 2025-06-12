//! Edit text messages method `messages/editText`
//! [More info](https://teams.vk.com/botapi/#/messages/get_messages_editText)
use crate::prelude::*;
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
    response = ResponseMessagesEditText {},
}

impl MessageTextSetters for RequestMessagesEditText {
    /// Set text
    /// ## Parameters
    /// - `parser`: [`MessageTextParser`]
    fn set_text(self, parser: MessageTextParser) -> Result<Self> {
        let (text, parse_mode) = parser.parse()?;
        Ok(self.with_text(text).with_parse_mode(parse_mode))
    }
    /// Set format
    /// ## Parameters
    /// - `format`: [`MessageFormat`]
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
        let req = RequestMessagesEditText::new((ChatId("c1".to_string()), MsgId("m1".to_string())));
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
        let req = RequestMessagesEditText::new((ChatId("c1".to_string()), MsgId("m1".to_string())));
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
        let req = RequestMessagesEditText::new((ChatId("c1".to_string()), MsgId("m1".to_string())));
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
