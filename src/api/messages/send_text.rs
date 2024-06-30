use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::MessagesSendText`]
/// - chatId: [`ChatId`] - reuired
/// - text: String - required
/// - replyMsgId: [`MsgId`] - optional
/// - forwardChatId: [`ChatId`] - optional
/// - forwardMsgId: [`MsgId`] - optional
/// - inlineKeyboardMarkup: `Vec<MessageKeyboard>` - optional
/// - format: [`MessageFormat`] - optional (follow [`Tutorial-Text Formatting`] for more info)
/// - parseMode: [`ParseMode`] - optional (default: [`ParseMode::HTML`])
///
/// [`Tutorial-Text Formatting`]: https://teams.vk.com/botapi/tutorial/?lang=en
/// [`SendMessagesAPIMethods::MessagesSendText`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendText
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesSendText {
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
}
/// Response for method [`SendMessagesAPIMethods::MessagesSendText`]
///
/// [`SendMessagesAPIMethods::MessagesSendText`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendText
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMessagesSendText {
    #[serde(default)]
    pub msg_id: MsgId, //ok = True
    #[serde(default)]
    pub description: String, //ok = False
    pub ok: bool,
}
impl BotRequest for RequestMessagesSendText {
    const METHOD: &'static str = "messages/sendText";
    type RequestType = Self;
    type ResponseType = ResponseMessagesSendText;
}
impl RequestMessagesSendText {
    /// Create a new RequestMessagesSendText with the chat_id
    /// - `chat_id` - [`ChatId`]
    pub fn new(chat_id: ChatId) -> Self {
        Self {
            chat_id,
            ..Default::default()
        }
    }
}
impl MessageTextSetters for RequestMessagesSendText {
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
