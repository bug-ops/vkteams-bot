//! API types
use std::fmt::*;
use std::time::Duration;
#[cfg(feature = "templates")]
use tera::Context;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
#[cfg(feature = "templates")]
use tera::Tera;

/// Environment variable name for bot API URL
pub const VKTEAMS_BOT_API_URL: &str = "VKTEAMS_BOT_API_URL";
/// Environment variable name for bot API token
pub const VKTEAMS_BOT_API_TOKEN: &str = "VKTEAMS_BOT_API_TOKEN";
/// Environment variable name for bot Proxy URL
pub const VKTEAMS_PROXY: &str = "VKTEAMS_PROXY";
/// Timeout for long polling
pub const POLL_TIME: u64 = 30;
/// Global timeout for [`reqwest::Client`]
///
/// [`reqwest::Client`]: https://docs.rs/reqwest/latest/reqwest/struct.Client.html
pub const POLL_DURATION: &Duration = &Duration::from_secs(POLL_TIME + 10);
/// Supported API versions
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
    const METHOD: &'static str;
    const HTTP_METHOD: HTTPMethod = HTTPMethod::GET;
    type RequestType: Serialize + Debug + Default;
    type ResponseType: Serialize + DeserializeOwned + Debug + Default;
    fn get_file(&self) -> MultipartName {
        MultipartName::None
    }
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
    pub(crate) text: Vec<MessageTextFormat>,
    // Context for templates
    #[cfg(feature = "templates")]
    pub(crate) ctx: Context,
    // Template name
    #[cfg(feature = "templates")]
    pub(crate) name: String,
    // Tera template engine
    #[cfg(feature = "templates")]
    pub(crate) tmpl: Tera,
    /// ## Parse mode
    /// - `HTML` - HTML
    /// - `MarkdownV2` - Markdown
    pub(crate) parse_mode: ParseMode,
}
/// Keyboard for send message methods
/// One of variants must be set:
/// - {`text`: String,`url`: String,`style`: [`ButtonStyle`]} - simple buttons
/// - {`text`: String,`callback_data`: String,`style`: [`ButtonStyle`]} - buttons with callback
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ButtonKeyboard {
    pub(crate) text: String, // formatting is not supported
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) callback_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) style: Option<ButtonStyle>,
}
#[derive(Serialize, Clone, Debug)]
/// Array of keyboard buttons
pub struct Keyboard {
    pub(crate) buttons: Vec<Vec<ButtonKeyboard>>,
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
    #[serde(rename = "type", flatten)]
    pub event_type: EventType,
    // pub payload: EventPayload,
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
    #[serde(default)]
    pub format: MessageFormat,
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
    pub format: MessageFormat,
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
    #[serde(default)]
    pub format: MessageFormat,
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
    #[serde(rename = "type")]
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
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct ChatId(pub String);
/// Message id struct
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct MsgId(pub String);
/// User id struct
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct UserId(pub String);
/// File id struct
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct FileId(pub String);
/// Query id struct
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct QueryId(pub String);
/// Timestamp struct
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum ChatActions {
    Looking,
    #[default]
    Typing,
}
/// Multipart name
#[derive(Debug, Default, Clone)]
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
