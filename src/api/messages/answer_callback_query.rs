//! Answer callback query method `messages/answerCallbackQuery`
//! [More info](https://teams.vk.com/botapi/#/messages/get_messages_answerCallbackQuery)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Answer callback query request for method `messages/answerCallbackQuery`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesAnswerCallbackQuery {
    pub query_id: QueryId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_alert: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}
/// Answer callback query response for method `messages/answerCallbackQuery`
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ResponseMessagesAnswerCallbackQuery {
    pub ok: bool,
}
impl BotRequest for RequestMessagesAnswerCallbackQuery {
    const METHOD: &'static str = "messages/answerCallbackQuery";
    type RequestType = Self;
    type ResponseType = ResponseMessagesAnswerCallbackQuery;
}
impl RequestMessagesAnswerCallbackQuery {
    /// Create a new [`RequestMessagesAnswerCallbackQuery`]
    /// ## Parameters
    /// - `query_id`: [`QueryId`]
    pub fn new(query_id: QueryId) -> Self {
        Self {
            query_id,
            ..Default::default()
        }
    }
    /// Set text
    /// ## Parameters
    /// - `text`: [`String`]
    pub fn set_text(mut self, text: String) -> Self {
        self.text = Some(text);
        self.to_owned()
    }
    /// Set show_alert
    /// ## Parameters
    /// - `show_alert`: `bool`
    pub fn set_alert(mut self, show_alert: bool) -> Self {
        self.show_alert = Some(show_alert);
        self.to_owned()
    }
    /// Set url
    /// ## Parameters
    /// - `url`: [`String`]
    pub fn set_url(mut self, url: String) -> Self {
        self.url = Some(url);
        self.to_owned()
    }
}
