pub mod keyboard;
pub mod parser;
#[cfg(feature = "templates")]
pub mod templates;
pub use crate::api::types::*;
use crate::error::{BotError, Result};
pub use parser::*;
#[allow(unused_variables)]
pub trait MessageTextSetters {
    /// Set text
    /// ## Parameters
    /// - `parser`: [`MessageTextParser`] - Text parser
    fn set_text(self, parser: MessageTextParser) -> Result<Self>
    where
        Self: Sized + Clone,
    {
        Err(BotError::Validation("Method not implemented".to_string()))
    }
    /// Set forward message ID
    /// ## Parameters
    /// - `chat_id`: [`ChatId`] - Chat ID
    /// - `msg_id`: [`MsgId`] - Message ID
    fn set_forward_msg_id(self, chat_id: ChatId, msg_id: MsgId) -> Result<Self>
    where
        Self: Sized + Clone,
    {
        Err(BotError::Validation("Method not implemented".to_string()))
    }
    /// Set keyboard
    /// ## Parameters
    /// - `keyboard`: [`Keyboard`] - Keyboard
    fn set_keyboard(self, keyboard: Keyboard) -> Result<Self>
    where
        Self: Sized + Clone,
    {
        Err(BotError::Validation("Method not implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{ChatId, Keyboard, MsgId};
    use crate::api::utils::parser::MessageTextParser;

    #[derive(Clone, Debug)]
    struct Dummy;
    impl MessageTextSetters for Dummy {}

    #[test]
    fn test_set_text_default_returns_error() {
        let dummy = Dummy;
        let parser = MessageTextParser::default();
        let res = dummy.clone().set_text(parser);
        assert!(res.is_err());
        let err = res.unwrap_err();
        match err {
            crate::error::BotError::Validation(msg) => assert!(msg.contains("not implemented")),
            _ => panic!("Unexpected error type: {:?}", err),
        }
    }

    #[test]
    fn test_set_forward_msg_id_default_returns_error() {
        let dummy = Dummy;
        let res = dummy
            .clone()
            .set_forward_msg_id(ChatId("test".into()), MsgId("mid".into()));
        assert!(res.is_err());
        let err = res.unwrap_err();
        match err {
            crate::error::BotError::Validation(msg) => assert!(msg.contains("not implemented")),
            _ => panic!("Unexpected error type: {:?}", err),
        }
    }

    #[test]
    fn test_set_keyboard_default_returns_error() {
        let dummy = Dummy;
        let kb = Keyboard { buttons: vec![] };
        let res = dummy.clone().set_keyboard(kb);
        assert!(res.is_err());
        let err = res.unwrap_err();
        match err {
            crate::error::BotError::Validation(msg) => assert!(msg.contains("not implemented")),
            _ => panic!("Unexpected error type: {:?}", err),
        }
    }

    #[test]
    fn test_custom_impl_overrides_default() {
        #[derive(Clone, Debug)]
        struct Custom;
        impl MessageTextSetters for Custom {
            fn set_text(self, _parser: MessageTextParser) -> crate::error::Result<Self> {
                Ok(self)
            }
        }
        let custom = Custom;
        let parser = MessageTextParser::default();
        let res = custom.clone().set_text(parser);
        assert!(res.is_ok());
    }
}
