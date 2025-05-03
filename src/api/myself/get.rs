//! Get information about the current user method `self/get`
//! [More info](https://teams.vk.com/botapi/#/self/get_self_get)
use crate::api::types::*;
use serde::{Deserialize, Serialize};

bot_api_method! {
    method = "self/get",
    request = RequestSelfGet {
        required {},
        optional {}
    },
    response = ResponseSelfGet {
        user_id: UserId,
        nick: String,
        first_name: String,
        about: String,
        photo: Vec<PhotoUrl>,
        ok: bool,
    },
}
