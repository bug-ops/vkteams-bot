pub mod keyboard;
pub mod parser;
#[cfg(feature = "templates")]
pub mod templates;
pub use crate::api::types::*;
pub use parser::*;

#[allow(unused_variables)]
pub trait MessageTextSetters {
    /// Set text
    /// ## Parameters
    /// - `parser`: [`MessageTextParser`] - Text parser
    fn set_text(self, parser: MessageTextParser) -> Self
    where
        Self: Sized + Clone,
    {
        warn!("Method not implemented");
        self.to_owned()
    }
    /// Set forward message ID
    /// ## Parameters
    /// - `chat_id`: [`ChatId`] - Chat ID
    /// - `msg_id`: [`MsgId`] - Message ID
    fn set_forward_msg_id(self, chat_id: ChatId, msg_id: MsgId) -> Self
    where
        Self: Sized + Clone,
    {
        warn!("Method not implemented");
        self.to_owned()
    }
    /// Set keyboard
    /// ## Parameters
    /// - `keyboard`: [`Keyboard`] - Keyboard
    fn set_keyboard(self, keyboard: Keyboard) -> Self
    where
        Self: Sized + Clone,
    {
        warn!("Method not implemented");
        self.to_owned()
    }
}
