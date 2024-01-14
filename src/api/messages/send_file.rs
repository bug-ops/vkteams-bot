use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::MessagesSendFile`]
///
/// [`SendMessagesAPIMethods::MessagesSendFile`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendFile
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesSendFile {
    pub chat_id: ChatId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
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
/// Response for method [`SendMessagesAPIMethods::MessagesSendFile`]
///
/// [`SendMessagesAPIMethods::MessagesSendFile`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendFile
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMessagesSendFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<MsgId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    pub ok: bool,
}
impl BotRequest for RequestMessagesSendFile {
    const METHOD: &'static str = "messages/sendFile";
    const HTTP_METHOD: HTTPMethod = HTTPMethod::POST;
    type RequestType = Self;
    type ResponseType = ResponseMessagesSendFile;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::MessagesSendFile(chat_id, multipart) => Self {
                chat_id: chat_id.to_owned(),
                multipart: multipart.to_owned(),
                ..Default::default()
            },
            _ => panic!("Wrong API method for RequestMessagesSendFile"),
        }
    }
    fn get_file(&self) -> Option<MultipartName> {
        match self.multipart {
            MultipartName::File(..) | MultipartName::Image(..) => Some(self.multipart.to_owned()),
            _ => None,
        }
    }
}
impl MessageTextSetters for RequestMessagesSendFile {
    fn set_text(&mut self, parser: Option<MessageTextParser>) -> &mut Self {
        match parser {
            Some(p) => {
                let (text, parse_mode) = p.parse();
                self.caption = Some(text);
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
