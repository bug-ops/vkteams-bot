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

    #[test]
    fn test_set_text_valid() {
        let req = RequestMessagesSendText::new(ChatId::from("c1"));
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
        let req = RequestMessagesSendText::new(ChatId::from("c1"));
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
        let req = RequestMessagesSendText::new(ChatId::from("c1"));
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
    fn test_serialize_deserialize_request_minimal() {
        let req = RequestMessagesSendText::new(ChatId::from("c1"));
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["chatId"], "c1");
        let req2: RequestMessagesSendText = serde_json::from_value(val).unwrap();
        assert_eq!(req2.chat_id.0, "c1");
        assert!(req2.text.is_none());
    }

    #[test]
    fn test_serialize_deserialize_request_full() {
        let mut req = RequestMessagesSendText::new(ChatId::from("c1"));
        req.text = Some("hello".to_string());
        req.reply_msg_id = Some(MsgId("m1".to_string()));
        req.forward_chat_id = Some(ChatId::from("c2"));
        req.forward_msg_id = Some(MsgId("m2".to_string()));
        let val = serde_json::to_value(&req).unwrap();
        let req2: RequestMessagesSendText = serde_json::from_value(val).unwrap();
        assert_eq!(req2.text.as_deref(), Some("hello"));
        assert_eq!(req2.reply_msg_id.as_ref().unwrap().0, "m1");
        assert_eq!(req2.forward_chat_id.as_ref().unwrap().0, "c2");
        assert_eq!(req2.forward_msg_id.as_ref().unwrap().0, "m2");
    }

    #[test]
    fn test_serialize_deserialize_response() {
        let resp = ResponseMessagesSendText {
            msg_id: MsgId("m1".to_string()),
        };
        let val = serde_json::to_value(&resp).unwrap();
        assert_eq!(val["msgId"], "m1");
        let resp2: ResponseMessagesSendText = serde_json::from_value(val).unwrap();
        assert_eq!(resp2.msg_id.0, "m1");
    }

    #[test]
    fn test_request_missing_required_field() {
        let val = serde_json::json!({"text": "hello"});
        let req = serde_json::from_value::<RequestMessagesSendText>(val);
        assert!(req.is_err());
    }
}
