//! Set a chat's about text method `chats/setAbout`
//! [Nore info](https://teams.vk.com/botapi/#/chats/get_chats_setAbout)
use crate::api::types::*;
bot_api_method! {
    method = "chats/setAbout",
    request = RequestChatsSetAbout {
        required {
            chat_id: ChatId,
            about: String,
        },
        optional {}
    },
    response = ResponseChatsSetAbout {},
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::ChatId;
    use serde_json::json;

    #[test]
    fn test_request_chats_set_about_serialize() {
        let req = RequestChatsSetAbout {
            chat_id: ChatId("c1".to_string()),
            about: "about text".to_string(),
        };
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["chatId"], "c1");
        assert_eq!(val["about"], "about text");
    }

    #[test]
    fn test_request_chats_set_about_deserialize() {
        let val = json!({"chatId": "c2", "about": "desc"});
        let req: RequestChatsSetAbout = serde_json::from_value(val).unwrap();
        assert_eq!(req.chat_id.0, "c2");
        assert_eq!(req.about, "desc");
    }

    #[test]
    fn test_request_chats_set_about_empty_about() {
        let val = json!({"chatId": "c3", "about": ""});
        let req: RequestChatsSetAbout = serde_json::from_value(val).unwrap();
        assert_eq!(req.about, "");
    }

    #[test]
    fn test_request_chats_set_about_missing_about() {
        let val = json!({"chatId": "c4"});
        let res: Result<RequestChatsSetAbout, _> = serde_json::from_value(val);
        assert!(res.is_err());
    }

    #[test]
    fn test_request_chats_set_about_invalid_chat_id() {
        let val = json!({"chatId": 123, "about": "desc"});
        let res: Result<RequestChatsSetAbout, _> = serde_json::from_value(val);
        assert!(res.is_err());
    }

    #[test]
    fn test_response_chats_set_about_serialize_deserialize() {
        let resp = ResponseChatsSetAbout {};
        let val = serde_json::to_value(&resp).unwrap();
        let resp2: ResponseChatsSetAbout = serde_json::from_value(val).unwrap();
        let _ = resp2;
    }
}
