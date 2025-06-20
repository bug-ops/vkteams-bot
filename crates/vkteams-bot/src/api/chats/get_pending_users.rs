#![allow(unused_parens)]
//! Get pending users method `chats/getPendingUsers`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_getPendingUsers)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/getPendingUsers",
    request  = RequestChatsGetPendingUsers {
        required {
            chat_id: ChatId,
        },
        optional {}
    },
    response = ResponseChatsGetPendingUsers {
        users: Vec<Users>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{ChatId, UserId, Users};
    use serde_json::json;

    #[test]
    fn test_request_chats_get_pending_users_serialize() {
        let req = RequestChatsGetPendingUsers {
            chat_id: ChatId::from("c1"),
        };
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["chatId"], "c1");
    }

    #[test]
    fn test_request_chats_get_pending_users_deserialize() {
        let val = json!({"chatId": "c2"});
        let req: RequestChatsGetPendingUsers = serde_json::from_value(val).unwrap();
        assert_eq!(req.chat_id.0, "c2");
    }

    #[test]
    fn test_response_chats_get_pending_users_serialize() {
        let resp = ResponseChatsGetPendingUsers {
            users: vec![Users {
                user_id: UserId("u1".to_string()),
            }],
        };
        let val = serde_json::to_value(&resp).unwrap();
        assert_eq!(val["users"][0]["userId"], "u1");
    }

    #[test]
    fn test_response_chats_get_pending_users_deserialize() {
        let val = json!({"users": [{"userId": "u2"}]});
        let resp: ResponseChatsGetPendingUsers = serde_json::from_value(val).unwrap();
        assert_eq!(resp.users.len(), 1);
        assert_eq!(resp.users[0].user_id.0, "u2");
    }

    #[test]
    fn test_response_chats_get_pending_users_empty() {
        let val = json!({"users": []});
        let resp: ResponseChatsGetPendingUsers = serde_json::from_value(val).unwrap();
        assert!(resp.users.is_empty());
    }

    #[test]
    fn test_response_chats_get_pending_users_missing_users() {
        let val = json!({});
        let resp = serde_json::from_value::<ResponseChatsGetPendingUsers>(val);
        assert!(resp.is_err());
    }

    #[test]
    fn test_response_chats_get_pending_users_invalid_user() {
        let val = json!({"users": [{"foo": "bar"}]});
        let resp = serde_json::from_value::<ResponseChatsGetPendingUsers>(val);
        assert!(resp.is_err());
    }
}
