use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::MessagesEditText`]
///
/// [`SendMessagesAPIMethods::MessagesEditText`]: enum.SendMessagesAPIMethods.html#variant.MessagesEditText
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
/// Response for method [`SendMessagesAPIMethods::MessagesEditText`]
///
/// [`SendMessagesAPIMethods::MessagesEditText`]: enum.SendMessagesAPIMethods.html#variant.MessagesEditText
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
    /// Create a new RequestMessagesEditText with the chat_id and msg_id
    /// - `chat_id` - [`ChatId`]
    /// - `msg_id` - [`MsgId`]
    pub fn new(chat_id: ChatId, msg_id: MsgId) -> Self {
        Self {
            chat_id,
            msg_id,
            ..Default::default()
        }
    }
}
impl MessageTextSetters for RequestMessagesEditText {
    fn set_text(&mut self, parser: MessageTextParser) -> Self {
        let (text, parse_mode) = parser.parse();
        self.text = Some(text);
        self.parse_mode = Some(parse_mode);
        self.to_owned()
    }
    fn set_keyboard(&mut self, keyboard: Keyboard) -> Self {
        self.inline_keyboard_markup = Some(keyboard.into());
        self.to_owned()
    }
}
