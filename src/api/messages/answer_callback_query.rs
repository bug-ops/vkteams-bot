use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::MessagesAnswerCallbackQuery`]
///
/// [`SendMessagesAPIMethods::MessagesAnswerCallbackQuery`]: enum.SendMessagesAPIMethods.html#variant.MessagesAnswerCallbackQuery
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessagesAnswerCallbackQuery {
    pub query_id: QueryId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_alert: Option<ShowAlert>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}
/// Response for method [`SendMessagesAPIMethods::MessagesAnswerCallbackQuery`]
///
/// [`SendMessagesAPIMethods::MessagesAnswerCallbackQuery`]: enum.SendMessagesAPIMethods.html#variant.MessagesAnswerCallbackQuery
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ResponseMessagesAnswerCallbackQuery {
    pub ok: bool,
}
impl BotRequest for RequestMessagesAnswerCallbackQuery {
    const METHOD: &'static str = "messages/answerCallbackQuery";
    type RequestType = Self;
    type ResponseType = ResponseMessagesAnswerCallbackQuery;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::MessagesAnswerCallbackQuery(query_id, text, show_alert, url) => Self {
                query_id: query_id.to_owned(),
                text: text.to_owned(),
                show_alert: show_alert.to_owned(),
                url: url.to_owned(),
            },
            _ => panic!("Wrong API method for RequestMessagesAnswerCallbackQuery"),
        }
    }
}
