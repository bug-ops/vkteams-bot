//! Get information about the current user method `self/get`
//! [More info](https://teams.vk.com/botapi/#/self/get_self_get)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// # Get information about the current user method `self/get`
#[derive(Serialize, Clone, Debug, Default)]
pub struct RequestSelfGet;
/// # Get information about the current user method `self/get`
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseSelfGet {
    pub user_id: UserId,
    pub nick: String,
    pub first_name: String,
    #[serde(default)]
    pub about: String,
    #[serde(default)]
    pub photo: Vec<PhotoUrl>,
    pub ok: bool,
}
impl BotRequest for RequestSelfGet {
    const METHOD: &'static str = "self/get";
    type RequestType = Self;
    type ResponseType = ResponseSelfGet;
}
impl RequestSelfGet {
    /// Create a new [`RequestSelfGet`]
    pub fn new() -> Self {
        Self
    }
}
