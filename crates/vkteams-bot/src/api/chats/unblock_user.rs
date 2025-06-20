//! Unblock User in chat method `chats/unblockUser`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_unblockUser)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/unblockUser",
    request  = RequestChatsUnblockUser {
        required {
            chat_id: ChatId,
            user_id: UserId,
        },
        optional {}
    },
    response = ResponseChatsUnblockUser {},
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{ChatId, UserId};
    use serde_json::json;

    #[test]
    fn test_request_chats_unblock_user_serialize() {
        let req = RequestChatsUnblockUser {
            chat_id: ChatId::from("c1"),
            user_id: UserId("u1".to_string()),
        };
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["chatId"], "c1");
        assert_eq!(val["userId"], "u1");
    }

    #[test]
    fn test_request_chats_unblock_user_deserialize() {
        let val = json!({"chatId": "c2", "userId": "u2"});
        let req: RequestChatsUnblockUser = serde_json::from_value(val).unwrap();
        assert_eq!(req.chat_id.0, "c2");
        assert_eq!(req.user_id.0, "u2");
    }

    #[test]
    fn test_request_chats_unblock_user_missing_required() {
        let val = json!({"chatId": "c3"});
        let req = serde_json::from_value::<RequestChatsUnblockUser>(val);
        assert!(req.is_err());
    }

    #[test]
    fn test_request_chats_unblock_user_invalid_types() {
        let val = json!({"chatId": 123, "userId": true});
        let req = serde_json::from_value::<RequestChatsUnblockUser>(val);
        assert!(req.is_err());
    }

    #[test]
    fn test_response_chats_unblock_user_serialize() {
        let resp = ResponseChatsUnblockUser {};
        let val = serde_json::to_value(&resp).unwrap();
        assert_eq!(val.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_response_chats_unblock_user_deserialize() {
        let val = json!({});
        let resp: ResponseChatsUnblockUser = serde_json::from_value(val).unwrap();
        let _ = resp;
    }
}
