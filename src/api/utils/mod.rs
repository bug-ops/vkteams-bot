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
