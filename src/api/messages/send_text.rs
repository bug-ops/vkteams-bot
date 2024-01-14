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
/// - parseMode: [`ParseMode`] - optional (default: [`ParseMode::MarkdownV2`])
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<MsgId>, //ok = True
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>, //ok = False
    pub ok: bool,
}
impl BotRequest for RequestMessagesSendText {
    const METHOD: &'static str = "messages/sendText";
    type RequestType = Self;
    type ResponseType = ResponseMessagesSendText;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::MessagesSendText(chat_id) => Self {
                chat_id: chat_id.to_owned(),
                ..Default::default()
            },
            _ => panic!("Wrong API method for RequestMessagesSendText"),
        }
    }
}
impl MessageTextSetters for RequestMessagesSendText {
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
    fn set_reply_msg_id(&mut self, msg_id: Option<MsgId>) -> &mut Self {
        self.reply_msg_id = msg_id;
        self
    }
    fn set_forward_msg_id(&mut self, chat_id: Option<ChatId>, msg_id: Option<MsgId>) -> &mut Self {
        self.forward_chat_id = chat_id;
        self.forward_msg_id = msg_id;
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
