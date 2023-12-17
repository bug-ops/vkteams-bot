use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::fmt::*;
use std::string::String;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File;
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
/// [`reqwest::Client`]: https://docs.rs/reqwest/0.11.4/reqwest/struct.Client.html
pub const POLL_DURATION: Duration = Duration::from_secs(POLL_TIME + 1);
/// Supported API versions
pub enum APIVersionUrl {
    /// default V1
    V1,
}
/// Link basse path for API version
impl Display for APIVersionUrl {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            APIVersionUrl::V1 => write!(f, "bot/v1/"),
        }
    }
}
/// Bot class with attributes
/// - `client`: [`reqwest::Client`]
/// - `token`: String
/// - `base_api_url`: [`reqwest::Url`]
/// - `base_api_path`: String
/// - `evtent_id`: [`std::sync::Arc<_>`]
///
/// [`reqwest::Client`]: https://docs.rs/reqwest/0.11.4/reqwest/struct.Client.html
/// [`reqwest::Url`]: https://docs.rs/reqwest/0.11.4/reqwest/struct.Url.html
/// [`std::sync::Arc<_>`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
pub struct Bot {
    pub(crate) client: Client,
    pub(crate) token: String,
    pub(crate) base_api_url: Url,
    pub(crate) base_api_path: String,
    pub(crate) event_id: Arc<u64>,
}
/// API possible methods
pub enum SendMessagesAPIMethods {
    MessagesSendText,
    MessagesEditText,
    MessagesDeleteMessages,
    MessagesAnswerCallbackQuery,
    MessagesSendFile,
    MessagesSendVoice,
    ChatsAvatarSet,
    ChatsSendActions,
    ChatsGetInfo,
    ChatsGetAdmins,
    ChatsGetMembers,
    ChatsMembersDelete,
    SelfGet,
    FilesGetInfo,
    EventsGet,
}
pub type Methods = SendMessagesAPIMethods;
/// Derive `Display` trait for API methods
impl Display for SendMessagesAPIMethods {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            SendMessagesAPIMethods::MessagesSendText => write!(f, "messages/sendText"),
            SendMessagesAPIMethods::MessagesEditText => write!(f, "messages/editText"),
            SendMessagesAPIMethods::MessagesDeleteMessages => write!(f, "messages/deleteMessages"),
            SendMessagesAPIMethods::MessagesAnswerCallbackQuery => {
                write!(f, "messages/answerCallbackQuery")
            }
            SendMessagesAPIMethods::MessagesSendFile => write!(f, "messages/sendFile"),
            SendMessagesAPIMethods::MessagesSendVoice => write!(f, "messages/sendVoice"),
            SendMessagesAPIMethods::ChatsAvatarSet => write!(f, "chats/avatar/set"),
            SendMessagesAPIMethods::ChatsSendActions => write!(f, "chats/sendActions"),
            SendMessagesAPIMethods::ChatsGetInfo => write!(f, "chats/getInfo"),
            SendMessagesAPIMethods::ChatsGetAdmins => write!(f, "chats/getAdmins"),
            SendMessagesAPIMethods::ChatsGetMembers => write!(f, "chats/getMembers"),
            SendMessagesAPIMethods::ChatsMembersDelete => write!(f, "chats/members/delete"),
            SendMessagesAPIMethods::SelfGet => write!(f, "self/get"),
            SendMessagesAPIMethods::FilesGetInfo => write!(f, "files/getInfo"),
            SendMessagesAPIMethods::EventsGet => write!(f, "events/get"),
            // _ => panic!("Unknown API method"),
        }
    }
}
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
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesSendText {
    pub chat_id: ChatId,
    pub text: String,
    pub reply_msg_id: Option<MsgId>,
    pub forward_chat_id: Option<ChatId>,
    pub forward_msg_id: Option<MsgId>,
    pub inline_keyboard_markup: Option<String>,
    pub format: Option<MessageFormat>,
    pub parse_mode: Option<ParseMode>,
}
/// Message text struct
#[derive(Serialize, Clone, Debug)]
pub enum MessageTextFormat {
    Plain(String),                // Plain text
    Bold(String),                 // Bold text
    Italic(String),               // Italic text
    Underline(String),            // Underline text
    Strikethrough(String),        // Strikethrough text
    Link(String, String),         // inline URL, text
    Mention(ChatId),              // inline mention of a user
    Code(String),                 // inline fixed-width code
    Pre(String, Option<String>),  //pre-formatted fixed-width code block
    OrderedList(Vec<String>),     // ordered (numbered) list
    UnOrdereredList(Vec<String>), // unordered (bulleted) list
    Quote(String),                // quote text
    None,
}

/// Message text parse struct
#[derive(Serialize, Clone, Debug)]
pub struct MessageTextParser {
    pub text: Vec<MessageTextFormat>,
    pub parse_mode: ParseMode,
}
/// Keyboard for method [`SendMessagesAPIMethods::MessagesSendText`]
/// One of variants must be set:
/// - {`text`: String,`url`: String,`style`: [`KeyboardStyle`]} - simple buttons
/// - {`text`: String,`callback_data`: String,`style`: [`KeyboardStyle`]} - buttons with callback
///
/// [`SendMessagesAPIMethods::MessagesSendText`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendText
#[derive(Serialize, Clone, Debug)]
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
#[derive(Serialize, Clone, Debug)]
/// Array of keyboard buttons
pub struct Keyboard {
    pub buttons: Vec<Vec<ButtonKeyboard>>,
}
/// Keyboard buttons style
#[derive(Serialize, Clone, Copy, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum ButtonStyle {
    Primary,
    Attention,
    #[default]
    Base,
}
/// Message text format parse mode
#[derive(Serialize, Deserialize, Clone, Debug, Copy, Default)]
pub enum ParseMode {
    MarkdownV2,
    #[default]
    HTML,
}
/// Response for method [`SendMessagesAPIMethods::MessagesSendText`]
///
/// [`SendMessagesAPIMethods::MessagesSendText`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendText
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMessagesSendText {
    pub msg_id: Option<MsgId>,       //ok = True
    pub description: Option<String>, //ok = False
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::MessagesDeleteMessages`]
///
/// [`SendMessagesAPIMethods::MessagesDeleteMessages`]: enum.SendMessagesAPIMethods.html#variant.MessagesDeleteMessages
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesDeleteMessages {
    pub chat_id: ChatId,
    pub msg_id: MsgId,
}
/// Response for method [`SendMessagesAPIMethods::MessagesDeleteMessages`]
///
/// [`SendMessagesAPIMethods::MessagesDeleteMessages`]: enum.SendMessagesAPIMethods.html#variant.MessagesDeleteMessages
#[derive(Deserialize, Debug)]
pub struct ResponseMessagesDeleteMessages {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::MessagesEditText`]
///
/// [`SendMessagesAPIMethods::MessagesEditText`]: enum.SendMessagesAPIMethods.html#variant.MessagesEditText
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesEditText {
    pub chat_id: ChatId,
    pub msg_id: MsgId,
    pub text: String,
    pub inline_keyboard_markup: Option<String>,
    pub format: Option<MessageFormat>,
    pub parse_mode: Option<ParseMode>,
}
/// Response for method [`SendMessagesAPIMethods::MessagesEditText`]
///
/// [`SendMessagesAPIMethods::MessagesEditText`]: enum.SendMessagesAPIMethods.html#variant.MessagesEditText
#[derive(Deserialize)]
pub struct ResponseMessagesEditText {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::EventsGet`]
///
/// [`SendMessagesAPIMethods::EventsGet`]: enum.SendMessagesAPIMethods.html#variant.EventsGet
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestEventsGet {
    pub last_event_id: String,
    pub poll_time: String,
}
/// Response for method [`SendMessagesAPIMethods::EventsGet`]
///
/// [`SendMessagesAPIMethods::EventsGet`]: enum.SendMessagesAPIMethods.html#variant.EventsGet
#[derive(Deserialize, Debug)]
pub struct ResponseEventsGet {
    pub events: Vec<EventMessage>,
    pub ok: bool,
}
/// Event message
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EventMessage {
    pub event_id: u64,
    #[serde(rename = "type")]
    pub event_type: EventType,
    pub payload: EventPayload,
}
/// Event message payload
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EventPayload {
    pub msg_id: Option<MsgId>,
    pub query_id: Option<String>,
    pub text: Option<String>,
    pub timestamp: Option<u64>,
    pub chat: Option<Chat>,
    pub new_members: Option<Vec<From>>,
    pub from: Option<From>,
    pub added_by: Option<From>,
    #[serde(rename = "format")]
    pub message_format: Option<MessageFormat>,
    #[serde(rename = "parts")]
    pub message_parts: Option<Vec<MessageParts>>,
    pub edited_timestamp: Option<u64>,
}
/// Message parts
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageParts {
    #[serde(rename = "type")]
    pub part_type: MessagePartsType,
    pub payload: MessagePartsPayload,
}
/// Message parts payload
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessagePartsPayload {
    pub file_id: Option<String>,
    pub user_id: Option<UserId>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[serde(rename = "type")]
    pub file_type: Option<String>,
    pub caption: Option<String>,
    pub format: Option<MessageFormat>,
    pub message: Option<MessagePayload>,
}
/// Array of message formats
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessageFormat {
    pub bold: Option<Vec<MessageFormatStruct>>,
    pub italic: Option<Vec<MessageFormatStruct>>,
    pub underline: Option<Vec<MessageFormatStruct>>,
    pub strikethrough: Option<Vec<MessageFormatStruct>>,
    pub link: Option<Vec<MessageFormatStruct>>,
    pub mention: Option<Vec<MessageFormatStruct>>,
    pub inline_code: Option<Vec<MessageFormatStruct>>,
    pub pre: Option<Vec<MessageFormatStruct>>,
    pub ordered_list: Option<Vec<MessageFormatStruct>>,
    pub quote: Option<Vec<MessageFormatStruct>>,
}
/// Message format struct
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessageFormatStruct {
    /// offset - required for every format
    pub offset: i32,
    /// length - required for every format
    pub length: i32,
    /// url is only for [`MessageFormat::link`]
    pub url: Option<String>,
    /// code is only for [`MessageFormat::pre`]
    pub code: Option<String>,
}
/// Event message payload
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessagePayload {
    pub from: From,
    pub msg_id: MsgId,
    pub text: String,
    pub timestamp: u64,
}
/// Chat id struct
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatId(pub String);
/// Display trait for [`ChatId`]
impl Display for ChatId {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.0)
    }
}
/// Message id struct
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MsgId(pub String);
/// User id struct
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserId(pub String);
/// Chat struct
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    pub chat_id: ChatId,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub chat_type: String,
}
/// From struct
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct From {
    pub first_name: String,
    pub last_name: Option<String>, //if its a bot, then it will be EMPTY
    pub user_id: UserId,
}
/// Message parts type
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum MessagePartsType {
    Sticker,
    Mention,
    Voice,
    File,
    Forward,
    Reply,
}
/// Event types
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum EventType {
    NewMessage,
    EditedMessage,
    DeleteMessage,
    PinnedMessage,
    UnpinnedMessage,
    NewChatMembers,
    LeftChatMembers,
    CallbackQuery,
}
/// Languages
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Languages {
    Ru,
    En,
}
/// Chat types
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ChatType {
    Private,
    Group,
    Channel,
}
/// Chat actions
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChatActions {
    Looking,
    Typing,
}
/// Multipart name
pub enum MultipartName {
    File(File),
    Image(File),
    None,
}
impl Display for MultipartName {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            MultipartName::File(_) => write!(f, "file"),
            MultipartName::Image(_) => write!(f, "image"),
            _ => write!(f, ""),
        }
    }
}
impl MultipartName {
    pub fn get_file(self) -> File {
        match self {
            MultipartName::File(file) | MultipartName::Image(file) => file,
            _ => panic!("No file"),
        }
    }
}
/// Request for method [`SendMessagesAPIMethods::ChatsAvatarSet`]
///
/// [`SendMessagesAPIMethods::ChatsAvatarSet`]: enum.SendMessagesAPIMethods.html#variant.ChatsAvatarSet
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsAvatarSet {
    pub chat_id: ChatId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsAvatarSet`]
///
/// [`SendMessagesAPIMethods::ChatsAvatarSet`]: enum.SendMessagesAPIMethods.html#variant.ChatsAvatarSet
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsAvatarSet {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::ChatsGetAdmins`]
///
/// [`SendMessagesAPIMethods::ChatsGetAdmins`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetAdmins
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetAdmins {
    pub chat_id: ChatId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsGetAdmins`]
///
/// [`SendMessagesAPIMethods::ChatsGetAdmins`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetAdmins
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGetAdmins {
    pub admins: Option<Vec<Admin>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Admin {
    pub user_id: UserId,
    pub creator: Option<bool>,
}
/// Request for method [`SendMessagesAPIMethods::ChatsGetMembers`]
///
/// [`SendMessagesAPIMethods::ChatsGetMembers`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetMembers
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetMembers {
    pub chat_id: ChatId,
    pub cursor: Option<u64>,
}
/// Response for method [`SendMessagesAPIMethods::ChatsGetMembers`]
///
/// [`SendMessagesAPIMethods::ChatsGetMembers`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetMembers
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGetMembers {
    pub members: Option<Vec<Member>>,
    pub cursor: Option<u64>,
}
/// Member struct
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub user_id: UserId,
    pub creator: Option<bool>,
    pub admin: Option<bool>,
}
/// Request for method [`SendMessagesAPIMethods::ChatsMembersDelete`]
///
/// [`SendMessagesAPIMethods::ChatsMembersDelete`]: enum.SendMessagesAPIMethods.html#variant.ChatsMembersDelete
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsMembersDelete {
    pub chat_id: ChatId,
    pub user_id: UserId,
    pub members: Vec<Sn>,
}
/// Sn struct for members
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Sn {
    pub sn: String,
    pub user_id: UserId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsMembersDelete`]
///
/// [`SendMessagesAPIMethods::ChatsMembersDelete`]: enum.SendMessagesAPIMethods.html#variant.ChatsMembersDelete
#[derive(Deserialize, Debug)]
pub struct ResponseChatsMembersDelete {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::SelfGet`]
///
/// [`SendMessagesAPIMethods::SelfGet`]: enum.SendMessagesAPIMethods.html#variant.SelfGet
#[derive(Serialize, Deserialize)]
pub struct RequestSelfGet {}
/// Response for method [`SendMessagesAPIMethods::SelfGet`]
///
/// [`SendMessagesAPIMethods::SelfGet`]: enum.SendMessagesAPIMethods.html#variant.SelfGet
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseSelfGet {
    pub user_id: UserId,
    pub nick: String,
    pub first_name: String,
    pub about: Option<String>,
    pub photo: Option<Vec<PhotoUrl>>,
    pub ok: bool,
}
/// Photo url struct
#[derive(Serialize, Deserialize)]
pub struct PhotoUrl {
    pub url: String,
}
/// Request for method [`SendMessagesAPIMethods::MessagesAnswerCallbackQuery`]
///
/// [`SendMessagesAPIMethods::MessagesAnswerCallbackQuery`]: enum.SendMessagesAPIMethods.html#variant.MessagesAnswerCallbackQuery
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesAnswerCallbackQuery {
    pub query_id: String,
    pub text: Option<String>,
    pub show_alert: Option<ShowAlert>,
    pub url: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ShowAlert(pub bool);
/// Response for method [`SendMessagesAPIMethods::MessagesAnswerCallbackQuery`]
///
/// [`SendMessagesAPIMethods::MessagesAnswerCallbackQuery`]: enum.SendMessagesAPIMethods.html#variant.MessagesAnswerCallbackQuery
#[derive(Deserialize)]
pub struct ResponseMessagesAnswerCallbackQuery {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::MessagesSendFile`]
///
/// [`SendMessagesAPIMethods::MessagesSendFile`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendFile
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesSendFile {
    pub chat_id: ChatId,
    pub caption: Option<String>,
    pub reply_msg_id: Option<MsgId>,
    pub forward_chat_id: Option<ChatId>,
    pub forward_msg_id: Option<MsgId>,
    pub inline_keyboard_markup: Option<String>,
    pub format: Option<MessageFormat>,
    pub parse_mode: Option<ParseMode>,
}
/// Response for method [`SendMessagesAPIMethods::MessagesSendFile`]
///
/// [`SendMessagesAPIMethods::MessagesSendFile`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendFile
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMessagesSendFile {
    pub msg_id: Option<MsgId>,
    pub file_id: Option<String>,
    pub ok: bool,
}
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
/// Request for method [`SendMessagesAPIMethods::MessagesSendVoice`]
///
/// [`SendMessagesAPIMethods::MessagesSendVoice`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendVoice
pub struct RequestMessagesSendVoice {
    pub chat_id: ChatId,
    pub caption: Option<String>,
    pub reply_msg_id: Option<MsgId>,
    pub forward_chat_id: Option<ChatId>,
    pub forward_msg_id: Option<MsgId>,
    pub inline_keyboard_markup: Option<String>,
    pub format: Option<MessageFormat>,
    pub parse_mode: Option<ParseMode>,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
/// Response for method [`SendMessagesAPIMethods::MessagesSendVoice`]
///
/// [`SendMessagesAPIMethods::MessagesSendVoice`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendVoice
pub struct ResponseMessagesSendVoice {
    pub msg_id: Option<MsgId>,
    pub file_id: Option<String>,
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::ChatsSendActions`]
///
/// [`SendMessagesAPIMethods::ChatsSendActions`]: enum.SendMessagesAPIMethods.html#variant.ChatsSendActions
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsSendAction {
    pub chat_id: ChatId,
    pub actions: ChatActions,
}
/// Response for method [`SendMessagesAPIMethods::ChatsSendActions`]
///
/// [`SendMessagesAPIMethods::ChatsSendActions`]: enum.SendMessagesAPIMethods.html#variant.ChatsSendActions
#[derive(Deserialize)]
pub struct ResponseChatsSendAction {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::ChatsGetInfo`]
///
/// [`SendMessagesAPIMethods::ChatsGetInfo`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetInfo
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetInfo {
    pub chat_id: ChatId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsGetInfo`]
///
/// [`SendMessagesAPIMethods::ChatsGetInfo`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetInfo
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGetInfo {
    #[serde(rename = "type")]
    pub chat_type: ChatType,
    pub first_name: String,
    pub last_name: String,
    pub nick: Option<String>,
    pub about: String,
    pub is_bot: Option<bool>,
    pub language: Languages,
    // FIXME: Separate this struct for different chat types
    pub title: Option<String>,
    pub rules: Option<String>,
    pub invite_link: Option<String>,
    pub public: Option<bool>,
    pub join_moderation: Option<bool>,
}
/// Request for method [`SendMessagesAPIMethods::FilesGetInfo`]
///
/// [`SendMessagesAPIMethods::FilesGetInfo`]: enum.SendMessagesAPIMethods.html#variant.FilesGetInfo
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestFilesGetInfo {
    pub file_id: String,
}
/// Response for method [`SendMessagesAPIMethods::FilesGetInfo`]
///
/// [`SendMessagesAPIMethods::FilesGetInfo`]: enum.SendMessagesAPIMethods.html#variant.FilesGetInfo
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseFilesGetInfo {
    #[serde(rename = "type")]
    pub file_type: String,
    #[serde(rename = "size")]
    pub file_size: u64,
    #[serde(rename = "filename")]
    pub file_name: String,
    pub url: String,
}
