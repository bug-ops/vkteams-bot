use crate::api::types::*;
use anyhow::Result;
use reqwest::{Client, Url};
use serde::Serialize;
use std::sync::{Arc, Mutex};

/// Bot class with attributes
/// - `client`: [`reqwest::Client`]
/// - `token`: String
/// - `base_api_url`: [`reqwest::Url`]
/// - `base_api_path`: String
/// - `evtent_id`: [`std::sync::Arc<_>`]
///
/// [`reqwest::Client`]: https://docs.rs/reqwest/latest/reqwest/struct.Client.html
/// [`reqwest::Url`]: https://docs.rs/reqwest/latest/reqwest/struct.Url.html
/// [`std::sync::Arc<_>`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
pub struct Bot {
    pub(crate) client: Client,
    pub(crate) token: String,
    pub(crate) base_api_url: Url,
    pub(crate) base_api_path: String,
    pub(crate) event_id: Arc<Mutex<u64>>,
}
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
    /// Get last event id
    pub fn get_last_event_id(&self) -> u64 {
        *self.event_id.lock().unwrap()
    }
    /// Set last event id
    pub fn set_last_event_id(&self, id: u64) {
        *self.event_id.lock().unwrap() = id;
    }
    /// Listen for events and execute callback function
    /// - `func` - callback function with `Result<ResponseEventsGet>` as argument
    pub async fn event_listener(&self, func: impl Fn(&Result<ResponseEventsGet>)) {
        loop {
            // Make a request to the API
            let req = RequestEventsGet::new(&Methods::EventsGet(self.get_last_event_id()));
            // Get response
            let res = self.send::<RequestEventsGet>(req).await;
            // Execute callback function
            func(&res);
            // Update last event id
            match &res {
                Ok(events) => {
                    let evt = events.events.clone();
                    // If at least one event read
                    if !evt.is_empty() {
                        // Update last event id
                        self.set_last_event_id(evt[evt.len() - 1].event_id);
                    }
                }
                Err(e) => {
                    debug!("Error: {:?}", e);
                }
            }
        }
    }
    /// Append method path to `base_api_path`
    /// - `path` - append path to `base_api_path`
    pub fn set_path(&self, path: String) -> String {
        // Get base API path
        let mut full_path = self.base_api_path.clone();
        // Append method path
        full_path.push_str(&path);
        // Return full path
        full_path
    }
    /// Build full URL with optional query parameters
    /// - `path` - append path to `base_api_path`
    /// - `query` - append `token` query parameter to URL
    /// Parse with [`Url::parse`]
    ///
    /// [`base_api_url`]: #structfield.base_api_url
    pub fn get_parsed_url(&self, path: String, query: String) -> Result<Url> {
        // Make URL with base API path
        let url = Url::parse(self.base_api_url.as_str());
        match url {
            Ok(mut u) => {
                // Append path to URL
                u.set_path(&path);
                //Set bound query
                u.set_query(Some(&query));
                // Append default query query
                u.query_pairs_mut().append_pair("token", &self.token);
                Ok(u)
            }
            // Error with URL
            Err(e) => {
                error!("Error parse URL: {}", e);
                Err(e.into())
            }
        }
    }
    /// Send request, get response
    /// Serialize request generic type `Rq` with [`serde_url_params::to_string`] into query string
    /// Get response body with [`response`]
    /// Deserialize response with [`serde_json::from_str`]
    ///
    /// [`response`]: #method.response
    pub async fn send<Rq>(&self, message: Rq) -> Result<<Rq>::ResponseType>
    where
        Rq: BotRequest + Serialize,
    {
        // Serialize request type `Rq` with serde_url_params::to_string into query string
        match serde_url_params::to_string(&message) {
            Ok(query) => {
                // Try to parse URL
                match self.get_parsed_url(self.set_path(<Rq>::METHOD.to_string()), query.to_owned())
                {
                    Ok(url) => {
                        // Get response body
                        let body = match <Rq>::HTTP_METHOD {
                            HTTPMethod::POST => {
                                // For POST method get multipart form from file name
                                match file_to_multipart(message.get_file()).await {
                                    Ok(f) => {
                                        // Send file POST request with multipart form
                                        post_response_file(
                                            self.client.clone(),
                                            self.get_parsed_url(
                                                self.set_path(<Rq>::METHOD.to_string()),
                                                query,
                                            )
                                            .unwrap(),
                                            f,
                                        )
                                        .await
                                    }
                                    // Error with file
                                    Err(e) => return Err(e),
                                }
                            }
                            HTTPMethod::GET => {
                                // Simple GET request
                                get_response(self.client.clone(), url).await
                            }
                        };
                        // Deserialize response with serde_json::from_str
                        match body {
                            Ok(b) => {
                                let rs = serde_json::from_str::<<Rq>::ResponseType>(b.as_str());
                                match rs {
                                    Ok(r) => Ok(r),
                                    Err(e) => Err(e.into()),
                                }
                            }
                            // Error with response
                            Err(e) => Err(e),
                        }
                    }
                    // Error with URL
                    Err(e) => {
                        error!("Error parse URL: {}", e);
                        Err(e)
                    }
                }
            }
            // Error with parse query
            Err(e) => {
                error!("Error serialize request: {}", e);
                Err(e.into())
            }
        }
    }
    /// Download file from URL
    pub async fn files_download(&self, url: String) -> Result<Vec<u8>> {
        match Url::parse(url.as_str()) {
            Ok(u) => match self.client.get(u.as_str()).send().await {
                Ok(r) => {
                    debug!("Response status: OK");
                    match r.bytes().await {
                        Ok(b) => Ok(b.to_vec()),
                        Err(e) => Err(e.into()),
                    }
                }
                Err(e) => {
                    error!("Response status: {}", e);
                    Err(e.into())
                }
            },
            Err(e) => Err(e.into()),
        }
    }
    /// Method `chats/avatarSet` for upload avatar image
    /// - `file` - local file name path for upload
    pub async fn chats_avatar_set(
        &self,
        chat_id: ChatId,
        file: String,
    ) -> Result<ResponseChatsAvatarSet> {
        let rq = RequestChatsAvatarSet::new(&Methods::ChatsAvatarSet(
            chat_id,
            MultipartName::Image(file),
        ));
        self.send(rq).await
    }
    /// Method `chats/blockUser` for block user in chat
    /// - `del_last_message` - delete last message send by user
    pub async fn chats_block_user(
        &self,
        chat_id: ChatId,
        user_id: UserId,
        del_last_message: bool,
    ) -> Result<ResponseChatsBlockUser> {
        let rq = RequestChatsBlockUser::new(&Methods::ChatsBlockUser(
            chat_id,
            user_id,
            del_last_message,
        ));
        self.send(rq).await
    }
    /// Method `chats/getAdmins` for get chat admins
    pub async fn chats_get_admins(&self, chat_id: ChatId) -> Result<ResponseChatsGetAdmins> {
        let rq = RequestChatsGetAdmins::new(&Methods::ChatsGetAdmins(chat_id));
        self.send(rq).await
    }
    /// Method `chats/getBlockedUsers` for get blocked users in chat
    pub async fn chats_get_blocked_users(
        &self,
        chat_id: ChatId,
    ) -> Result<ResponseChatsGetBlockedUsers> {
        let rq = RequestChatsGetBlockedUsers::new(&Methods::ChatsGetBlockedUsers(chat_id));
        self.send(rq).await
    }
    /// Method `chats/getInfo` for get chat info
    pub async fn chats_get_info(&self, chat_id: ChatId) -> Result<ResponseChatsGetInfo> {
        let rq = RequestChatsGetInfo::new(&Methods::ChatsGetInfo(chat_id));
        self.send(rq).await
    }
    /// Method `chats/getMembers` for get chat members
    pub async fn chats_get_members(&self, chat_id: ChatId) -> Result<ResponseChatsGetMembers> {
        let rq = RequestChatsGetMembers::new(&Methods::ChatsGetMembers(chat_id));
        self.send(rq).await
    }
    /// Method `chats/getPendingUsers` for get pending users in chat
    pub async fn chats_get_pending_users(
        &self,
        chat_id: ChatId,
    ) -> Result<ResponseChatsGetPendingUsers> {
        let rq = RequestChatsGetPendingUsers::new(&Methods::ChatsGetPendingUsers(chat_id));
        self.send(rq).await
    }
    /// Method `chats/membersDelete` for delete user from chat
    pub async fn chats_members_delete(
        &self,
        chat_id: ChatId,
        user_id: UserId,
    ) -> Result<ResponseChatsMembersDelete> {
        let rq = RequestChatsMembersDelete::new(&Methods::ChatsMembersDelete(chat_id, user_id));
        self.send(rq).await
    }
    /// Method `chats/pinMessage` for pin message in chat
    pub async fn chats_pin_message(
        &self,
        chat_id: ChatId,
        msg_id: MsgId,
    ) -> Result<ResponseChatsPinMessage> {
        let rq = RequestChatsPinMessage::new(&Methods::ChatsPinMessage(chat_id, msg_id));
        self.send(rq).await
    }
    /// Method `chats/resolverPending` for resolve pending users in chat
    /// - `approve` - approve or decline
    /// - `everyone` - approve or decline all
    pub async fn chats_resolve_pending(
        &self,
        chat_id: ChatId,
        approve: bool,
        user_id: Option<UserId>,
        everyone: Option<bool>,
    ) -> Result<ResponseChatsResolvePending> {
        let rq = RequestChatsResolvePending::new(&Methods::ChatsResolvePending(
            chat_id, approve, user_id, everyone,
        ));
        self.send(rq).await
    }
    /// Method `chats/sendActions` for send chat actions
    pub async fn chats_send_actions(
        &self,
        chat_id: ChatId,
        action_type: ChatActions,
    ) -> Result<ResponseChatsSendAction> {
        let rq = RequestChatsSendAction::new(&Methods::ChatsSendAction(chat_id, action_type));
        self.send(rq).await
    }
    /// Method `chats/setAbout` for set chat about
    /// - `about` - text chat about
    pub async fn set_about(&self, chat_id: ChatId, about: String) -> Result<ResponseChatsSetAbout> {
        let rq = RequestChatsSetAbout::new(&Methods::ChatsSetAbout(chat_id, about));
        self.send(rq).await
    }
    /// Method `chats/setRules` for set chat rules
    /// - `rules` - text chat rules
    pub async fn set_rules(&self, chat_id: ChatId, rules: String) -> Result<ResponseChatsSetRules> {
        let rq = RequestChatsSetRules::new(&Methods::ChatsSetRules(chat_id, rules));
        self.send(rq).await
    }
    /// Method `chats/setTitle` for set chat title
    /// - `title` - text chat title
    pub async fn chats_set_title(
        &self,
        chat_id: ChatId,
        title: String,
    ) -> Result<ResponseChatsSetTitle> {
        let rq = RequestChatsSetTitle::new(&Methods::ChatsSetTitle(chat_id, title));
        self.send(rq).await
    }
    /// Method `chats/unblockUser` for unblock user in chat
    pub async fn chats_unblock_user(
        &self,
        chat_id: ChatId,
        user_id: UserId,
    ) -> Result<ResponseChatsUnblockUser> {
        let rq = RequestChatsUnblockUser::new(&Methods::ChatsUnblockUser(chat_id, user_id));
        self.send(rq).await
    }
    /// Method `chats/unpinMessage` for unpin message in chat
    pub async fn chats_unpin_message(
        &self,
        chat_id: ChatId,
        msg_id: MsgId,
    ) -> Result<ResponseChatsUnpinMessage> {
        let rq = RequestChatsUnpinMessage::new(&Methods::ChatsUnpinMessage(chat_id, msg_id));
        self.send(rq).await
    }
    /// Method `events/get` for get bot API events
    pub async fn events_get(&self) -> Result<ResponseEventsGet> {
        let rq = RequestEventsGet::new(&Methods::EventsGet(self.get_last_event_id()));
        self.send(rq).await
    }
    /// Method `files/getInfo` for get file info
    pub async fn files_get_info(&self, file_id: FileId) -> Result<ResponseFilesGetInfo> {
        self.send(RequestFilesGetInfo::new(&Methods::FilesGetInfo(file_id)))
            .await
    }
    /// Method `messages/answerCallbackQuery`
    /// - `query_id` - identifier callback query received by the bot
    /// - `text` - the text of the notification to be displayed to the user. If the text is not specified, nothing will be displayed.
    /// - `show_alert` - if `true` show alert instead of notification
    /// - `url` - URL to be opened by the client application
    pub async fn messages_answer_callback_query(
        &self,
        query_id: QueryId,
        text: Option<String>,
        show_alert: Option<ShowAlert>,
        url: Option<String>,
    ) -> Result<ResponseMessagesAnswerCallbackQuery> {
        let rq = RequestMessagesAnswerCallbackQuery::new(&Methods::MessagesAnswerCallbackQuery(
            query_id, text, show_alert, url,
        ));
        self.send(rq).await
    }
    /// Method `messages/deleteMessages` for delete messages in chat
    pub async fn messages_delete_messages(
        &self,
        chat_id: ChatId,
        msg_id: MsgId,
    ) -> Result<ResponseMessagesDeleteMessages> {
        //TODO: Add delete for multiple messages
        let rq =
            RequestMessagesDeleteMessages::new(&Methods::MessagesDeleteMessages(chat_id, msg_id));
        self.send(rq).await
    }
    /// Method `messages/editText` for edit message text
    pub async fn messages_edit_text(
        &self,
        chat_id: ChatId,
        msg_id: MsgId,
        parser: Option<MessageTextParser>,
    ) -> Result<ResponseMessagesEditText> {
        let rq = RequestMessagesEditText::new(&Methods::MessagesEditText(chat_id, msg_id))
            .set_text(parser)
            .to_owned();
        self.send(rq).await
    }
    /// Method `messages/sendFile` for send file with text caption, keyboard, forward or reply messages
    #[allow(clippy::too_many_arguments)]
    pub async fn messages_send_file(
        &self,
        chat_id: ChatId,
        file: String,
        parser: Option<MessageTextParser>,
        keyboard: Option<Keyboard>,
        forward_msg_id: Option<MsgId>,
        forward_chat_id: Option<ChatId>,
        reply_msg_id: Option<MsgId>,
    ) -> Result<ResponseMessagesSendFile> {
        let rq = RequestMessagesSendFile::new(&Methods::MessagesSendFile(
            chat_id,
            MultipartName::File(file),
        ))
        .set_text(parser)
        .set_keyboard(keyboard)
        .set_forward_msg_id(forward_chat_id, forward_msg_id)
        .set_reply_msg_id(reply_msg_id)
        .to_owned();
        self.send(rq).await
    }
    /// Method `messages/sendVoice` for send voice message with text caption, keyboard, forward or reply messages
    #[allow(clippy::too_many_arguments)]
    pub async fn messages_send_voice(
        &self,
        chat_id: ChatId,
        file: String,
        parser: Option<MessageTextParser>,
        keyboard: Option<Keyboard>,
        forward_msg_id: Option<MsgId>,
        forward_chat_id: Option<ChatId>,
        reply_msg_id: Option<MsgId>,
    ) -> Result<ResponseMessagesSendVoice> {
        let rq = RequestMessagesSendVoice::new(&Methods::MessagesSendVoice(
            chat_id,
            MultipartName::File(file),
        ))
        .set_text(parser)
        .set_keyboard(keyboard)
        .set_forward_msg_id(forward_chat_id, forward_msg_id)
        .set_reply_msg_id(reply_msg_id)
        .to_owned();
        self.send(rq).await
    }
    /// Method `messages/sendText` for send text message with keyboard, forward or reply messages
    pub async fn messages_send_text(
        &self,
        chat_id: ChatId,
        parser: Option<MessageTextParser>,
        keyboard: Option<Keyboard>,
        forward_msg_id: Option<MsgId>,
        forward_chat_id: Option<ChatId>,
        reply_msg_id: Option<MsgId>,
    ) -> Result<ResponseMessagesSendText> {
        let rq = RequestMessagesSendText::new(&Methods::MessagesSendText(chat_id))
            .set_text(parser)
            .set_keyboard(keyboard)
            .set_forward_msg_id(forward_chat_id, forward_msg_id)
            .set_reply_msg_id(reply_msg_id)
            .to_owned();
        self.send(rq).await
    }
    /// Method `self/get` for get bot info
    pub async fn self_get(&self) -> Result<ResponseSelfGet> {
        let rq = RequestSelfGet::new(&Methods::SelfGet());
        self.send(rq).await
    }
}
