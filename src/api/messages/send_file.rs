//! Send a file to a chat method `messages/sendFile`
//! [More info](https://teams.vk.com/botapi/#/messages/get_messages_sendFile)
use crate::prelude::*;
use serde::{Deserialize, Serialize};
/// Send a file to a chat method `messages/sendFile`
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesSendFile {
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
/// Send a file to a chat method `messages/sendFile`
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
    /// Get the file [`MultipartName`]
    fn get_file(&self) -> MultipartName {
        match self.multipart {
            MultipartName::File(..) | MultipartName::Image(..) => self.multipart.to_owned(),
            _ => MultipartName::None,
        }
    }
}
impl RequestMessagesSendFile {
    /// Create a new [`RequestMessagesSendFile`]
    /// ## Parameters
    /// - `chat_id`: [`ChatId`]
    /// - `multipart`: [`MultipartName`]
    pub fn new(chat_id: ChatId, multipart: MultipartName) -> Self {
        Self {
            chat_id,
            multipart,
            ..Default::default()
        }
    }
}
impl MessageTextSetters for RequestMessagesSendFile {
    /// Set the text of the message
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
    /// Set keyboard for the message
    /// ## Parameters
    /// - `keyboard`: [`Keyboard`]
    fn set_keyboard(&mut self, keyboard: Keyboard) -> Self {
        self.inline_keyboard_markup = Some(keyboard.into());
        self.to_owned()
    }
}
