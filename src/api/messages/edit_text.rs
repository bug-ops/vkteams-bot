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
    fn new(method: &Methods) -> Self {
        match method {
            Methods::MessagesEditText(chat_id, msg_id) => Self {
                chat_id: chat_id.to_owned(),
                msg_id: msg_id.to_owned(),
                ..Default::default()
            },
            _ => panic!("Wrong API method for RequestMessagesEditText"),
        }
    }
}
impl MessageTextSetters for RequestMessagesEditText {
    fn set_text(&mut self, parser: Option<MessageTextParser>) -> &mut Self {
        match parser {
            Some(p) => {
                let (text, parse_mode) = p.parse();
                self.text = Some(text);
                self.parse_mode = Some(parse_mode);
                self
            }
            None => self,
        }
    }
    fn set_reply_msg_id(&mut self, _: Option<MsgId>) -> &mut Self {
        warn!("Reply message ID is not supported for edit message");
        self
    }
    fn set_forward_msg_id(&mut self, _: Option<ChatId>, _: Option<MsgId>) -> &mut Self {
        warn!("Forward message ID is not supported for edit message");
        self
    }
    fn set_keyboard(&mut self, keyboard: Option<Keyboard>) -> &mut Self {
        match keyboard {
            Some(k) => {
                self.inline_keyboard_markup = Some(k.into());
                self
            }
            None => self,
        }
    }
}
