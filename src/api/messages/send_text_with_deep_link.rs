use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::MessagesSendTextWithDeepLink`]
///
/// [`SendMessagesAPIMethods::MessagesSendTextWithDeepLink`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendTextWithDeepLink
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesSendTextWithDeepLink {
    pub chat_id: ChatId,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    pub deep_link: String,
}
/// Response for method [`SendMessagesAPIMethods::MessagesSendTextWithDeepLink`]
///
/// [`SendMessagesAPIMethods::MessagesSendTextWithDeepLink`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendTextWithDeepLink
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMessagesSendTextWithDeepLink {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<MsgId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub ok: bool,
}
impl BotRequest for RequestMessagesSendTextWithDeepLink {
    const METHOD: &'static str = "messages/sendTextWithDeepLink";
    type RequestType = Self;
    type ResponseType = ResponseMessagesSendTextWithDeepLink;
}
impl RequestMessagesSendTextWithDeepLink {
    /// Create a new RequestMessagesSendTextWithDeepLink with the chat_id and deep_link
    /// - `chat_id` - [`ChatId`]
    /// - `deep_link` - `String`
    pub fn new(chat_id: ChatId, deep_link: String) -> Self {
        Self {
            chat_id,
            deep_link,
            ..Default::default()
        }
    }
}
impl MessageTextSetters for RequestMessagesSendTextWithDeepLink {
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
