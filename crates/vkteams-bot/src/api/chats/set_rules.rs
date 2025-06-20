//! Set rules for a chat method `chats/setRules`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_setRules)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/setRules",
    request  = RequestChatsSetRules {
        required {
            chat_id: ChatId,
            rules: String,
        },
        optional {}
    },
    response = ResponseChatsSetRules {},
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::ChatId;
    use serde_json::json;

    #[test]
    fn test_request_chats_set_rules_serialize() {
        let req = RequestChatsSetRules {
            chat_id: ChatId::from("c1"),
            rules: "rules text".to_string(),
        };
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["chatId"], "c1");
        assert_eq!(val["rules"], "rules text");
    }

    #[test]
    fn test_request_chats_set_rules_deserialize() {
        let val = json!({"chatId": "c2", "rules": "desc"});
        let req: RequestChatsSetRules = serde_json::from_value(val).unwrap();
        assert_eq!(req.chat_id.0, "c2");
        assert_eq!(req.rules, "desc");
    }

    #[test]
    fn test_request_chats_set_rules_empty_rules() {
        let val = json!({"chatId": "c3", "rules": ""});
        let req: RequestChatsSetRules = serde_json::from_value(val).unwrap();
        assert_eq!(req.rules, "");
    }

    #[test]
    fn test_request_chats_set_rules_missing_rules() {
        let val = json!({"chatId": "c4"});
        let res: Result<RequestChatsSetRules, _> = serde_json::from_value(val);
        assert!(res.is_err());
    }

    #[test]
    fn test_request_chats_set_rules_invalid_chat_id() {
        let val = json!({"chatId": 123, "rules": "desc"});
        let res: Result<RequestChatsSetRules, _> = serde_json::from_value(val);
        assert!(res.is_err());
    }

    #[test]
    fn test_response_chats_set_rules_serialize_deserialize() {
        let resp = ResponseChatsSetRules {};
        let val = serde_json::to_value(&resp).unwrap();
        let resp2: ResponseChatsSetRules = serde_json::from_value(val).unwrap();
        let _ = resp2;
    }
}
