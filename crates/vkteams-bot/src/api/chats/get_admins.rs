#![allow(unused_parens)]
//! Get chat admins method `chats/getAdmins`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_getAdmins)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/getAdmins",
    request  = RequestChatsGetAdmins {
        required {
            chat_id: ChatId,
        },
        optional {}
    },
    response = ResponseChatsGetAdmins {
        admins: Vec<Admin>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{Admin, ChatId, UserId};
    use serde_json::json;

    #[test]
    fn test_request_chats_get_admins_serialize() {
        let req = RequestChatsGetAdmins {
            chat_id: ChatId("c1".to_string()),
        };
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["chatId"], "c1");
    }

    #[test]
    fn test_request_chats_get_admins_deserialize() {
        let val = json!({"chatId": "c2"});
        let req: RequestChatsGetAdmins = serde_json::from_value(val).unwrap();
        assert_eq!(req.chat_id.0, "c2");
    }

    #[test]
    fn test_response_chats_get_admins_serialize() {
        let resp = ResponseChatsGetAdmins {
            admins: vec![Admin {
                user_id: UserId("u1".to_string()),
                creator: Some(true),
            }],
        };
        let val = serde_json::to_value(&resp).unwrap();
        assert_eq!(val["admins"][0]["userId"], "u1");
        assert_eq!(val["admins"][0]["creator"], true);
    }

    #[test]
    fn test_response_chats_get_admins_deserialize() {
        let val = json!({"admins": [{"userId": "u2", "creator": false}]});
        let resp: ResponseChatsGetAdmins = serde_json::from_value(val).unwrap();
        assert_eq!(resp.admins.len(), 1);
        assert_eq!(resp.admins[0].user_id.0, "u2");
        assert_eq!(resp.admins[0].creator, Some(false));
    }

    #[test]
    fn test_response_chats_get_admins_empty_admins() {
        let val = json!({"admins": []});
        let resp: ResponseChatsGetAdmins = serde_json::from_value(val).unwrap();
        assert!(resp.admins.is_empty());
    }

    #[test]
    fn test_response_chats_get_admins_missing_admins() {
        let val = json!({});
        let resp = serde_json::from_value::<ResponseChatsGetAdmins>(val);
        assert!(resp.is_err());
    }

    #[test]
    fn test_response_chats_get_admins_admin_missing_userid() {
        let val = json!({"admins": [{"creator": true}]});
        let resp = serde_json::from_value::<ResponseChatsGetAdmins>(val);
        assert!(resp.is_err());
    }

    #[test]
    fn test_response_chats_get_admins_admin_creator_none() {
        let val = json!({"admins": [{"userId": "u3"}]});
        let resp: ResponseChatsGetAdmins = serde_json::from_value(val).unwrap();
        assert_eq!(resp.admins[0].user_id.0, "u3");
        assert_eq!(resp.admins[0].creator, None);
    }
}
