//! Avatar set method `chats/avatar/set`
//! [More info](https://teams.vk.com/botapi/#/chats/post_chats_avatar_set)
use crate::api::types::*;
bot_api_method! {
    method      = "chats/avatar/set",
    http_method = HTTPMethod::POST,
    request     = RequestChatsAvatarSet {
        required {
            chat_id: ChatId,
            multipart: MultipartName,
        },
        optional {}
    },
    response = ResponseChatsAvatarSet {},
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{ChatId, MultipartName};
    use serde_json::json;

    #[test]
    fn test_request_chats_avatar_set_serialize() {
        let req = RequestChatsAvatarSet {
            chat_id: ChatId("c1".to_string()),
            multipart: MultipartName::FilePath("file.png".to_string()),
        };
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["chatId"], "c1");
        // multipart сериализуется как строка "file" (Display), либо как объект, если derive
    }

    #[test]
    fn test_request_chats_avatar_set_deserialize() {
        let val = json!({"chatId": "c2", "multipart": {"FilePath": "file.png"}});
        let req: RequestChatsAvatarSet = serde_json::from_value(val).unwrap();
        assert_eq!(req.chat_id.0, "c2");
        match req.multipart {
            MultipartName::FilePath(ref s) => assert_eq!(s, "file.png"),
            _ => panic!("Expected MultipartName::FilePath"),
        }
    }

    #[test]
    fn test_response_chats_avatar_set_serialize() {
        let resp = ResponseChatsAvatarSet {};
        let val = serde_json::to_value(&resp).unwrap();
        assert_eq!(val.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_response_chats_avatar_set_deserialize() {
        let val = json!({});
        let resp: ResponseChatsAvatarSet = serde_json::from_value(val).unwrap();
        let _ = resp;
    }

    #[test]
    fn test_request_chats_avatar_set_missing_fields() {
        let val = json!({"chatId": "c1"});
        let req = serde_json::from_value::<RequestChatsAvatarSet>(val);
        assert!(req.is_err());
    }

    #[test]
    fn test_request_chats_avatar_set_wrong_type() {
        let val = json!({"chatId": 123, "multipart": "file"});
        let req = serde_json::from_value::<RequestChatsAvatarSet>(val);
        assert!(req.is_err());
    }
}
