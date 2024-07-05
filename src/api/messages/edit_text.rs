//! Edit text messages method `messages/editText`
//! [More info](https://teams.vk.com/botapi/#/messages/get_messages_editText)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Edit text messages method `messages/editText`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesEditText {
    pub chat_id: ChatId,
    pub msg_id: MsgId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_keyboard_markup: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<MessageFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
}
/// Edit text messages method `messages/editText`
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ResponseMessagesEditText {
    pub ok: bool,
}
impl BotRequest for RequestMessagesEditText {
    const METHOD: &'static str = "messages/editText";
    type RequestType = Self;
    type ResponseType = ResponseMessagesEditText;
}
impl RequestMessagesEditText {
    /// Create a new [`RequestMessagesEditText`]
    /// ## Parameters
    /// - `chat_id`: [`ChatId`]
    /// - `msg_id`: [`MsgId`]
    pub fn new(chat_id: ChatId, msg_id: MsgId) -> Self {
        Self {
            chat_id,
            msg_id,
            ..Default::default()
        }
    }
}
impl MessageTextSetters for RequestMessagesEditText {
    /// Set text
    /// ## Parameters
    /// - `parser`: [`MessageTextParser`]
    fn set_text(&mut self, parser: MessageTextParser) -> Self {
        let (text, parse_mode) = parser.parse();
        self.text = Some(text);
        self.parse_mode = Some(parse_mode);
        self.to_owned()
    }
    /// Set format
    /// ## Parameters
    /// - `format`: [`MessageFormat`]
    fn set_keyboard(&mut self, keyboard: Keyboard) -> Self {
        self.inline_keyboard_markup = Some(keyboard.into());
        self.to_owned()
    }
}
