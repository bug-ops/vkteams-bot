pub use crate::api::chats::{
    avatar_set::*, block_user::*, get_admins::*, get_blocked_users::*, get_info::*, get_members::*,
    get_pending_users::*, members_delete::*, pin_message::*, resolve_pendings::*, send_action::*,
    set_about::*, set_rules::*, set_title::*, unblock_user::*, unpin_message::*,
};
pub use crate::api::events::get::*;
pub use crate::api::files::get_info::*;
pub use crate::api::messages::{
    answer_callback_query::*, delete_messages::*, edit_text::*, send_file::*, send_text::*,
    send_text_with_deep_link::*, send_voice::*,
};
pub use crate::api::myself::get::*;
pub use crate::api::{net::*, utils::*};
pub use crate::bot::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::*;
use std::time::Duration;

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
pub const POLL_DURATION: &Duration = &Duration::from_secs(POLL_TIME + 1);
/// Supported API versions
pub enum APIVersionUrl {
    /// default V1
    V1,
}
/// API possible methods with required parameters
#[derive(Debug, Default)]
pub enum SendMessagesAPIMethods {
    /// messages/sendText
    MessagesSendText(ChatId),
    /// messages/sendTextWithDeepLink
    MessagesSendTextWithDeepLink(ChatId, String),
    /// messages/editText
    MessagesEditText(ChatId, MsgId),
    /// messages/deleteMessages
    MessagesDeleteMessages(ChatId, MsgId),
    /// messages/answerCallbackQuery
    MessagesAnswerCallbackQuery(QueryId, Option<String>, Option<ShowAlert>, Option<String>),
    /// messages/sendFile
    MessagesSendFile(ChatId, MultipartName),
    /// messages/sendVoice
    MessagesSendVoice(ChatId, MultipartName),
    /// chats/avatar/set
    ChatsAvatarSet(ChatId, MultipartName),
    /// chats/sendActions
    ChatsSendAction(ChatId, ChatActions),
    /// chats/getInfo
    ChatsGetInfo(ChatId),
    /// chats/getAdmins
    ChatsGetAdmins(ChatId),
    /// chats/getMembers
    ChatsGetMembers(ChatId),
    /// chats/getBlockedUsers
    ChatsGetBlockedUsers(ChatId),
    /// chats/getPendingUsers
    ChatsGetPendingUsers(ChatId),
    /// chats/blockUser
    ChatsBlockUser(ChatId, UserId, bool),
    /// chats/unblockUser
    ChatsUnblockUser(ChatId, UserId),
    /// chats/resolvePending
    ChatsResolvePending(ChatId, bool, Option<UserId>, Option<bool>),
    /// chats/members/delete
    ChatsMembersDelete(ChatId, UserId),
    /// chats/setTitle
    ChatsSetTitle(ChatId, String),
    /// chats/setAbout
    ChatsSetAbout(ChatId, String),
    /// chats/setRules
    ChatsSetRules(ChatId, String),
    /// chats/pinMessage
    ChatsPinMessage(ChatId, MsgId),
    /// chats/unpinMessage
    ChatsUnpinMessage(ChatId, MsgId),
    /// self/get
    SelfGet(),
    /// files/getInfo
    FilesGetInfo(FileId),
    /// events/get
    EventsGet(u64),
    #[default]
    None,
}
/// Supported API HTTP methods
#[derive(Debug, Default)]
pub enum HTTPMethod {
    #[default]
    GET,
    POST,
}
/// Bot request trait
pub trait BotRequest {
    const METHOD: &'static str;
    const HTTP_METHOD: HTTPMethod = HTTPMethod::GET;
    type RequestType: Serialize + Debug + Default;
    type ResponseType: Serialize + DeserializeOwned + Debug + Default;
    fn new(method: &Methods) -> Self;
    fn get_file(&self) -> Option<MultipartName> {
        None
    }
}

pub type Methods = SendMessagesAPIMethods;
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
    UnOrdereredList(Vec<String>),
    /// Quote text
    Quote(String),
    None,
}
/// Message text parse struct
#[derive(Serialize, Clone, Debug)]
pub struct MessageTextParser {
    /// Array of text formats
    pub(crate) text: Vec<MessageTextFormat>,
    /// ## Parse mode
    /// - `HTML` - HTML
    /// - `MarkdownV2` - Markdown
    pub(crate) parse_mode: ParseMode,
}
/// Keyboard for method [`SendMessagesAPIMethods::MessagesSendText`]
/// One of variants must be set:
/// - {`text`: String,`url`: String,`style`: [`ButtonStyle`]} - simple buttons
/// - {`text`: String,`callback_data`: String,`style`: [`ButtonStyle`]} - buttons with callback
///
/// [`SendMessagesAPIMethods::MessagesSendText`]: enum.SendMessagesAPIMethods.html#variant.MessagesSendText
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ChatId(pub String);
/// Message id struct
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MsgId(pub String);
/// User id struct
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct UserId(pub String);
/// File id struct
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct FileId(pub String);
/// Query id struct
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct QueryId(pub String);
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
/// Show alert struct
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShowAlert(pub bool);
