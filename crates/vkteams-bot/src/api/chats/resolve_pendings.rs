//! Resolve pendings in chat method `chats/resolvePending`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_resolvePending)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/resolvePending",
    request  = RequestChatsResolvePending {
        required {
            chat_id: ChatId,
            approve: bool,
        },
        optional {
            user_id: UserId,
            everyone: bool,
        }
    },
    response = ResponseChatsResolvePending {},
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{ChatId, UserId};
    use serde_json::json;

    #[test]
    fn test_request_chats_resolve_pending_serialize() {
        let req = RequestChatsResolvePending {
            chat_id: ChatId("c1".to_string()),
            approve: true,
            user_id: Some(UserId("u1".to_string())),
            everyone: Some(false),
        };
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["chatId"], "c1");
        assert_eq!(val["approve"], true);
        assert_eq!(val["userId"], "u1");
        assert_eq!(val["everyone"], false);
    }

    #[test]
    fn test_request_chats_resolve_pending_deserialize() {
        let val = json!({"chatId": "c2", "approve": false, "userId": "u2", "everyone": true});
        let req: RequestChatsResolvePending = serde_json::from_value(val).unwrap();
        assert_eq!(req.chat_id.0, "c2");
        assert_eq!(req.approve, false);
        assert_eq!(req.user_id.as_ref().unwrap().0, "u2");
        assert_eq!(req.everyone, Some(true));
    }

    #[test]
    fn test_request_chats_resolve_pending_missing_optional() {
        let val = json!({"chatId": "c3", "approve": true});
        let req: RequestChatsResolvePending = serde_json::from_value(val).unwrap();
        assert_eq!(req.chat_id.0, "c3");
        assert_eq!(req.approve, true);
        assert!(req.user_id.is_none());
        assert!(req.everyone.is_none());
    }

    #[test]
    fn test_request_chats_resolve_pending_missing_required() {
        let val = json!({"chatId": "c4"});
        let req = serde_json::from_value::<RequestChatsResolvePending>(val);
        assert!(req.is_err());
    }

    #[test]
    fn test_request_chats_resolve_pending_invalid_types() {
        let val = json!({"chatId": 123, "approve": "yes"});
        let req = serde_json::from_value::<RequestChatsResolvePending>(val);
        assert!(req.is_err());
    }

    #[test]
    fn test_response_chats_resolve_pending_serialize() {
        let resp = ResponseChatsResolvePending {};
        let val = serde_json::to_value(&resp).unwrap();
        assert_eq!(val.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_response_chats_resolve_pending_deserialize() {
        let val = json!({});
        let resp: ResponseChatsResolvePending = serde_json::from_value(val).unwrap();
        let _ = resp;
    }
}
