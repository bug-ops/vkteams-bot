//! Get information about the current user method `self/get`
//! [More info](https://teams.vk.com/botapi/#/self/get_self_get)
use crate::api::types::*;
bot_api_method! {
    method = "self/get",
    request = RequestSelfGet {
        required {},
        optional {}
    },
    response = ResponseSelfGet {
        user_id: UserId,
        nick: String,
        first_name: String,
        about: Option<String>,
        photo: Option<Vec<PhotoUrl>>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{PhotoUrl, UserId};
    use serde_json::json;

    #[test]
    fn test_request_self_get_serialize() {
        let req = RequestSelfGet::default();
        let val = serde_json::to_value(&req).unwrap();
        // Ожидаем пустой объект
        assert_eq!(val, json!({}));
    }

    #[test]
    fn test_request_self_get_deserialize() {
        let val = json!({});
        let req: RequestSelfGet = serde_json::from_value(val).unwrap();
        let _ = req;
    }

    #[test]
    fn test_response_self_get_serialize() {
        let resp = ResponseSelfGet {
            user_id: UserId("u123".to_string()),
            nick: "nick123".to_string(),
            first_name: "Ivan".to_string(),
            about: Some("About me".to_string()),
            photo: Some(vec![PhotoUrl {
                url: "https://example.com/photo.jpg".to_string(),
            }]),
        };
        let val = serde_json::to_value(&resp).unwrap();
        assert_eq!(val["userId"], "u123");
        assert_eq!(val["nick"], "nick123");
        assert_eq!(val["firstName"], "Ivan");
        assert_eq!(val["about"], "About me");
        assert_eq!(val["photo"][0]["url"], "https://example.com/photo.jpg");
    }

    #[test]
    fn test_response_self_get_deserialize() {
        let val = json!({
            "userId": "u456",
            "nick": "nick456",
            "firstName": "Petr",
            "about": "Test about",
            "photo": [
                { "url": "https://example.com/pic.png" }
            ]
        });
        let resp: ResponseSelfGet = serde_json::from_value(val).unwrap();
        assert_eq!(resp.user_id.0, "u456");
        assert_eq!(resp.nick, "nick456");
        assert_eq!(resp.first_name, "Petr");
        assert_eq!(resp.about, Some("Test about".to_string()));
        assert_eq!(resp.photo.as_ref().unwrap().len(), 1);
        assert_eq!(
            resp.photo.as_ref().unwrap()[0].url,
            "https://example.com/pic.png"
        );
    }

    #[test]
    fn test_request_self_get_default() {
        let req = RequestSelfGet::default();
        let _ = req;
    }
}
