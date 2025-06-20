#![allow(unused_parens)]
//! Get blocked users method `chats/getBlockedUsers`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_getBlockedUsers)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/getBlockedUsers",
    request  = RequestChatsGetBlockedUsers {
        required {
            chat_id: ChatId,
        },
        optional {}
    },
    response = ResponseChatsGetBlockedUsers {
        users: Vec<Users>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{ChatId, UserId, Users};
    use serde_json::json;

    #[test]
    fn test_request_chats_get_blocked_users_serialize() {
        let req = RequestChatsGetBlockedUsers {
            chat_id: ChatId::from("c1"),
        };
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["chatId"], "c1");
    }

    #[test]
    fn test_request_chats_get_blocked_users_deserialize() {
        let val = json!({"chatId": "c2"});
        let req: RequestChatsGetBlockedUsers = serde_json::from_value(val).unwrap();
        assert_eq!(req.chat_id.0, "c2");
    }

    #[test]
    fn test_response_chats_get_blocked_users_serialize() {
        let resp = ResponseChatsGetBlockedUsers {
            users: vec![Users {
                user_id: UserId("u1".to_string()),
            }],
        };
        let val = serde_json::to_value(&resp).unwrap();
        assert_eq!(val["users"][0]["userId"], "u1");
    }

    #[test]
    fn test_response_chats_get_blocked_users_deserialize() {
        let val = json!({"users": [{"userId": "u2"}]});
        let resp: ResponseChatsGetBlockedUsers = serde_json::from_value(val).unwrap();
        assert_eq!(resp.users.len(), 1);
        assert_eq!(resp.users[0].user_id.0, "u2");
    }

    #[test]
    fn test_response_chats_get_blocked_users_empty_users() {
        let val = json!({"users": []});
        let resp: ResponseChatsGetBlockedUsers = serde_json::from_value(val).unwrap();
        assert!(resp.users.is_empty());
    }

    #[test]
    fn test_response_chats_get_blocked_users_missing_users() {
        let val = json!({});
        let resp = serde_json::from_value::<ResponseChatsGetBlockedUsers>(val);
        assert!(resp.is_err());
    }

    #[test]
    fn test_response_chats_get_blocked_users_user_missing_userid() {
        let val = json!({"users": [{}]});
        let resp = serde_json::from_value::<ResponseChatsGetBlockedUsers>(val);
        assert!(resp.is_err());
    }
}
