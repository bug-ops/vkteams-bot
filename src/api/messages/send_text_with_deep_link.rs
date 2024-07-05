//! Send a text message with a deep link method `messages/sendTextWithDeepLink`
//! [More info](https://teams.vk.com/botapi/#/messages/post_messages_sendTextWithDeepLink)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// # Send a text message with a deep link method `messages/sendTextWithDeepLink`
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
/// # Send a text message with a deep link method `messages/sendTextWithDeepLink`
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
    /// Create a new [`RequestMessagesSendTextWithDeepLink`]
    /// ## Parameters
    /// - `chat_id`: [`ChatId`]
    /// - `deep_link`: `String`
    pub fn new(chat_id: ChatId, deep_link: String) -> Self {
        Self {
            chat_id,
            deep_link,
            ..Default::default()
        }
    }
}
impl MessageTextSetters for RequestMessagesSendTextWithDeepLink {
    /// Set text and parse mode
    /// ## Parameters
    /// - `parser`: [`MessageTextParser`]
    fn set_text(&mut self, parser: MessageTextParser) -> Self {
        let (text, parse_mode) = parser.parse();
        self.text = Some(text);
        self.parse_mode = Some(parse_mode);
        self.to_owned()
    }
    /// Set reply message id
    /// ## Parameters
    /// - `msg_id`: [`MsgId`]
    fn set_reply_msg_id(&mut self, msg_id: MsgId) -> Self {
        self.reply_msg_id = Some(msg_id);
        self.to_owned()
    }
    /// Set forward message id
    /// ## Parameters
    /// - `chat_id`: [`ChatId`]
    /// - `msg_id`: [`MsgId`]
    fn set_forward_msg_id(&mut self, chat_id: ChatId, msg_id: MsgId) -> Self {
        self.forward_chat_id = Some(chat_id);
        self.forward_msg_id = Some(msg_id);
        self.to_owned()
    }
    /// Set message keyboard
    /// ## Parameters
    /// - `keyboard`: [`Keyboard`]
    fn set_keyboard(&mut self, keyboard: Keyboard) -> Self {
        self.inline_keyboard_markup = Some(keyboard.into());
        self.to_owned()
    }
}
