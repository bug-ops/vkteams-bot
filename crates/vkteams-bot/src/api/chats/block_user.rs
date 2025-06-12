//! Block User method `chats/blockUser`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_blockUser)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/blockUser",
    request  = RequestChatsBlockUser {
        required {
            chat_id: ChatId,
            user_id: UserId,
        },
        optional {
            del_last_messages: bool,
        }
    },
    response = ResponseChatsBlockUser {},
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{ChatId, UserId};
    use serde_json::json;

    #[test]
    fn test_request_chats_block_user_serialize() {
        let req = RequestChatsBlockUser {
            chat_id: ChatId("c1".to_string()),
            user_id: UserId("u1".to_string()),
            del_last_messages: Some(true),
        };
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["chatId"], "c1");
        assert_eq!(val["userId"], "u1");
        assert_eq!(val["delLastMessages"], true);
    }

    #[test]
    fn test_request_chats_block_user_deserialize() {
        let val = json!({"chatId": "c2", "userId": "u2", "delLastMessages": false});
        let req: RequestChatsBlockUser = serde_json::from_value(val).unwrap();
        assert_eq!(req.chat_id.0, "c2");
        assert_eq!(req.user_id.0, "u2");
        assert_eq!(req.del_last_messages, Some(false));
    }

    #[test]
    fn test_request_chats_block_user_missing_optional() {
        let val = json!({"chatId": "c3", "userId": "u3"});
        let req: RequestChatsBlockUser = serde_json::from_value(val).unwrap();
        assert_eq!(req.chat_id.0, "c3");
        assert_eq!(req.user_id.0, "u3");
        assert_eq!(req.del_last_messages, None);
    }

    #[test]
    fn test_request_chats_block_user_missing_required() {
        let val = json!({"chatId": "c4"});
        let req = serde_json::from_value::<RequestChatsBlockUser>(val);
        assert!(req.is_err());
    }

    #[test]
    fn test_request_chats_block_user_invalid_types() {
        let val = json!({"chatId": 123, "userId": true});
        let req = serde_json::from_value::<RequestChatsBlockUser>(val);
        assert!(req.is_err());
    }

    #[test]
    fn test_response_chats_block_user_serialize() {
        let resp = ResponseChatsBlockUser {};
        let val = serde_json::to_value(&resp).unwrap();
        assert_eq!(val.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_response_chats_block_user_deserialize() {
        let val = json!({});
        let resp: ResponseChatsBlockUser = serde_json::from_value(val).unwrap();
        let _ = resp;
    }
}
