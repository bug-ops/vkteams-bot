#![allow(unused_parens)]
//! # Get chat members method `chats/getMembers`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_getMembers)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/getMembers",
    request  = RequestChatsGetMembers {
        required {
            chat_id: ChatId,
        },
        optional {
            cursor: u32,
        }
    },
    response = ResponseChatsGetMembers {
        #[serde(default)]
        members: Vec<Member>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cursor: Option<u32>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{ChatId, Member, UserId};
    use serde_json::json;

    #[test]
    fn test_request_chats_get_members_serialize() {
        let req = RequestChatsGetMembers {
            chat_id: ChatId("c1".to_string()),
            cursor: Some(42),
        };
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["chatId"], "c1");
        assert_eq!(val["cursor"], 42);
    }

    #[test]
    fn test_request_chats_get_members_deserialize() {
        let val = json!({"chatId": "c2", "cursor": 7});
        let req: RequestChatsGetMembers = serde_json::from_value(val).unwrap();
        assert_eq!(req.chat_id.0, "c2");
        assert_eq!(req.cursor, Some(7));
    }

    #[test]
    fn test_request_chats_get_members_no_cursor() {
        let val = json!({"chatId": "c3"});
        let req: RequestChatsGetMembers = serde_json::from_value(val).unwrap();
        assert_eq!(req.cursor, None);
    }

    #[test]
    fn test_request_chats_get_members_invalid_chat_id() {
        let val = json!({"chatId": 123});
        let res: Result<RequestChatsGetMembers, _> = serde_json::from_value(val);
        assert!(res.is_err());
    }

    #[test]
    fn test_response_chats_get_members_serialize_deserialize() {
        let members = vec![Member {
            user_id: UserId("u1".to_string()),
            creator: Some(true),
            admin: Some(false),
        }];
        let resp = ResponseChatsGetMembers {
            members: members.clone(),
            cursor: Some(99),
        };
        let val = serde_json::to_value(&resp).unwrap();
        assert_eq!(val["cursor"], 99);
        let resp2: ResponseChatsGetMembers = serde_json::from_value(val).unwrap();
        assert_eq!(resp2.members.len(), 1);
        assert_eq!(resp2.cursor, Some(99));
    }

    #[test]
    fn test_response_chats_get_members_empty_members() {
        let val = json!({"members": [], "cursor": 1});
        let resp: ResponseChatsGetMembers = serde_json::from_value(val).unwrap();
        assert_eq!(resp.members.len(), 0);
        assert_eq!(resp.cursor, Some(1));
    }

    #[test]
    fn test_response_chats_get_members_missing_members() {
        let val = json!({"cursor": 2});
        let resp: ResponseChatsGetMembers = serde_json::from_value(val).unwrap();
        assert_eq!(resp.members.len(), 0); // default
        assert_eq!(resp.cursor, Some(2));
    }

    #[test]
    fn test_response_chats_get_members_no_cursor() {
        let val = json!({"members": [{"userId": "u2"}]});
        let resp: ResponseChatsGetMembers = serde_json::from_value(val).unwrap();
        assert_eq!(resp.cursor, None);
        assert_eq!(resp.members.len(), 1);
    }
}
