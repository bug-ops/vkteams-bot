//! Send voice messages to a chat method `messages/sendVoice`
//! [More info](https://teams.vk.com/botapi/#/messages/get_messages_sendVoice)
use crate::prelude::*;
bot_api_method! {
    method = "messages/sendVoice",
    http_method = HTTPMethod::POST,
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
    },
}

impl MessageTextSetters for RequestMessagesSendVoice {
    /// Set the text of the message
    /// ## Parameters
    /// - `parser`: [`MessageTextParser`]
    fn set_text(self, parser: MessageTextParser) -> Result<Self> {
        let (text, parse_mode) = parser.parse()?;
        Ok(self.with_text(text).with_parse_mode(parse_mode))
    }
    /// Set the forward message id
    /// ## Parameters
    /// - `chat_id`: [`ChatId`]
    /// - `msg_id`: [`MsgId`]
    fn set_forward_msg_id(self, chat_id: ChatId, msg_id: MsgId) -> Result<Self> {
        Ok(self
            .with_forward_chat_id(chat_id)
            .with_forward_msg_id(msg_id))
    }
    /// Set the keyboard
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
        let req = RequestMessagesSendVoice::new((
            ChatId("c1".to_string()),
            MultipartName::FilePath("f1".to_string()),
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
        let req = RequestMessagesSendVoice::new((
            ChatId("c1".to_string()),
            MultipartName::FilePath("f1".to_string()),
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
        let req = RequestMessagesSendVoice::new((
            ChatId("c1".to_string()),
            MultipartName::FilePath("f1".to_string()),
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
        let req = RequestMessagesSendVoice::new((
            ChatId("c1".to_string()),
            MultipartName::FilePath("f1".to_string()),
        ));
        let res = req.set_forward_msg_id(ChatId("c2".to_string()), MsgId("m1".to_string()));
        assert!(res.is_ok());
        let req2 = res.unwrap();
        assert_eq!(req2.forward_chat_id.unwrap().0, "c2");
        assert_eq!(req2.forward_msg_id.unwrap().0, "m1");
    }

    #[test]
    fn test_serialize_deserialize_request_minimal() {
        let req = RequestMessagesSendVoice::new((
            ChatId("c1".to_string()),
            MultipartName::FilePath("voice_id".to_string()),
        ));
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["chatId"], "c1");
        assert_eq!(val["multipart"]["FilePath"], "voice_id");
        let req2: RequestMessagesSendVoice = serde_json::from_value(val).unwrap();
        assert_eq!(req2.chat_id.0, "c1");
        match req2.multipart {
            MultipartName::FilePath(ref s) => assert_eq!(s, "voice_id"),
            _ => panic!("Expected FilePath variant"),
        }
        assert!(req2.text.is_none());
    }

    #[test]
    fn test_serialize_deserialize_request_full() {
        let mut req = RequestMessagesSendVoice::new((
            ChatId("c1".to_string()),
            MultipartName::FilePath("voice_id".to_string()),
        ));
        req.text = Some("hello".to_string());
        let val = serde_json::to_value(&req).unwrap();
        let req2: RequestMessagesSendVoice = serde_json::from_value(val).unwrap();
        assert_eq!(req2.text.as_deref(), Some("hello"));
    }

    #[test]
    fn test_serialize_deserialize_response() {
        let resp = ResponseMessagesSendVoice {
            msg_id: Some(MsgId("m1".to_string())),
            file_id: Some("voice_id".to_string()),
        };
        let val = serde_json::to_value(&resp).unwrap();
        assert_eq!(val["msgId"], "m1");
        assert_eq!(val["fileId"], "voice_id");
        let resp2: ResponseMessagesSendVoice = serde_json::from_value(val).unwrap();
        assert_eq!(resp2.msg_id.as_ref().unwrap().0, "m1");
        assert_eq!(resp2.file_id.as_ref().unwrap(), "voice_id");
    }

    #[test]
    fn test_request_missing_required_field() {
        let val = serde_json::json!({"text": "hello"});
        let req = serde_json::from_value::<RequestMessagesSendVoice>(val);
        assert!(req.is_err());
    }
}
