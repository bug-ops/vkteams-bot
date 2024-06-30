use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::MessagesSendVoice`]
///
/// [`SendMessagesAPIMethods::MessagesSendVoice`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendVoice
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesSendVoice {
    pub chat_id: ChatId,
    #[serde(skip_serializing_if = "Option::is_none", rename = "caption")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_msg_id: Option<MsgId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward_chat_id: Option<ChatId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward_msg_id: Option<MsgId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_keyboard_markup: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<MessageFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(skip)]
    pub multipart: MultipartName,
}
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
/// Response for method [`SendMessagesAPIMethods::MessagesSendVoice`]
///
/// [`SendMessagesAPIMethods::MessagesSendVoice`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendVoice
pub struct ResponseMessagesSendVoice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<MsgId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    pub ok: bool,
}
impl BotRequest for RequestMessagesSendVoice {
    const METHOD: &'static str = "messages.sendVoice";
    const HTTP_METHOD: HTTPMethod = HTTPMethod::POST;
    type RequestType = Self;
    type ResponseType = ResponseMessagesSendVoice;
    /// Get the file from the multipart
    fn get_file(&self) -> Option<MultipartName> {
        match self.multipart {
            MultipartName::File(..) | MultipartName::Image(..) => Some(self.multipart.to_owned()),
            _ => None,
        }
    }
}
impl RequestMessagesSendVoice {
    /// Create a new RequestMessagesSendVoice with the chat_id
    /// - `chat_id` - [`ChatId`]
    pub fn new(chat_id: ChatId) -> Self {
        Self {
            chat_id,
            ..Default::default()
        }
    }
}
impl MessageTextSetters for RequestMessagesSendVoice {
    fn set_text(&mut self, parser: MessageTextParser) -> Self {
        let (text, parse_mode) = parser.parse();
        self.text = Some(text);
        self.parse_mode = Some(parse_mode);
        self.to_owned()
    }
    fn set_reply_msg_id(&mut self, msg_id: MsgId) -> Self {
        self.reply_msg_id = Some(msg_id);
        self.to_owned()
    }
    fn set_forward_msg_id(&mut self, chat_id: ChatId, msg_id: MsgId) -> Self {
        self.forward_chat_id = Some(chat_id);
        self.forward_msg_id = Some(msg_id);
        self.to_owned()
    }
    fn set_keyboard(&mut self, keyboard: Keyboard) -> Self {
        self.inline_keyboard_markup = Some(keyboard.into());
        self.to_owned()
    }
}
