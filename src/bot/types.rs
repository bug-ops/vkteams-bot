use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::fmt::*;
use std::string::String;
use std::sync::{Arc, Mutex};
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
    pub(crate) event_id: Arc<Mutex<u64>>,
}
/// API possible methods
pub enum SendMessagesAPIMethods {
    /// messages/sendText
    MessagesSendText,
    /// messages/sendTextWithDeepLink
    MessagesSendTextWithDeepLink,
    /// messages/editText
    MessagesEditText,
    /// messages/deleteMessages
    MessagesDeleteMessages,
    /// messages/answerCallbackQuery
    MessagesAnswerCallbackQuery,
    /// messages/sendFile
    MessagesSendFile,
    /// messages/sendVoice
    MessagesSendVoice,
    /// chats/avatar/set
    ChatsAvatarSet,
    /// chats/sendActions
    ChatsSendActions,
    /// chats/getInfo
    ChatsGetInfo,
    /// chats/getAdmins
    ChatsGetAdmins,
    /// chats/getMembers
    ChatsGetMembers,
    /// chats/getBlockedUsers
    ChatsGetBlockedUsers,
    /// chats/getPendingUsers
    ChatsGetPendingUsers,
    /// chats/blockUser
    ChatsBlockUser,
    /// chats/unblockUser
    ChatsUnblockUser,
    /// chats/resolvePending
    ChatsResolvePending,
    /// chats/members/delete
    ChatsMembersDelete,
    /// chats/setTitle
    ChatsSetTitle,
    /// chats/setAbout
    ChatsSetAbout,
    /// chats/setRules
    ChatsSetRules,
    /// chats/pinMessage
    ChatsPinMessage,
    /// chats/unpinMessage
    ChatsUnpinMessage,
    /// self/get
    SelfGet,
    /// files/getInfo
    FilesGetInfo,
    /// events/get
    EventsGet,
}
pub type Methods = SendMessagesAPIMethods;
/// Derive `Display` trait for API methods
impl Display for SendMessagesAPIMethods {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            SendMessagesAPIMethods::MessagesSendText => write!(f, "messages/sendText"),
            SendMessagesAPIMethods::MessagesSendTextWithDeepLink => {
                write!(f, "messages/sendTextWithDeepLink")
            }
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
            SendMessagesAPIMethods::ChatsGetBlockedUsers => write!(f, "chats/getBlockedUsers"),
            SendMessagesAPIMethods::ChatsGetPendingUsers => write!(f, "chats/getPendingUsers"),
            SendMessagesAPIMethods::ChatsBlockUser => write!(f, "chats/blockUser"),
            SendMessagesAPIMethods::ChatsUnblockUser => write!(f, "chats/unblockUser"),
            SendMessagesAPIMethods::ChatsResolvePending => write!(f, "chats/resolvePending"),
            SendMessagesAPIMethods::ChatsSetTitle => write!(f, "chats/setTitle"),
            SendMessagesAPIMethods::ChatsSetAbout => write!(f, "chats/setAbout"),
            SendMessagesAPIMethods::ChatsSetRules => write!(f, "chats/setRules"),
            SendMessagesAPIMethods::ChatsPinMessage => write!(f, "chats/pinMessage"),
            SendMessagesAPIMethods::ChatsUnpinMessage => write!(f, "chats/unpinMessage"),
            SendMessagesAPIMethods::SelfGet => write!(f, "self/get"),
            SendMessagesAPIMethods::FilesGetInfo => write!(f, "files/getInfo"),
            SendMessagesAPIMethods::EventsGet => write!(f, "events/get"),
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
#[derive(Serialize, Clone, Debug, Default)]
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
}
/// Response for method [`SendMessagesAPIMethods::MessagesSendText`]
///
/// [`SendMessagesAPIMethods::MessagesSendText`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendText
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMessagesSendText {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<MsgId>, //ok = True
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>, //ok = False
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::MessagesSendTextWithDeepLink`]
///
/// [`SendMessagesAPIMethods::MessagesSendTextWithDeepLink`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendTextWithDeepLink
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesSendTextWithDeepLink {
    pub chat_id: ChatId,
    pub text: String,
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
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMessagesSendTextWithDeepLink {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<MsgId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::MessagesDeleteMessages`]
///
/// [`SendMessagesAPIMethods::MessagesDeleteMessages`]: enum.SendMessagesAPIMethods.html#variant.MessagesDeleteMessages
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesDeleteMessages {
    pub chat_id: ChatId,
    pub msg_id: MsgId,
}
/// Response for method [`SendMessagesAPIMethods::MessagesDeleteMessages`]
///
/// [`SendMessagesAPIMethods::MessagesDeleteMessages`]: enum.SendMessagesAPIMethods.html#variant.MessagesDeleteMessages
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResponseMessagesDeleteMessages {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::MessagesEditText`]
///
/// [`SendMessagesAPIMethods::MessagesEditText`]: enum.SendMessagesAPIMethods.html#variant.MessagesEditText
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesEditText {
    pub chat_id: ChatId,
    pub msg_id: MsgId,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_keyboard_markup: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<MessageFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
}
/// Response for method [`SendMessagesAPIMethods::MessagesEditText`]
///
/// [`SendMessagesAPIMethods::MessagesEditText`]: enum.SendMessagesAPIMethods.html#variant.MessagesEditText
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResponseMessagesEditText {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::EventsGet`]
///
/// [`SendMessagesAPIMethods::EventsGet`]: enum.SendMessagesAPIMethods.html#variant.EventsGet
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestEventsGet {
    pub last_event_id: u64,
    pub poll_time: String,
}
/// Response for method [`SendMessagesAPIMethods::EventsGet`]
///
/// [`SendMessagesAPIMethods::EventsGet`]: enum.SendMessagesAPIMethods.html#variant.EventsGet
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResponseEventsGet {
    pub events: Vec<EventMessage>,
    pub ok: bool,
}
/// Event message
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EventMessage {
    pub event_id: u64,
    #[serde(rename = "type")]
    pub event_type: EventType,
    pub payload: EventPayload,
}
/// Event message payload
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EventPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<MsgId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat: Option<Chat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_members: Option<Vec<From>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<From>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub added_by: Option<From>,
    #[serde(rename = "format", skip_serializing_if = "Option::is_none")]
    pub message_format: Option<MessageFormat>,
    #[serde(rename = "parts", skip_serializing_if = "Option::is_none")]
    pub message_parts: Option<Vec<MessageParts>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_timestamp: Option<u64>,
}
/// Message parts
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageParts {
    #[serde(rename = "type")]
    pub part_type: MessagePartsType,
    pub payload: MessagePartsPayload,
}
/// Message parts payload
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessagePartsPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<UserId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub file_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<MessageFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<MessagePayload>,
}
/// Array of message formats
#[derive(Serialize, Deserialize, Clone, Debug)]
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
    pub link: Option<Vec<MessageFormatStruct>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mention: Option<Vec<MessageFormatStruct>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_code: Option<Vec<MessageFormatStruct>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre: Option<Vec<MessageFormatStruct>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ordered_list: Option<Vec<MessageFormatStruct>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<Vec<MessageFormatStruct>>,
}
/// Message format struct
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageFormatStruct {
    /// offset - required for every format
    pub offset: i32,
    /// length - required for every format
    pub length: i32,
    /// url is only for [`MessageFormat::link`]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// code is only for [`MessageFormat::pre`]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}
/// Event message payload
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessagePayload {
    pub from: From,
    pub msg_id: MsgId,
    pub text: String,
    pub timestamp: u64,
}
/// Chat id struct
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatId(pub String);
/// Display trait for [`ChatId`]
impl Display for ChatId {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.0)
    }
}
/// Message id struct
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MsgId(pub String);
/// User id struct
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserId(pub String);
/// Chat struct
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    pub chat_id: ChatId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub chat_type: String,
}
/// From struct
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct From {
    pub first_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>, //if its a bot, then it will be EMPTY
    pub user_id: UserId,
}
/// Message parts type
#[derive(Serialize, Deserialize, Clone, Debug)]
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
#[derive(Serialize, Deserialize, Clone, Debug)]
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
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Languages {
    Ru,
    En,
}
/// Chat types
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ChatType {
    Private,
    Group,
    Channel,
}
/// Chat actions
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ChatActions {
    Looking,
    Typing,
}
/// Multipart name
pub enum MultipartName {
    File { name: String, file: File },
    Image { name: String, file: File },
    None,
}
impl Display for MultipartName {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            MultipartName::File { .. } => write!(f, "file"),
            MultipartName::Image { .. } => write!(f, "image"),
            _ => write!(f, ""),
        }
    }
}
impl MultipartName {
    pub fn get_file(self) -> (String, File) {
        match self {
            MultipartName::File { name, file } | MultipartName::Image { name, file } => {
                (name, file)
            }
            _ => panic!("No file"),
        }
    }
}
/// Request for method [`SendMessagesAPIMethods::ChatsAvatarSet`]
///
/// [`SendMessagesAPIMethods::ChatsAvatarSet`]: enum.SendMessagesAPIMethods.html#variant.ChatsAvatarSet
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsAvatarSet {
    pub chat_id: ChatId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsAvatarSet`]
///
/// [`SendMessagesAPIMethods::ChatsAvatarSet`]: enum.SendMessagesAPIMethods.html#variant.ChatsAvatarSet
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsAvatarSet {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::ChatsGetAdmins`]
///
/// [`SendMessagesAPIMethods::ChatsGetAdmins`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetAdmins
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetAdmins {
    pub chat_id: ChatId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsGetAdmins`]
///
/// [`SendMessagesAPIMethods::ChatsGetAdmins`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetAdmins
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGetAdmins {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admins: Option<Vec<Admin>>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Admin {
    pub user_id: UserId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<bool>,
}
/// Request for method [`SendMessagesAPIMethods::ChatsGetMembers`]
///
/// [`SendMessagesAPIMethods::ChatsGetMembers`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetMembers
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetMembers {
    pub chat_id: ChatId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<u64>,
}
/// Response for method [`SendMessagesAPIMethods::ChatsGetMembers`]
///
/// [`SendMessagesAPIMethods::ChatsGetMembers`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetMembers
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGetMembers {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<Member>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<u64>,
}
/// Request for method [`SendMessagesAPIMethods::ChatsGetBlockedUsers`]
///
/// [`SendMessagesAPIMethods::ChatsGetBlockedUsers`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetBlockedUsers
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetBlockedUsers {
    pub chat_id: ChatId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsGetBlockedUsers`]
///
/// [`SendMessagesAPIMethods::ChatsGetBlockedUsers`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetBlockedUsers
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGetBlockedUsers {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<BlockedUser>>,
}
/// Blocked user struct
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockedUser {
    pub user_id: UserId,
}
/// Request for method [`SendMessagesAPIMethods::ChatsBlockUser`]
///
/// [`SendMessagesAPIMethods::ChatsBlockUser`]: enum.SendMessagesAPIMethods.html#variant.ChatsBlockUser
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetPendingUsers {
    pub chat_id: ChatId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsBlockUser`]
///
/// [`SendMessagesAPIMethods::ChatsBlockUser`]: enum.SendMessagesAPIMethods.html#variant.ChatsBlockUser
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGetPendingUsers {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<PendingUser>>,
}
///Pending user struct
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PendingUser {
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
/// Request for method [`SendMessagesAPIMethods::ChatsBlockUser`]
///
/// [`SendMessagesAPIMethods::ChatsBlockUser`]: enum.SendMessagesAPIMethods.html#variant.ChatsBlockUser
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsBlockUser {
    pub chat_id: ChatId,
    pub user_id: UserId,
    pub del_last_messages: bool,
}
/// Response for method [`SendMessagesAPIMethods::ChatsBlockUser`]
///
/// [`SendMessagesAPIMethods::ChatsBlockUser`]: enum.SendMessagesAPIMethods.html#variant.ChatsBlockUser
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsBlockUser {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::ChatsUnblockUser`]
///
/// [`SendMessagesAPIMethods::ChatsResolvePending`]: enum.SendMessagesAPIMethods.html#variant.ChatsUnblockUser
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsUnblockUser {
    pub chat_id: ChatId,
    pub user_id: UserId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsUnblockUser`]
///
/// [`SendMessagesAPIMethods::ChatsResolvePending`]: enum.SendMessagesAPIMethods.html#variant.ChatsUnblockUser
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsUnblockUser {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::ChatsResolvePending`]
///
/// [`SendMessagesAPIMethods::ChatsResolvePending`]: enum.SendMessagesAPIMethods.html#variant.ChatsResolvePending
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsResolvePending {
    pub chat_id: ChatId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<UserId>,
    pub approve: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub everyone: Option<bool>,
}
/// Response for method [`SendMessagesAPIMethods::ChatsResolvePending`]
///
/// [`SendMessagesAPIMethods::ChatsResolvePending`]: enum.SendMessagesAPIMethods.html#variant.ChatsResolvePending
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsResolvePending {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::ChatsMembersDelete`]
///
/// [`SendMessagesAPIMethods::ChatsMembersDelete`]: enum.SendMessagesAPIMethods.html#variant.ChatsMembersDelete
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsMembersDelete {
    pub chat_id: ChatId,
    pub user_id: UserId,
    pub members: Vec<Sn>,
}
/// Sn struct for members
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Sn {
    pub sn: String,
    pub user_id: UserId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsMembersDelete`]
///
/// [`SendMessagesAPIMethods::ChatsMembersDelete`]: enum.SendMessagesAPIMethods.html#variant.ChatsMembersDelete
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResponseChatsMembersDelete {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::ChatsSetTitle`]
///
/// [`SendMessagesAPIMethods::ChatsSetTitle`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetTitle
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsSetTitle {
    pub chat_id: ChatId,
    pub title: String,
}
/// Response for method [`SendMessagesAPIMethods::ChatsSetTitle`]
///
/// [`SendMessagesAPIMethods::ChatsSetTitle`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetTitle
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsSetTitle {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::ChatsSetAbout`]
///
/// [`SendMessagesAPIMethods::ChatsSetAbout`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetAbout
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsSetAbout {
    pub chat_id: ChatId,
    pub about: String,
}
/// Response for method [`SendMessagesAPIMethods::ChatsSetAbout`]
///
/// [`SendMessagesAPIMethods::ChatsSetAbout`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetAbout
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsSetAbout {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::ChatsSetRules`]
///
/// [`SendMessagesAPIMethods::ChatsSetRules`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetRules
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsSetRules {
    pub chat_id: ChatId,
    pub rules: String,
}
/// Response for method [`SendMessagesAPIMethods::ChatsSetRules`]
///
/// [`SendMessagesAPIMethods::ChatsSetRules`]: enum.SendMessagesAPIMethods.html#variant.ChatsSetRules
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsSetRules {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::ChatsPinMessage`]
///
/// [`SendMessagesAPIMethods::ChatsPinMessage`]: enum.SendMessagesAPIMethods.html#variant.ChatsPinMessage
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsPinMessage {
    pub chat_id: ChatId,
    pub msg_id: MsgId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsPinMessage`]
///
/// [`SendMessagesAPIMethods::ChatsPinMessage`]: enum.SendMessagesAPIMethods.html#variant.ChatsPinMessage
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsPinMessage {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::ChatsUnpinMessage`]
///
/// [`SendMessagesAPIMethods::ChatsUnpinMessage`]: enum.SendMessagesAPIMethods.html#variant.ChatsUnpinMessage
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsUnpinMessage {
    pub chat_id: ChatId,
    pub msg_id: MsgId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsUnpinMessage`]
///
/// [`SendMessagesAPIMethods::ChatsUnpinMessage`]: enum.SendMessagesAPIMethods.html#variant.ChatsUnpinMessage
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsUnpinMessage {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::SelfGet`]
///
/// [`SendMessagesAPIMethods::SelfGet`]: enum.SendMessagesAPIMethods.html#variant.SelfGet
#[derive(Serialize, Clone, Debug)]
pub struct RequestSelfGet {}
/// Response for method [`SendMessagesAPIMethods::SelfGet`]
///
/// [`SendMessagesAPIMethods::SelfGet`]: enum.SendMessagesAPIMethods.html#variant.SelfGet
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseSelfGet {
    pub user_id: UserId,
    pub nick: String,
    pub first_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo: Option<Vec<PhotoUrl>>,
    pub ok: bool,
}
/// Photo url struct
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PhotoUrl {
    pub url: String,
}
/// Request for method [`SendMessagesAPIMethods::MessagesAnswerCallbackQuery`]
///
/// [`SendMessagesAPIMethods::MessagesAnswerCallbackQuery`]: enum.SendMessagesAPIMethods.html#variant.MessagesAnswerCallbackQuery
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesAnswerCallbackQuery {
    pub query_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_alert: Option<ShowAlert>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShowAlert(pub bool);
/// Response for method [`SendMessagesAPIMethods::MessagesAnswerCallbackQuery`]
///
/// [`SendMessagesAPIMethods::MessagesAnswerCallbackQuery`]: enum.SendMessagesAPIMethods.html#variant.MessagesAnswerCallbackQuery
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResponseMessagesAnswerCallbackQuery {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::MessagesSendFile`]
///
/// [`SendMessagesAPIMethods::MessagesSendFile`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendFile
#[derive(Serialize, Deserialize, Clone, Debug)]
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
}
/// Response for method [`SendMessagesAPIMethods::MessagesSendFile`]
///
/// [`SendMessagesAPIMethods::MessagesSendFile`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendFile
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMessagesSendFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<MsgId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    pub ok: bool,
}
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
/// Request for method [`SendMessagesAPIMethods::MessagesSendVoice`]
///
/// [`SendMessagesAPIMethods::MessagesSendVoice`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendVoice
pub struct RequestMessagesSendVoice {
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
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
/// Response for method [`SendMessagesAPIMethods::MessagesSendVoice`]
///
/// [`SendMessagesAPIMethods::MessagesSendVoice`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendVoice
pub struct ResponseMessagesSendVoice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<MsgId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::ChatsSendActions`]
///
/// [`SendMessagesAPIMethods::ChatsSendActions`]: enum.SendMessagesAPIMethods.html#variant.ChatsSendActions
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsSendAction {
    pub chat_id: ChatId,
    pub actions: ChatActions,
}
/// Response for method [`SendMessagesAPIMethods::ChatsSendActions`]
///
/// [`SendMessagesAPIMethods::ChatsSendActions`]: enum.SendMessagesAPIMethods.html#variant.ChatsSendActions
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResponseChatsSendAction {
    pub ok: bool,
}
/// Request for method [`SendMessagesAPIMethods::ChatsGetInfo`]
///
/// [`SendMessagesAPIMethods::ChatsGetInfo`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetInfo
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestChatsGetInfo {
    pub chat_id: ChatId,
}
/// Response for method [`SendMessagesAPIMethods::ChatsGetInfo`]
///
/// [`SendMessagesAPIMethods::ChatsGetInfo`]: enum.SendMessagesAPIMethods.html#variant.ChatsGetInfo
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGetInfo {
    #[serde(rename = "type")]
    pub chat_type: ChatType,
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nick: Option<String>,
    pub about: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_bot: Option<bool>,
    pub language: Languages,
    // FIXME: Separate this struct for different chat types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invite_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join_moderation: Option<bool>,
}
/// Request for method [`SendMessagesAPIMethods::FilesGetInfo`]
///
/// [`SendMessagesAPIMethods::FilesGetInfo`]: enum.SendMessagesAPIMethods.html#variant.FilesGetInfo
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestFilesGetInfo {
    pub file_id: String,
}
/// Response for method [`SendMessagesAPIMethods::FilesGetInfo`]
///
/// [`SendMessagesAPIMethods::FilesGetInfo`]: enum.SendMessagesAPIMethods.html#variant.FilesGetInfo
#[derive(Serialize, Deserialize, Clone, Debug)]
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
