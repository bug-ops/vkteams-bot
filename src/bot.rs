use anyhow::{anyhow, Result};
use std::sync::Arc;
use tokio::fs::File;
// pub mod db; //TODO: Add db module
pub mod net;
pub mod types;
pub mod utils;
use crate::net::*;
use crate::types::*;

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
            event_id: Arc::new(0),
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
        self.send_get_request::<RequestEventsGet, ResponseEventsGet, Methods>(
            RequestEventsGet {
                last_event_id: Arc::clone(&self.event_id).to_string(),
                poll_time: POLL_TIME.to_string(),
            },
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
        self.send_get_request::<RequestSelfGet, ResponseSelfGet, Methods>(
            RequestSelfGet {},
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
        self.send_get_request::<RequestMessagesSendText, ResponseMessagesSendText, Methods>(
            request_message,
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
            Methods,
        >(
            request_message,
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
        self.send_get_request::<RequestMessagesEditText, ResponseMessagesEditText, Methods>(
            request_message,
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
        self.send_get_request::<RequestMessagesDeleteMessages, ResponseMessagesDeleteMessages, Methods>(
            request_message,
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
        self.send_get_request::<RequestMessagesAnswerCallbackQuery, ResponseMessagesAnswerCallbackQuery, Methods>(
            request_message,
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
        let file = File::open(file_path).await;
        match file {
            Ok(f) => self
                .send_post_request::<RequestMessagesSendFile, ResponseMessagesSendFile, Methods>(
                    request_message,
                    Methods::MessagesSendFile,
                    MultipartName::File(f),
                )
                .await,
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
        let file = File::open(file_path).await;
        match file {
            Ok(f) => self
                .send_post_request::<RequestMessagesSendVoice, ResponseMessagesSendVoice, Methods>(
                    request_message,
                    Methods::MessagesSendVoice,
                    MultipartName::File(f),
                )
                .await,
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
        let file = File::open(file_path).await;
        match file {
            Ok(f) => {
                self.send_post_request::<RequestChatsAvatarSet, ResponseChatsAvatarSet, Methods>(
                    request_message,
                    Methods::ChatsAvatarSet,
                    MultipartName::Image(f),
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
        self.send_get_request::<RequestChatsSendAction, ResponseChatsSendAction, Methods>(
            request_message,
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
        self.send_get_request::<RequestChatsGetInfo, ResponseChatsGetInfo, Methods>(
            request_message,
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
        self.send_get_request::<RequestChatsGetAdmins, ResponseChatsGetAdmins, Methods>(
            request_message,
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
        self.send_get_request::<RequestChatsGetMembers, ResponseChatsGetMembers, Methods>(
            request_message,
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
        self.send_get_request::<RequestChatsMembersDelete, ResponseChatsMembersDelete, Methods>(
            request_message,
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
        self.send_get_request::<RequestChatsSetTitle, ResponseChatsSetTitle, Methods>(
            request_message,
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
        self.send_get_request::<RequestChatsSetAbout, ResponseChatsSetAbout, Methods>(
            request_message,
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
        self.send_get_request::<RequestChatsSetRules, ResponseChatsSetRules, Methods>(
            request_message,
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
        self.send_get_request::<RequestChatsPinMessage, ResponseChatsPinMessage, Methods>(
            request_message,
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
        self.send_get_request::<RequestChatsUnpinMessage, ResponseChatsUnpinMessage, Methods>(
            request_message,
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
        self.send_get_request::<RequestFilesGetInfo, ResponseFilesGetInfo, Methods>(
            request_message,
            Methods::FilesGetInfo,
        )
        .await
    }

    // --CHATS--
    //
    // TODO: [GET]  /chats/getBlockedUsers
    // TODO: [GET]  /chats/getPendingUsers
    // TODO: [GET]  /chats/blockUser
    // TODO: [GET]  /chats/unblockUser
    // TODO: [GET]  /chats/resolvePending
}
