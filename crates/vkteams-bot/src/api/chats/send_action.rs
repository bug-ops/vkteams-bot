//! Send chat actions method `chats/sendActions`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_sendActions)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/sendActions",
    request  = RequestChatsSendAction {
        required {
            chat_id: ChatId,
            actions: ChatActions,
        },
        optional {}
    },
    response = ResponseChatsSendAction {},
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{ChatActions, ChatId};
    use serde_json::json;

    #[test]
    fn test_request_chats_send_action_serialize() {
        let req = RequestChatsSendAction {
            chat_id: ChatId::from("c1"),
            actions: ChatActions::Typing,
        };
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["chatId"], "c1");
        assert_eq!(val["actions"], "typing");
    }

    #[test]
    fn test_request_chats_send_action_deserialize() {
        let val = json!({"chatId": "c2", "actions": "looking"});
        let req: RequestChatsSendAction = serde_json::from_value(val).unwrap();
        assert_eq!(req.chat_id.0, "c2");
        assert_eq!(req.actions, ChatActions::Looking);
    }

    #[test]
    fn test_request_chats_send_action_missing_required() {
        let val = json!({"chatId": "c3"});
        let req = serde_json::from_value::<RequestChatsSendAction>(val);
        assert!(req.is_err());
    }

    #[test]
    fn test_request_chats_send_action_invalid_types() {
        let val = json!({"chatId": 123, "actions": 42});
        let req = serde_json::from_value::<RequestChatsSendAction>(val);
        assert!(req.is_err());
    }

    #[test]
    fn test_response_chats_send_action_serialize() {
        let resp = ResponseChatsSendAction {};
        let val = serde_json::to_value(&resp).unwrap();
        assert_eq!(val.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_response_chats_send_action_deserialize() {
        let val = json!({});
        let resp: ResponseChatsSendAction = serde_json::from_value(val).unwrap();
        let _ = resp;
    }
}
