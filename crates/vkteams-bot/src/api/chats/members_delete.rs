//! Delete members from the chat method `chats/members/delete`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_members_delete)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/members/delete",
    request  = RequestChatsMembersDelete {
        required {
            chat_id: ChatId,
            user_id: UserId,
            members: Vec<Sn>,
        },
        optional {}
    },
    response = ResponseChatsMembersDelete {},
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{ChatId, Sn, UserId};
    use serde_json::json;

    #[test]
    fn test_request_chats_members_delete_serialize() {
        let req = RequestChatsMembersDelete {
            chat_id: ChatId::from("c1"),
            user_id: UserId("u1".to_string()),
            members: vec![Sn {
                sn: "s1".to_string(),
                user_id: UserId("u2".to_string()),
            }],
        };
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["chatId"], "c1");
        assert_eq!(val["userId"], "u1");
        assert_eq!(val["members"][0]["sn"], "s1");
        assert_eq!(val["members"][0]["userId"], "u2");
    }

    #[test]
    fn test_request_chats_members_delete_deserialize() {
        let val =
            json!({"chatId": "c2", "userId": "u3", "members": [{"sn": "s2", "userId": "u4"}]});
        let req: RequestChatsMembersDelete = serde_json::from_value(val).unwrap();
        assert_eq!(req.chat_id.0, "c2");
        assert_eq!(req.user_id.0, "u3");
        assert_eq!(req.members[0].sn, "s2");
        assert_eq!(req.members[0].user_id.0, "u4");
    }

    #[test]
    fn test_request_chats_members_delete_empty_members() {
        let val = json!({"chatId": "c3", "userId": "u5", "members": []});
        let req: RequestChatsMembersDelete = serde_json::from_value(val).unwrap();
        assert!(req.members.is_empty());
    }

    #[test]
    fn test_request_chats_members_delete_missing_members() {
        let val = json!({"chatId": "c4", "userId": "u6"});
        let req = serde_json::from_value::<RequestChatsMembersDelete>(val);
        assert!(req.is_err());
    }

    #[test]
    fn test_request_chats_members_delete_invalid_member() {
        let val = json!({"chatId": "c5", "userId": "u7", "members": [{"foo": "bar"}]});
        let req = serde_json::from_value::<RequestChatsMembersDelete>(val);
        assert!(req.is_err());
    }

    #[test]
    fn test_response_chats_members_delete_serialize() {
        let resp = ResponseChatsMembersDelete {};
        let val = serde_json::to_value(&resp).unwrap();
        assert_eq!(val.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_response_chats_members_delete_deserialize() {
        let val = json!({});
        let resp: ResponseChatsMembersDelete = serde_json::from_value(val).unwrap();
        let _ = resp;
    }
}
