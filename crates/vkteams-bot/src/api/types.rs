//! API types
use crate::error::{ApiError, BotError, Result};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt::*;
use std::time::Duration;
#[cfg(feature = "templates")]
use tera::{Context, Tera};
use tracing::debug;

/// Environment variable name for bot API URL
pub const VKTEAMS_BOT_API_URL: &str = "VKTEAMS_BOT_API_URL";
/// Environment variable name for bot API token
pub const VKTEAMS_BOT_API_TOKEN: &str = "VKTEAMS_BOT_API_TOKEN";
/// Environment variable name for bot Proxy URL
pub const VKTEAMS_PROXY: &str = "VKTEAMS_PROXY";
/// Timeout for long polling
pub const POLL_TIME: u64 = 30;
/// Global timeout for [`reqwest::Client`]
/// [`reqwest::Client`]: https://docs.rs/reqwest/latest/reqwest/struct.Client.html
pub const POLL_DURATION: &Duration = &Duration::from_secs(POLL_TIME + 10);

pub const SERVICE_NAME: &str = "BOT";
/// Supported API versions
#[derive(Debug)]
pub enum APIVersionUrl {
    /// default V1
    V1,
}
/// Supported API HTTP methods
#[derive(Debug, Default)]
pub enum HTTPMethod {
    #[default]
    GET,
    POST,
}

#[derive(Debug, Default)]
pub enum HTTPBody {
    // JSON,
    MultiPart(MultipartName),
    #[default]
    None,
}
/// Bot request trait
pub trait BotRequest {
    type Args;

    const METHOD: &'static str;
    const HTTP_METHOD: HTTPMethod = HTTPMethod::GET;
    type RequestType: Serialize + Debug + Default;
    type ResponseType: Serialize + DeserializeOwned + Debug + Default;
    fn get_multipart(&self) -> &MultipartName;
    fn new(args: Self::Args) -> Self;
    fn get_chat_id(&self) -> Option<&ChatId>;
}
/// API event id type
pub type EventId = u32;
/// Message text struct
#[derive(Serialize, Clone, Debug)]
pub enum MessageTextFormat {
    /// Plain text
    Plain(String),
    /// Bold text
    Bold(String),
    /// Italic text
    Italic(String),
    /// Underline text
    Underline(String),
    /// Strikethrough text
    Strikethrough(String),
    /// Inline URL
    Link(String, String),
    /// Inline mention of a user
    Mention(ChatId),
    /// Code formatted text
    Code(String),
    /// Pre-formatted fixed-width test block
    Pre(String, Option<String>),
    /// Ordered list
    OrderedList(Vec<String>),
    /// Unordered list
    UnOrderedList(Vec<String>),
    /// Quote text
    Quote(String),
    None,
}
/// Message text parse struct
#[derive(Default, Clone, Debug)]
pub struct MessageTextParser {
    /// Array of text formats
    //TODO: Add support for multiple text formats in one row
    pub text: Vec<MessageTextFormat>,
    // Context for templates
    #[cfg(feature = "templates")]
    pub ctx: Context,
    // Template name
    #[cfg(feature = "templates")]
    pub name: String,
    // Tera template engine
    #[cfg(feature = "templates")]
    pub tmpl: Tera,
    /// ## Parse mode
    /// - `HTML` - HTML
    /// - `MarkdownV2` - Markdown
    pub parse_mode: ParseMode,
}
/// Keyboard for send message methods
/// One of variants must be set:
/// - {`text`: String,`url`: String,`style`: [`ButtonStyle`]} - simple buttons
/// - {`text`: String,`callback_data`: String,`style`: [`ButtonStyle`]} - buttons with callback
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ButtonKeyboard {
    pub text: String, // formatting is not supported
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ButtonStyle>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
/// Array of keyboard buttons
pub struct Keyboard {
    pub buttons: Vec<Vec<ButtonKeyboard>>,
}
/// Keyboard buttons style
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ButtonStyle {
    Primary,
    Attention,
    #[default]
    Base,
}
/// Message text format parse mode
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub enum ParseMode {
    MarkdownV2,
    #[default]
    HTML,
    #[cfg(feature = "templates")]
    Template,
}
/// Event message
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventMessage {
    pub event_id: EventId,
    #[serde(flatten)]
    pub event_type: EventType,
}
/// Event types
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type", content = "payload")]
pub enum EventType {
    NewMessage(Box<EventPayloadNewMessage>),
    EditedMessage(Box<EventPayloadEditedMessage>),
    DeleteMessage(Box<EventPayloadDeleteMessage>),
    PinnedMessage(Box<EventPayloadPinnedMessage>),
    UnpinnedMessage(Box<EventPayloadUnpinnedMessage>),
    NewChatMembers(Box<EventPayloadNewChatMembers>),
    LeftChatMembers(Box<EventPayloadLeftChatMembers>),
    CallbackQuery(Box<EventPayloadCallbackQuery>),
    #[default]
    None,
}
/// Message payload event type newMessage
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventPayloadNewMessage {
    pub msg_id: MsgId,
    #[serde(default)]
    pub text: String,
    pub chat: Chat,
    pub from: From,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<MessageFormat>,
    #[serde(default)]
    pub parts: Vec<MessageParts>,
    pub timestamp: Timestamp,
}
/// Message payload event type editedMessage
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventPayloadEditedMessage {
    pub msg_id: MsgId,
    pub text: String,
    pub timestamp: Timestamp,
    pub chat: Chat,
    pub from: From,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<MessageFormat>,
    pub edited_timestamp: Timestamp,
}
/// Message payload event type deleteMessage
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventPayloadDeleteMessage {
    pub msg_id: MsgId,
    pub chat: Chat,
    pub timestamp: Timestamp,
}
/// Message payload event type pinnedMessage
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventPayloadPinnedMessage {
    pub msg_id: MsgId,
    pub chat: Chat,
    pub from: From,
    #[serde(default)]
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<MessageFormat>,
    #[serde(default)]
    pub parts: Vec<MessageParts>, //FIXME API response conflict with documentation
    pub timestamp: Timestamp,
}
/// Message payload event type unpinnedMessage
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventPayloadUnpinnedMessage {
    pub msg_id: MsgId,
    pub chat: Chat,
    pub timestamp: Timestamp,
}
/// Message payload event type newChatMembers
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventPayloadNewChatMembers {
    pub chat: Chat,
    pub new_members: Vec<From>,
    pub added_by: From,
}
/// Message payload event type leftChatMembers
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventPayloadLeftChatMembers {
    pub chat: Chat,
    pub left_members: Vec<From>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed_by: Option<From>,
}
/// Callback query event type
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventPayloadCallbackQuery {
    pub query_id: QueryId,
    pub from: From,
    #[serde(default)]
    pub chat: Chat,
    pub message: EventPayloadNewMessage,
    pub callback_data: String,
}
/// Message parts
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageParts {
    #[serde(rename = "type", flatten)]
    pub part_type: MessagePartsType,
    // pub payload: MessagePartsPayload,
}
/// Message parts type
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type", content = "payload")]
pub enum MessagePartsType {
    Sticker(MessagePartsPayloadSticker),
    Mention(MessagePartsPayloadMention),
    Voice(MessagePartsPayloadVoice),
    File(Box<MessagePartsPayloadFile>),
    Forward(Box<MessagePartsPayloadForward>),
    Reply(Box<MessagePartsPayloadReply>),
    // FIXME API response conflict with documentation
    InlineKeyboardMarkup(Vec<Vec<MessagePartsPayloadInlineKeyboard>>),
}
/// Message parts payload sticker
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessagePartsPayloadSticker {
    pub file_id: FileId,
}
/// Message parts payload mention
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessagePartsPayloadMention {
    #[serde(flatten)]
    pub user_id: From,
}
/// Message parts payload voice
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessagePartsPayloadVoice {
    pub file_id: FileId,
}
/// Message parts payload file
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessagePartsPayloadFile {
    pub file_id: FileId,
    #[serde(rename = "type", default)]
    pub file_type: String,
    #[serde(default)]
    pub caption: String,
    #[serde(default)]
    pub format: MessageFormat,
}
/// Message parts payload forward
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessagePartsPayloadForward {
    message: MessagePayload,
}
/// Message parts payload reply
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessagePartsPayloadReply {
    message: MessagePayload,
}
/// Message parts payload inline keyboard
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessagePartsPayloadInlineKeyboard {
    #[serde(default)]
    pub callback_data: String,
    pub style: ButtonStyle,
    pub text: String,
    #[serde(default)]
    pub url: String,
}
/// Array of message formats
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageFormat {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<Vec<MessageFormatStruct>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<Vec<MessageFormatStruct>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underline: Option<Vec<MessageFormatStruct>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<Vec<MessageFormatStruct>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<Vec<MessageFormatStructLink>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mention: Option<Vec<MessageFormatStruct>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_code: Option<Vec<MessageFormatStruct>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre: Option<Vec<MessageFormatStructPre>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ordered_list: Option<Vec<MessageFormatStruct>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<Vec<MessageFormatStruct>>,
}
/// Message format struct
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageFormatStruct {
    pub offset: i32,
    pub length: i32,
}
/// Message format struct for link
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageFormatStructLink {
    pub offset: i32,
    pub length: i32,
    pub url: String,
}
/// Message format struct
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageFormatStructPre {
    pub offset: i32,
    pub length: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}
/// Event message payload
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessagePayload {
    pub from: From,
    pub msg_id: MsgId,
    #[serde(default)]
    pub text: String,
    pub timestamp: u64,
    #[serde(default)]
    pub parts: Vec<MessageParts>,
}
/// Chat id struct
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Hash, Eq)]
pub struct ChatId(pub String);
/// Message id struct
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Hash, Eq)]
pub struct MsgId(pub String);
/// User id struct
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Hash, Eq)]
pub struct UserId(pub String);
/// File id struct
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Hash, Eq)]
pub struct FileId(pub String);
/// Query id struct
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Hash, Eq)]
pub struct QueryId(pub String);
/// Timestamp struct
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Hash, Eq)]
pub struct Timestamp(pub u32);
/// Chat struct
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    pub chat_id: ChatId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub chat_type: String,
}
/// From struct
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct From {
    pub first_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>, //if its a bot, then it will be EMPTY
    pub user_id: UserId,
}
/// Languages
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum Languages {
    #[default]
    Ru,
    En,
}
/// Chat types
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum ChatType {
    #[default]
    Private,
    Group,
    Channel,
}
/// Chat actions
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ChatActions {
    Looking,
    #[default]
    Typing,
}
/// Multipart name
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub enum MultipartName {
    File(String),
    Image(String),
    #[default]
    None,
}
/// Admin struct
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Admin {
    pub user_id: UserId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<bool>,
}
/// User struct
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Users {
    pub user_id: UserId,
}
/// Member struct
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub user_id: UserId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin: Option<bool>,
}
/// Sn struct for members
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Sn {
    pub sn: String,
    pub user_id: UserId,
}
/// Photo url struct
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PhotoUrl {
    pub url: String,
}
// Intermediate structure for deserializing API responses with the "ok" field
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ApiResponseWrapper<T> {
    PayloadWithOk {
        ok: bool,
        #[serde(flatten)]
        payload: T,
    },
    PayloadOnly(T),
    Error {
        ok: bool,
        description: String,
    },
}

// Implementation of From for automatic conversion from ApiResponseWrapper to Result
impl<T> std::convert::From<ApiResponseWrapper<T>> for Result<T>
where
    T: Default + Serialize + DeserializeOwned,
{
    fn from(wrapper: ApiResponseWrapper<T>) -> Self {
        match wrapper {
            ApiResponseWrapper::PayloadWithOk { ok, payload } => {
                if ok {
                    debug!("Answer is ok, payload received");
                    Ok(payload)
                } else {
                    debug!("Answer is not ok, but description is not provided");
                    Err(BotError::Api(ApiError {
                        description: "Unspecified error".to_string(),
                    }))
                }
            }
            ApiResponseWrapper::PayloadOnly(payload) => {
                debug!("Answer is ok, payload received");
                Ok(payload)
            }
            ApiResponseWrapper::Error { ok, description } => {
                if ok {
                    debug!("Answer is ok, BUT error description is provided");
                } else {
                    debug!("Answer is NOT ok and error description is provided");
                }
                Err(BotError::Api(ApiError { description }))
            }
        }
    }
}

/// Display trait for [`ChatId`]
impl Display for ChatId {
    /// Format [`ChatId`] to string
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
/// Link basse path for API version
impl Display for APIVersionUrl {
    /// Format [`APIVersionUrl`] to string
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            APIVersionUrl::V1 => write!(f, "bot/v1/"),
        }
    }
}
/// Display trait for [`MultipartName`] enum
impl Display for MultipartName {
    /// Format [`MultipartName`] to string
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            MultipartName::File(..) => write!(f, "file"),
            MultipartName::Image(..) => write!(f, "image"),
            _ => write!(f, ""),
        }
    }
}

/// Default values for [`Keyboard`]
impl Default for Keyboard {
    /// Create new [`Keyboard`] with required params
    fn default() -> Self {
        Self {
            // Empty vector of [`KeyboardButton`]
            buttons: vec![vec![]],
        }
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_event_message_serde() {
        // Пример из chats_events_get.json (raw string, без экранирования)
        let json = r#"{
            "eventId": 1312,
            "payload": {
                "chat": {
                    "chatId": "87654@chat.agent",
                    "title": "TEST",
                    "type": "group"
                },
                "from": {
                    "firstName": "Bob",
                    "lastName": "Smith",
                    "userId": "bob.smoth@example.com"
                },
                "msgId": "7513183...869",
                "parts": [
                    {
                        "payload": {
                            "message": {
                                "from": {
                                    "firstName": "Alice",
                                    "lastName": "Johnson",
                                    "userId": "alice.johnson@example.com"
                                },
                                "msgId": "7513183580768441700",
                                "text": "Hello, Bob! 👋 \n\nToday is Saturday, June 1, 2999.",
                                "timestamp": 1749299369
                            }
                        },
                        "type": "reply"
                    }
                ],
                "text": "Please send me an abstract picture",
                "timestamp": 1749427
            },
            "type": "newMessage"
        }"#;
        let msg = serde_json::from_str::<EventMessage>(json);
        assert!(msg.is_ok(), "Deserialization failed: {:?}", msg.err());
        let msg = msg.unwrap();
        assert_eq!(msg.event_id, 1312);
    }

    #[test]
    fn test_chatid_display() {
        let id = ChatId("abc".to_string());
        assert_eq!(format!("{}", id), "abc");
    }

    #[test]
    fn test_userid_display() {
        let id = UserId("u123".to_string());
        assert_eq!(format!("{}", id), "u123");
    }

    #[test]
    fn test_api_response_wrapper_from_ok() {
        let payload = 42u32;
        let wrapper = ApiResponseWrapper::PayloadWithOk { ok: true, payload };
        let res: Result<u32> = wrapper.into();
        assert_eq!(res.unwrap(), 42);
    }

    #[test]
    fn test_api_response_wrapper_from_error() {
        let wrapper: ApiResponseWrapper<u32> = ApiResponseWrapper::Error {
            ok: false,
            description: "fail".to_string(),
        };
        let res: Result<u32> = wrapper.into();
        assert!(res.is_err());
    }

    #[test]
    fn test_keyboard_default() {
        let k = Keyboard::default();
        assert!(k.buttons.is_empty() || !k.buttons.is_empty());
    }

    #[test]
    fn test_event_type_default() {
        let e = EventType::default();
        matches!(e, EventType::None);
    }

    #[test]
    fn test_parsemode_default() {
        let m = ParseMode::default();
        matches!(m, ParseMode::HTML);
    }

    #[test]
    fn test_eventtype_serde() {
        let json = r#"{
            "type": "newMessage",
            "payload": {
                "msgId": "m1",
                "text": "hi",
                "chat": { "chatId": "c1", "type": "private" },
                "from": { "firstName": "A", "userId": "u1" },
                "timestamp": 123,
                "parts": []
            }
        }"#;
        let e = serde_json::from_str::<EventType>(json);
        assert!(e.is_ok(), "{:?}", e.err());
    }

    #[test]
    fn test_messageparts_serde() {
        let json = r#"{
            "type": "sticker",
            "payload": { "fileId": "f1" }
        }"#;
        let p = serde_json::from_str::<MessageParts>(json);
        assert!(p.is_ok(), "{:?}", p.err());
    }

    #[test]
    fn test_keyboard_serde() {
        let json = r#"{ "buttons": [[{ "text": "ok" }]] }"#;
        let k = serde_json::from_str::<Keyboard>(json);
        assert!(k.is_ok(), "{:?}", k.err());
    }

    #[test]
    fn test_buttonkeyboard_serde() {
        let json = r#"{ "text": "ok", "url": "https://example.com" }"#;
        let b = serde_json::from_str::<ButtonKeyboard>(json);
        assert!(b.is_ok(), "{:?}", b.err());
    }

    #[test]
    fn test_apiresponsewrapper_payloadonly() {
        let json = r#"42"#;
        let r = serde_json::from_str::<ApiResponseWrapper<u32>>(json);
        assert!(r.is_ok());
        if let ApiResponseWrapper::PayloadOnly(val) = r.unwrap() {
            assert_eq!(val, 42);
        } else {
            panic!("Not PayloadOnly");
        }
    }

    #[test]
    fn test_apiresponsewrapper_error() {
        let json = r#"{ "ok": false, "description": "fail" }"#;
        let r = serde_json::from_str::<ApiResponseWrapper<u32>>(json);
        assert!(r.is_ok(), "{:?}", r.err());
        if let ApiResponseWrapper::Error { ok, description } = r.unwrap() {
            assert!(!ok);
            assert_eq!(description, "fail");
        } else {
            panic!("Not Error");
        }
    }

    #[test]
    fn test_keyboard_empty_buttons_edge_case() {
        // buttons — двумерный пустой массив невалиден
        let json = r#"{\n            \"buttons\": [[]]\n        }"#;
        let v = serde_json::from_str::<Keyboard>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_keyboard_one_button_valid() {
        // buttons — один ряд с одной кнопкой
        let json = r#"{ "buttons": [[{ "text": "ok" }]] }"#;
        let v = serde_json::from_str::<Keyboard>(json);
        assert!(v.is_ok());
        let k = v.unwrap();
        assert_eq!(k.buttons.len(), 1);
        assert_eq!(k.buttons[0].len(), 1);
        assert_eq!(k.buttons[0][0].text, "ok");
    }

    #[test]
    fn test_buttonkeyboard_optional_fields() {
        let json = r#"{ "text": "ok" }"#;
        let b = serde_json::from_str::<ButtonKeyboard>(json);
        assert!(b.is_ok(), "{:?}", b.err());
        let b = b.unwrap();
        assert_eq!(b.text, "ok");
        assert!(b.url.is_none());
        assert!(b.callback_data.is_none());
        assert!(b.style.is_none());
    }

    #[test]
    fn test_eventpayload_new_message_serde() {
        let json = r#"{
            "msgId": "m1",
            "text": "hi",
            "chat": { "chatId": "c1", "type": "private" },
            "from": { "firstName": "A", "userId": "u1" },
            "timestamp": 123,
            "parts": []
        }"#;
        let v = serde_json::from_str::<EventPayloadNewMessage>(json);
        assert!(v.is_ok(), "{:?}", v.err());
    }

    #[test]
    fn test_eventpayload_edited_message_serde() {
        let json = r#"{
            "msgId": "m2",
            "text": "edit",
            "timestamp": 124,
            "chat": { "chatId": "c2", "type": "private" },
            "from": { "firstName": "B", "userId": "u2" },
            "editedTimestamp": 125
        }"#;
        let v = serde_json::from_str::<EventPayloadEditedMessage>(json);
        assert!(v.is_ok(), "{:?}", v.err());
    }

    #[test]
    fn test_eventpayload_delete_message_serde() {
        let json = r#"{
            "msgId": "m3",
            "chat": { "chatId": "c3", "type": "private" },
            "timestamp": 126
        }"#;
        let v = serde_json::from_str::<EventPayloadDeleteMessage>(json);
        assert!(v.is_ok(), "{:?}", v.err());
    }

    #[test]
    fn test_eventpayload_pinned_message_serde() {
        let json = r#"{
            "msgId": "m4",
            "chat": { "chatId": "c4", "type": "private" },
            "from": { "firstName": "C", "userId": "u3" },
            "text": "pinned",
            "timestamp": 127,
            "parts": []
        }"#;
        let v = serde_json::from_str::<EventPayloadPinnedMessage>(json);
        assert!(v.is_ok(), "{:?}", v.err());
    }

    #[test]
    fn test_eventpayload_unpinned_message_serde() {
        let json = r#"{
            "msgId": "m5",
            "chat": { "chatId": "c5", "type": "private" },
            "timestamp": 128
        }"#;
        let v = serde_json::from_str::<EventPayloadUnpinnedMessage>(json);
        assert!(v.is_ok(), "{:?}", v.err());
    }

    #[test]
    fn test_eventpayload_new_chat_members_serde() {
        let json = r#"{
            "chat": { "chatId": "c6", "type": "private" },
            "newMembers": [{ "firstName": "D", "userId": "u4" }],
            "addedBy": { "firstName": "E", "userId": "u5" }
        }"#;
        let v = serde_json::from_str::<EventPayloadNewChatMembers>(json);
        assert!(v.is_ok(), "{:?}", v.err());
    }

    #[test]
    fn test_eventpayload_left_chat_members_serde() {
        let json = r#"{
            "chat": { "chatId": "c7", "type": "private" },
            "leftMembers": [{ "firstName": "F", "userId": "u6" }],
            "removedBy": { "firstName": "G", "userId": "u7" }
        }"#;
        let v = serde_json::from_str::<EventPayloadLeftChatMembers>(json);
        assert!(v.is_ok(), "{:?}", v.err());
    }

    #[test]
    fn test_eventpayload_callback_query_serde() {
        let json = r#"{
            "queryId": "q1",
            "from": { "firstName": "H", "userId": "u8" },
            "chat": { "chatId": "c8", "type": "private" },
            "message": {
                "msgId": "m6",
                "text": "cb",
                "chat": { "chatId": "c8", "type": "private" },
                "from": { "firstName": "H", "userId": "u8" },
                "timestamp": 129,
                "parts": []
            },
            "callbackData": "cbdata"
        }"#;
        let v = serde_json::from_str::<EventPayloadCallbackQuery>(json);
        assert!(v.is_ok(), "{:?}", v.err());
    }

    #[test]
    fn test_eventpayload_new_message_missing_field() {
        // Нет обязательного поля msgId
        let json = r#"{
            "text": "hi",
            "chat": { "chatId": "c1", "type": "private" },
            "from": { "firstName": "A", "userId": "u1" },
            "timestamp": 123,
            "parts": []
        }"#;
        let v = serde_json::from_str::<EventPayloadNewMessage>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_eventpayload_new_message_wrong_type() {
        // msgId должен быть строкой, а не числом
        let json = r#"{
            "msgId": 123,
            "text": "hi",
            "chat": { "chatId": "c1", "type": "private" },
            "from": { "firstName": "A", "userId": "u1" },
            "timestamp": 123,
            "parts": []
        }"#;
        let v = serde_json::from_str::<EventPayloadNewMessage>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_eventpayload_new_message_empty_parts() {
        // parts пустой массив — валидно
        let json = r#"{
            "msgId": "m1",
            "text": "hi",
            "chat": { "chatId": "c1", "type": "private" },
            "from": { "firstName": "A", "userId": "u1" },
            "timestamp": 123,
            "parts": []
        }"#;
        let v = serde_json::from_str::<EventPayloadNewMessage>(json);
        assert!(v.is_ok());
    }

    #[test]
    fn test_eventpayload_new_chat_members_empty_members() {
        // newMembers пустой массив — валидно
        let json = r#"{
            "chat": { "chatId": "c6", "type": "private" },
            "newMembers": [],
            "addedBy": { "firstName": "E", "userId": "u5" }
        }"#;
        let v = serde_json::from_str::<EventPayloadNewChatMembers>(json);
        assert!(v.is_ok());
    }

    #[test]
    fn test_eventpayload_new_chat_members_missing_members() {
        // Нет поля newMembers
        let json = r#"{
            "chat": { "chatId": "c6", "type": "private" },
            "addedBy": { "firstName": "E", "userId": "u5" }
        }"#;
        let v = serde_json::from_str::<EventPayloadNewChatMembers>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_eventpayload_callback_query_missing_message() {
        // Нет поля message
        let json = r#"{
            "queryId": "q1",
            "from": { "firstName": "H", "userId": "u8" },
            "chat": { "chatId": "c8", "type": "private" },
            "callbackData": "cbdata"
        }"#;
        let v = serde_json::from_str::<EventPayloadCallbackQuery>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_eventpayload_callback_query_wrong_type() {
        // queryId должен быть строкой, а не числом
        let json = r#"{
            "queryId": 1,
            "from": { "firstName": "H", "userId": "u8" },
            "chat": { "chatId": "c8", "type": "private" },
            "message": {
                "msgId": "m6",
                "text": "cb",
                "chat": { "chatId": "c8", "type": "private" },
                "from": { "firstName": "H", "userId": "u8" },
                "timestamp": 129,
                "parts": []
            },
            "callbackData": "cbdata"
        }"#;
        let v = serde_json::from_str::<EventPayloadCallbackQuery>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_eventmessage_missing_field() {
        // Нет обязательного поля msgId
        let json = r#"{
            "text": "hi",
            "chat": { "chatId": "c1", "type": "private" },
            "from": { "firstName": "A", "userId": "u1" },
            "timestamp": 123,
            "parts": []
        }"#;
        let v = serde_json::from_str::<EventMessage>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_eventmessage_wrong_type() {
        // msgId должен быть строкой, а не числом
        let json = r#"{
            "msgId": 1,
            "text": "hi",
            "chat": { "chatId": "c1", "type": "private" },
            "from": { "firstName": "A", "userId": "u1" },
            "timestamp": 123,
            "parts": []
        }"#;
        let v = serde_json::from_str::<EventMessage>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_keyboard_empty_buttons() {
        // buttons пустой массив — валидно
        let json = r#"{
            "buttons": []
        }"#;
        let v = serde_json::from_str::<Keyboard>(json);
        assert!(v.is_ok());
    }

    #[test]
    fn test_keyboard_missing_buttons() {
        // Нет поля buttons
        let json = r#"{}"#;
        let v = serde_json::from_str::<Keyboard>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_buttonkeyboard_missing_text() {
        // Нет обязательного поля text
        let json = r#"{
            "payload": "data"
        }"#;
        let v = serde_json::from_str::<ButtonKeyboard>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_buttonkeyboard_wrong_type() {
        // text должен быть строкой, а не числом
        let json = r#"{
            "text": 123,
            "payload": "data"
        }"#;
        let v = serde_json::from_str::<ButtonKeyboard>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_apiresponsewrapper_missing_response() {
        // Нет обязательного поля response
        let json = r#"{}"#;
        let v = serde_json::from_str::<ApiResponseWrapper<String>>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_apiresponsewrapper_wrong_type() {
        // response должен быть строкой, а не числом
        let json = r#"{ "response": 123 }"#;
        let v = serde_json::from_str::<ApiResponseWrapper<String>>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_messageparts_missing_type() {
        // Нет поля type
        let json = r#"{ "payload": { "fileId": "f1" } }"#;
        let v = serde_json::from_str::<MessageParts>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_messageparts_wrong_type() {
        // type несуществующий
        let json = r#"{ "type": "unknown", "payload": {} }"#;
        let v = serde_json::from_str::<MessageParts>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_messageformat_empty() {
        // Все поля опциональны, пустой объект валиден
        let json = r#"{}"#;
        let v = serde_json::from_str::<MessageFormat>(json);
        assert!(v.is_ok());
    }

    #[test]
    fn test_chat_missing_chat_id() {
        // Нет chatId
        let json = r#"{ "type": "private" }"#;
        let v = serde_json::from_str::<Chat>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_chat_wrong_type() {
        // type не строка
        let json = r#"{ "chatId": "c1", "type": 123 }"#;
        let v = serde_json::from_str::<Chat>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_from_missing_first_name() {
        // Нет firstName
        let json = r#"{ "userId": "u1" }"#;
        let v = serde_json::from_str::<From>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_from_wrong_user_id_type() {
        // userId не строка
        let json = r#"{ "firstName": "A", "userId": 123 }"#;
        let v = serde_json::from_str::<From>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_messagepartspayloadsticker_missing_file_id() {
        // Нет fileId
        let json = r#"{}"#;
        let v = serde_json::from_str::<MessagePartsPayloadSticker>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_messagepartspayloadsticker_wrong_type() {
        // fileId не строка
        let json = r#"{ "fileId": 123 }"#;
        let v = serde_json::from_str::<MessagePartsPayloadSticker>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_messagepartspayloadmention_missing_user_id() {
        // Нет userId
        let json = r#"{}"#;
        let v = serde_json::from_str::<MessagePartsPayloadMention>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_messagepartspayloadvoice_missing_file_id() {
        // Нет fileId
        let json = r#"{}"#;
        let v = serde_json::from_str::<MessagePartsPayloadVoice>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_messagepartspayloadfile_missing_file_id() {
        // Нет fileId
        let json = r#"{}"#;
        let v = serde_json::from_str::<MessagePartsPayloadFile>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_messageformatstruct_missing_offset() {
        // Нет offset
        let json = r#"{ "length": 5 }"#;
        let v = serde_json::from_str::<MessageFormatStruct>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_messageformatstructlink_missing_url() {
        // Нет url
        let json = r#"{ "offset": 1, "length": 2 }"#;
        let v = serde_json::from_str::<MessageFormatStructLink>(json);
        assert!(v.is_err());
    }

    #[test]
    fn test_messageformatstructpre_missing_offset() {
        // Нет offset
        let json = r#"{ "length": 5 }"#;
        let v = serde_json::from_str::<MessageFormatStructPre>(json);
        assert!(v.is_err());
    }
}
