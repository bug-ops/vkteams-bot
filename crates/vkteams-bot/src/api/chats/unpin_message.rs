//! Unpin Message method `chats/unpinMessage`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_unpinMessage)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/unpinMessage",
    request  = RequestChatsUnpinMessage {
        required {
            chat_id: ChatId,
            msg_id: MsgId,
        },
        optional {}
    },
    response = ResponseChatsUnpinMessage {},
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{ChatId, MsgId};
    use serde_json::json;

    #[test]
    fn test_request_chats_unpin_message_serialize() {
        let req = RequestChatsUnpinMessage {
            chat_id: ChatId::from("c1"),
            msg_id: MsgId("m1".to_string()),
        };
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["chatId"], "c1");
        assert_eq!(val["msgId"], "m1");
    }

    #[test]
    fn test_request_chats_unpin_message_deserialize() {
        let val = json!({"chatId": "c2", "msgId": "m2"});
        let req: RequestChatsUnpinMessage = serde_json::from_value(val).unwrap();
        assert_eq!(req.chat_id.0, "c2");
        assert_eq!(req.msg_id.0, "m2");
    }

    #[test]
    fn test_response_chats_unpin_message_serialize() {
        let resp = ResponseChatsUnpinMessage {};
        let val = serde_json::to_value(&resp).unwrap();
        assert_eq!(val.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_response_chats_unpin_message_deserialize() {
        let val = json!({});
        let resp: ResponseChatsUnpinMessage = serde_json::from_value(val).unwrap();
        let _ = resp;
    }

    #[test]
    fn test_request_chats_unpin_message_missing_fields() {
        let val = json!({"chatId": "c1"});
        let req = serde_json::from_value::<RequestChatsUnpinMessage>(val);
        assert!(req.is_err());
    }

    #[test]
    fn test_request_chats_unpin_message_wrong_type() {
        let val = json!({"chatId": 123, "msgId": "m1"});
        let req = serde_json::from_value::<RequestChatsUnpinMessage>(val);
        assert!(req.is_err());
    }
}
