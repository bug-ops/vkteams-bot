//! Set chat title method `chats/setTitle`
//! [More Info](https://teams.vk.com/botapi/#/chats/get_chats_setTitle)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/setTitle",
    request  = RequestChatsSetTitle {
        required {
            chat_id: ChatId,
            title: String,
        },
        optional {}
    },
    response = ResponseChatsSetTitle {},
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::ChatId;
    use serde_json::json;

    #[test]
    fn test_request_chats_set_title_serialize() {
        let req = RequestChatsSetTitle {
            chat_id: ChatId::from("c1"),
            title: "title text".to_string(),
        };
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["chatId"], "c1");
        assert_eq!(val["title"], "title text");
    }

    #[test]
    fn test_request_chats_set_title_deserialize() {
        let val = json!({"chatId": "c2", "title": "desc"});
        let req: RequestChatsSetTitle = serde_json::from_value(val).unwrap();
        assert_eq!(req.chat_id.0, "c2");
        assert_eq!(req.title, "desc");
    }

    #[test]
    fn test_request_chats_set_title_empty_title() {
        let val = json!({"chatId": "c3", "title": ""});
        let req: RequestChatsSetTitle = serde_json::from_value(val).unwrap();
        assert_eq!(req.title, "");
    }

    #[test]
    fn test_request_chats_set_title_missing_title() {
        let val = json!({"chatId": "c4"});
        let res: Result<RequestChatsSetTitle, _> = serde_json::from_value(val);
        assert!(res.is_err());
    }

    #[test]
    fn test_request_chats_set_title_invalid_chat_id() {
        let val = json!({"chatId": 123, "title": "desc"});
        let res: Result<RequestChatsSetTitle, _> = serde_json::from_value(val);
        assert!(res.is_err());
    }

    #[test]
    fn test_response_chats_set_title_serialize_deserialize() {
        let resp = ResponseChatsSetTitle {};
        let val = serde_json::to_value(&resp).unwrap();
        let resp2: ResponseChatsSetTitle = serde_json::from_value(val).unwrap();
        let _ = resp2;
    }
}
