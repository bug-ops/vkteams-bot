//! Delete members from the chat method `chats/members/delete`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_members_delete)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
bot_api_method! {
    method   = "chats/members/delete",
    request  = RequestChatsMembersDelete {
        required {
            chat_id: ChatId,
            user_id: UserId,
            members: Vec<Sn>,
        },
        optional {}
    },
    response = ResponseChatsMembersDelete {},
}
