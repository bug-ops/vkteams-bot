use anyhow::{anyhow, Result};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::fs::File;
// pub mod db; //TODO: Add db module
pub mod net;
pub mod types;
pub mod utils;
use crate::net::*;
use crate::types::*;

impl Default for Bot {
    // default API version V1
    fn default() -> Self {
        Self::new(APIVersionUrl::V1)
    }
}
impl Bot {
    /// Creates a new `Bot` with API version [`APIVersionUrl`]
    ///
    /// Build optional proxy from .env variable `VKTEAMS_PROXY` if its bound, passes to [`reqwest::Proxy::all`].
    ///
    /// Build [`reqwest::Client`] with default settings.
    ///
    /// Get token from variable `VKTEAMS_BOT_API_TOKEN` in .env file
    ///
    /// Get base url from variable `VKTEAMS_BOT_API_URL` in .env file
    ///
    /// Set default path depending on API version
    /// ## Panics
    /// - Unable to parse proxy if its bound in `VKTEAMS_PROXY` env variable
    /// - Unable to build [`reqwest::Client`]
    /// - Unable to find token in .env file
    /// - Unable to find or parse url in .env file
    ///
    /// [`reqwest::Client`]: https://docs.rs/reqwest/latest/reqwest/struct.Client.html
    /// [`reqwest::Proxy::all`]: https://docs.rs/reqwest/latest/reqwest/struct.Proxy.html#method.all
    pub fn new(version: APIVersionUrl) -> Self {
        use reqwest::Proxy;
        // Set default reqwest settings
        let builder = default_reqwest_settings();
        // Set proxy if it is bound
        let client = match std::env::var(VKTEAMS_PROXY).ok() {
            Some(proxy) => builder.proxy(Proxy::all(proxy).expect("Unable to parse proxy")),
            None => builder,
        }
        .build()
        .expect("Unable to build reqwest client");

        Self {
            client,
            // Get token from .env file
            token: get_env_token(),
            // Get API URL from .env file
            base_api_url: get_env_url(),
            // Set default path depending on API version
            base_api_path: set_default_path(&version),
            // Default event id is 0
            event_id: Arc::new(Mutex::new(0)),
        }
    }
    /// Get chat events
    /// HTTP Method `GET`
    /// path `/events/get`
    /// query {`token`,`lastEventId`,`pollTime`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/events/get_events_get
    pub async fn get_events(&self) -> Result<ResponseEventsGet> {
        // Get last event id
        let counter: Arc<Mutex<u64>> = Arc::clone(&self.event_id);
        let event = *counter.lock().unwrap();
        self.send_get_request::<RequestEventsGet, ResponseEventsGet>(
            RequestEventsGet {
                last_event_id: event,
                poll_time: POLL_TIME.to_string(),
            },
            MultipartName::None,
            Methods::EventsGet,
        )
        .await
    }
    /// Get bot info
    /// HTTP Method `GET`
    /// path `/self/get`
    /// query {`token`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/self/get_self_get
    pub async fn self_get(&self) -> Result<ResponseSelfGet> {
        self.send_get_request::<RequestSelfGet, ResponseSelfGet>(
            RequestSelfGet {},
            MultipartName::None,
            Methods::SelfGet,
        )
        .await
    }
    /// Send text message to chat
    /// HTTP Metthod `GET`
    /// path `/messages/sendText`
    /// query {`token`,`chatId`,`text`,`replyMsgId`,`forwardChatId`,`forwardMsgId`,`inlineKeyboardMarkup`,`format`,`parseMode`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/messages/get_messages_sendText
    pub async fn messages_send_text(
        &self,
        request_message: RequestMessagesSendText,
    ) -> Result<ResponseMessagesSendText> {
        self.send_get_request::<RequestMessagesSendText, ResponseMessagesSendText>(
            request_message,
            MultipartName::None,
            Methods::MessagesSendText,
        )
        .await
    }
    /// Send text message with deeplink to chat
    /// HTTP Method `GET`
    /// path `/messages/sendTextWithDeeplink`
    /// query {`token`,`chatId`,`text`,`replyMsgId`,`forwardChatId`,`forwardMsgId`,`inlineKeyboardMarkup`,`format`,`parseMode`,`deeplink`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/messages/get_messages_sendTextWithDeeplink
    pub async fn request_messages_send_text_with_deeplink(
        &self,
        request_message: RequestMessagesSendTextWithDeepLink,
    ) -> Result<ResponseMessagesSendTextWithDeepLink> {
        self.send_get_request::<
            RequestMessagesSendTextWithDeepLink,
            ResponseMessagesSendTextWithDeepLink,
        >(
            request_message,
            MultipartName::None,
            Methods::MessagesSendTextWithDeepLink,
        )
        .await
    }
    /// Edit text message in chat
    /// HTTP Method `GET`
    /// path `/messages/editText`
    /// query {`token`,`chatId`,`msgId`,`text`,`inlineKeyboardMarkup`,`format`,`parseMode`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/messages/get_messages_editText
    pub async fn messages_edit_text(
        &self,
        request_message: RequestMessagesEditText,
    ) -> Result<ResponseMessagesEditText> {
        self.send_get_request::<RequestMessagesEditText, ResponseMessagesEditText>(
            request_message,
            MultipartName::None,
            Methods::MessagesEditText,
        )
        .await
    }
    /// Delete text message in chat
    /// HTTP Method `GET`
    /// path /messages/deleteMessages
    /// query {`token`,`chatId`,`msgId`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/messages/get_messages_deleteMessages
    pub async fn messages_delete_messages(
        &self,
        request_message: RequestMessagesDeleteMessages,
    ) -> Result<ResponseMessagesDeleteMessages> {
        self.send_get_request::<RequestMessagesDeleteMessages, ResponseMessagesDeleteMessages>(
            request_message,
            MultipartName::None,
            Methods::MessagesDeleteMessages,
        )
        .await
    }
    /// Answer callback query
    /// HTTP Method `GET`
    /// path `/messages/answerCallbackQuery`
    /// query {`queryId`,`text`,`showAlert`,`url`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/messages/get_messages_answerCallbackQuery
    pub async fn messages_answer_callback_query(
        &self,
        request_message: RequestMessagesAnswerCallbackQuery,
    ) -> Result<ResponseMessagesAnswerCallbackQuery> {
        self.send_get_request::<RequestMessagesAnswerCallbackQuery, ResponseMessagesAnswerCallbackQuery>(
            request_message,
            MultipartName::None,
            Methods::MessagesAnswerCallbackQuery,
        )
        .await
    }
    /// Send file to chat
    /// HTTP Method `POST`
    /// path `/messages/sendFile`
    /// query {`token`,`chatId`, `caption`,`replyMsgId`,`forwardChatId`,`forwardMsgId`,`inlineKeyboardMarkup`,`format`,`parseMode`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/messages/post_messages_sendFile
    pub async fn messages_send_file(
        &self,
        request_message: RequestMessagesSendFile,
        file_path: String,
    ) -> Result<ResponseMessagesSendFile> {
        let path = Path::new(&file_path);
        let file = File::open(path.to_owned()).await;
        match file {
            Ok(f) => {
                self.send_get_request::<RequestMessagesSendFile, ResponseMessagesSendFile>(
                    request_message,
                    MultipartName::File {
                        name: path.file_name().unwrap().to_string_lossy().to_string(),
                        file: f,
                    },
                    Methods::MessagesSendFile,
                )
                .await
            }
            Err(e) => Err(anyhow!(e)),
        }
    }
    /// Send voice message to chat
    /// HTTP Method `POST`
    /// path `/messages/sendVoice`
    /// query {`token`,`chatId`,`replyMsgId`,`forwardChatId`,`forwardMsgId`,`inlineKeyboardMarkup`,`format`,`parseMode`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/messages/post_messages_sendVoice
    pub async fn messages_send_voice(
        &self,
        request_message: RequestMessagesSendVoice,
        file_path: String,
    ) -> Result<ResponseMessagesSendVoice> {
        let path = Path::new(&file_path);
        let file = File::open(path.to_owned()).await;
        match file {
            Ok(f) => {
                self.send_get_request::<RequestMessagesSendVoice, ResponseMessagesSendVoice>(
                    request_message,
                    MultipartName::File {
                        name: path.file_name().unwrap().to_string_lossy().to_string(),
                        file: f,
                    },
                    Methods::MessagesSendVoice,
                )
                .await
            }
            Err(e) => Err(anyhow!(e)),
        }
    }
    /// Set chat avatar
    /// HTTP Method `POST`
    /// path `/chats/avatar/set`
    /// query {`token`,`chatId`}
    /// file `image`
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/chats/post_chats_avatar_set
    pub async fn chats_avatar_set(
        &self,
        request_message: RequestChatsAvatarSet,
        file_path: String,
    ) -> Result<ResponseChatsAvatarSet> {
        let path = Path::new(&file_path);
        let file = File::open(path.to_owned()).await;
        match file {
            Ok(f) => {
                self.send_get_request::<RequestChatsAvatarSet, ResponseChatsAvatarSet>(
                    request_message,
                    MultipartName::Image {
                        name: path.file_name().unwrap().to_string_lossy().to_string(),
                        file: f,
                    },
                    Methods::ChatsAvatarSet,
                )
                .await
            }
            Err(e) => Err(anyhow!(e)),
        }
    }
    /// Send typing action to chat
    /// HTTP Method `GET`
    /// path `/chats/sendAction`
    /// query {`token`,`chatId`,`action`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/chats/get_chats_sendActions
    pub async fn chats_send_action(
        &self,
        request_message: RequestChatsSendAction,
    ) -> Result<ResponseChatsSendAction> {
        self.send_get_request::<RequestChatsSendAction, ResponseChatsSendAction>(
            request_message,
            MultipartName::None,
            Methods::ChatsSendActions,
        )
        .await
    }
    /// Get chat info
    /// HTTP Method `GET`
    /// path `/chats/getInfo`
    /// query {`token`,`chatId`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/chats/get_chats_getInfo
    pub async fn chats_get_info(
        &self,
        request_message: RequestChatsGetInfo,
    ) -> Result<ResponseChatsGetInfo> {
        self.send_get_request::<RequestChatsGetInfo, ResponseChatsGetInfo>(
            request_message,
            MultipartName::None,
            Methods::ChatsGetInfo,
        )
        .await
    }
    /// Get chat admins
    /// HTTP Method 'GET'
    /// path `/chats/getAdmins`
    /// query {`token`,`chatId`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/chats/get_chats_getInfo
    pub async fn chats_get_admins(
        &self,
        request_message: RequestChatsGetAdmins,
    ) -> Result<ResponseChatsGetAdmins> {
        self.send_get_request::<RequestChatsGetAdmins, ResponseChatsGetAdmins>(
            request_message,
            MultipartName::None,
            Methods::ChatsGetAdmins,
        )
        .await
    }
    /// Get chat members
    /// HTTP Method `GET`
    /// path `/chats/getMembers`
    /// query {`token`,`chatId`,`cursor`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/chats/get_chats_getMembers
    pub async fn chats_get_members(
        &self,
        request_message: RequestChatsGetMembers,
    ) -> Result<ResponseChatsGetMembers> {
        self.send_get_request::<RequestChatsGetMembers, ResponseChatsGetMembers>(
            request_message,
            MultipartName::None,
            Methods::ChatsGetMembers,
        )
        .await
    }
    /// Delete chat members
    /// HTTP Method `GET`
    /// path `/chats/members/delete`
    /// query {`token`,`chatId`,`members`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/chats/get_chats_members_delete
    pub async fn chats_members_delete(
        &self,
        request_message: RequestChatsMembersDelete,
    ) -> Result<ResponseChatsMembersDelete> {
        self.send_get_request::<RequestChatsMembersDelete, ResponseChatsMembersDelete>(
            request_message,
            MultipartName::None,
            Methods::ChatsMembersDelete,
        )
        .await
    }
    /// Set chat title
    /// HTTP Method `GET`
    /// path `/chats/setTitle`
    /// query {`token`,`chatId`,`title`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/chats/get_chats_setTitle
    pub async fn chats_set_title(
        &self,
        request_message: RequestChatsSetTitle,
    ) -> Result<ResponseChatsSetTitle> {
        self.send_get_request::<RequestChatsSetTitle, ResponseChatsSetTitle>(
            request_message,
            MultipartName::None,
            Methods::ChatsSetTitle,
        )
        .await
    }
    /// Set chat about
    /// HTTP Method `GET`
    /// path `/chats/setAbout`
    /// query {`token`,`chatId`,`about`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/chats/get_chats_setAbout
    pub async fn chats_set_about(
        &self,
        request_message: RequestChatsSetAbout,
    ) -> Result<ResponseChatsSetAbout> {
        self.send_get_request::<RequestChatsSetAbout, ResponseChatsSetAbout>(
            request_message,
            MultipartName::None,
            Methods::ChatsSetAbout,
        )
        .await
    }
    /// Set chat rules
    /// HTTP Method `GET`
    /// path `/chats/setRules`
    /// query {`token`,`chatId`,`rules`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/chats/get_chats_setRules
    pub async fn chats_set_rules(
        &self,
        request_message: RequestChatsSetRules,
    ) -> Result<ResponseChatsSetRules> {
        self.send_get_request::<RequestChatsSetRules, ResponseChatsSetRules>(
            request_message,
            MultipartName::None,
            Methods::ChatsSetRules,
        )
        .await
    }
    /// Pin message in chat
    /// HTTP Method `GET`
    /// path `/chats/pinMessage`
    /// query {`token`,`chatId`,`msgId`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/chats/get_chats_pinMessage
    pub async fn chats_pin_message(
        &self,
        request_message: RequestChatsPinMessage,
    ) -> Result<ResponseChatsPinMessage> {
        self.send_get_request::<RequestChatsPinMessage, ResponseChatsPinMessage>(
            request_message,
            MultipartName::None,
            Methods::ChatsPinMessage,
        )
        .await
    }
    /// Unpin message in chat
    /// HTTP Method `GET`
    /// path `/chats/unpinMessage`
    /// query {`token`,`chatId`,`msgId`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/chats/get_chats_unpinMessage
    pub async fn chats_unpin_message(
        &self,
        request_message: RequestChatsUnpinMessage,
    ) -> Result<ResponseChatsUnpinMessage> {
        self.send_get_request::<RequestChatsUnpinMessage, ResponseChatsUnpinMessage>(
            request_message,
            MultipartName::None,
            Methods::ChatsUnpinMessage,
        )
        .await
    }
    /// Files get info
    /// HTTP Method `GET`
    /// path `/files/getInfo`
    /// query {`token`,`fileId`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/files/get_files_getInfo
    pub async fn files_get_info(
        &self,
        request_message: RequestFilesGetInfo,
    ) -> Result<ResponseFilesGetInfo> {
        self.send_get_request::<RequestFilesGetInfo, ResponseFilesGetInfo>(
            request_message,
            MultipartName::None,
            Methods::FilesGetInfo,
        )
        .await
    }
    /// Get blocked users
    /// HTTP Method `GET`
    /// path `/chats/getBlockedUsers`
    /// query {`token`,`chatId`,`
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/chats/get_chats_getBlockedUsers}
    pub async fn chats_get_blocked_users(
        &self,
        request_message: RequestChatsGetBlockedUsers,
    ) -> Result<ResponseChatsGetBlockedUsers> {
        self.send_get_request::<RequestChatsGetBlockedUsers, ResponseChatsGetBlockedUsers>(
            request_message,
            MultipartName::None,
            Methods::ChatsGetBlockedUsers,
        )
        .await
    }
    /// Get pending users
    /// HTTP Method `GET`
    /// path `/chats/getPendingUsers`
    /// query {`token`,`chatId`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/chats/get_chats_getPendingUsers
    pub async fn chats_get_pending_users(
        &self,
        request_message: RequestChatsGetPendingUsers,
    ) -> Result<ResponseChatsGetPendingUsers> {
        self.send_get_request::<RequestChatsGetPendingUsers, ResponseChatsGetPendingUsers>(
            request_message,
            MultipartName::None,
            Methods::ChatsGetPendingUsers,
        )
        .await
    }
    /// Block user
    /// HTTP Method `GET`
    /// path `/chats/blockUser`
    /// query {`token`, `chatId`, `userId`, `delLastMessages`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/chats/get_chats_blockUser
    pub async fn chats_block_user(
        &self,
        request_message: RequestChatsBlockUser,
    ) -> Result<ResponseChatsBlockUser> {
        self.send_get_request::<RequestChatsBlockUser, ResponseChatsBlockUser>(
            request_message,
            MultipartName::None,
            Methods::ChatsBlockUser,
        )
        .await
    }
    /// Unblock user
    /// HTTP Method `GET`
    /// path `/chats/unblockUser`
    /// query {`token`,`chatId`,`userId`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/chats/get_chats_unblockUser
    pub async fn chats_unblock_user(
        &self,
        request_message: RequestChatsUnblockUser,
    ) -> Result<ResponseChatsUnblockUser> {
        self.send_get_request::<RequestChatsUnblockUser, ResponseChatsUnblockUser>(
            request_message,
            MultipartName::None,
            Methods::ChatsUnblockUser,
        )
        .await
    }
    /// Resolve pending
    /// HTTP Method `GET`
    /// path `/chats/resolvePending`
    /// query {`token`,`chatId`,`approve`,`userId`,`everyone`}
    ///
    /// See the details in [VKTeams Bot API]
    ///
    /// [VKTeams Bot API]: https://teams.vk.com/botapi/?lang=en#/chats/get_chats_resolvePending
    pub async fn chats_resolve_pending(
        &self,
        request_message: RequestChatsResolvePending,
    ) -> Result<ResponseChatsResolvePending> {
        self.send_get_request::<RequestChatsResolvePending, ResponseChatsResolvePending>(
            request_message,
            MultipartName::None,
            Methods::ChatsResolvePending,
        )
        .await
    }
}
