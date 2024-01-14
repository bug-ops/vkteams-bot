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
    fn new(method: &Methods) -> Self {
        match method {
            Methods::MessagesSendTextWithDeepLink(chat_id, deep_link) => Self {
                chat_id: chat_id.to_owned(),
                deep_link: deep_link.to_owned(),
                ..Default::default()
            },
            _ => panic!("Wrong API method for RequestMessagesSendTextWithDeepLink"),
        }
    }
}
impl MessageTextSetters for RequestMessagesSendTextWithDeepLink {
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
