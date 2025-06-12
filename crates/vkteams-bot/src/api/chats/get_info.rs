#![allow(unused_parens)]
//! Get information about a chat method `chats/getInfo`
//! [More info](https://teams.vk.com/botapi/#/chats/get_chats_getInfo)
use crate::api::types::*;
bot_api_method! {
    method   = "chats/getInfo",
    request  = RequestChatsGetInfo {
        required {
            chat_id: ChatId,
        },
        optional {}
    },
    response = ResponseChatsGetInfo {
        #[serde(flatten)]
        types: EnumChatsGetInfo
    },
}

/// # Chats get info response method `chats/getInfo`
/// Response can be one of the following types:
/// - `private`: [`ResponseChatsPrivateGetInfo`]
/// - `group`: [`ResponseChatsGroupGetInfo`]
/// - `channel`: [`ResponseChatsChannelGetInfo`]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EnumChatsGetInfo {
    /// Private chat
    Private(ResponseChatsPrivateGetInfo),
    /// Group chat
    Group(ResponseChatsGroupGetInfo),
    /// Channel chat
    Channel(ResponseChatsChannelGetInfo),
    #[default]
    None,
}
/// # Chats get info response method `chats/getInfo` for private chat
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsPrivateGetInfo {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub nick: Option<String>,
    pub about: Option<String>,
    pub is_bot: Option<bool>,
    pub language: Option<Languages>,
}
/// # Chats get info response method `chats/getInfo` for group chat
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsGroupGetInfo {
    pub title: Option<String>,
    pub about: Option<String>,
    pub rules: Option<String>,
    pub invite_link: Option<String>,
    pub public: Option<bool>,
    pub join_moderation: Option<bool>,
}
/// # Chats get info response method `chats/getInfo` for channel chat
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseChatsChannelGetInfo {
    pub title: Option<String>,
    pub about: Option<String>,
    pub rules: Option<String>,
    pub invite_link: Option<String>,
    pub public: Option<bool>,
    pub join_moderation: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_enum_chats_get_info_deserialize_private() {
        let val = json!({
            "type": "private",
            "firstName": "Ivan",
            "lastName": "Ivanov",
            "nick": "ivan123",
            "about": "about",
            "isBot": true,
            "language": "ru"
        });
        let info: EnumChatsGetInfo = serde_json::from_value(val).unwrap();
        match info {
            EnumChatsGetInfo::Private(p) => {
                assert_eq!(p.first_name.as_deref(), Some("Ivan"));
                assert_eq!(p.is_bot, Some(true));
            }
            _ => panic!("Expected Private variant"),
        }
    }

    #[test]
    fn test_enum_chats_get_info_deserialize_group() {
        let val = json!({
            "type": "group",
            "title": "Group chat",
            "about": "desc",
            "rules": "rules",
            "inviteLink": "link",
            "public": true,
            "joinModeration": false
        });
        let info: EnumChatsGetInfo = serde_json::from_value(val).unwrap();
        match info {
            EnumChatsGetInfo::Group(g) => {
                assert_eq!(g.title.as_deref(), Some("Group chat"));
                assert_eq!(g.public, Some(true));
            }
            _ => panic!("Expected Group variant"),
        }
    }

    #[test]
    fn test_enum_chats_get_info_deserialize_channel() {
        let val = json!({
            "type": "channel",
            "title": "Channel chat"
        });
        let info: EnumChatsGetInfo = serde_json::from_value(val).unwrap();
        match info {
            EnumChatsGetInfo::Channel(c) => {
                assert_eq!(c.title.as_deref(), Some("Channel chat"));
            }
            _ => panic!("Expected Channel variant"),
        }
    }

    #[test]
    fn test_enum_chats_get_info_deserialize_none() {
        let val = json!({});
        let info: Result<EnumChatsGetInfo, _> = serde_json::from_value(val);
        assert!(info.is_err(), "Ожидалась ошибка при отсутствии поля 'type'");
    }

    #[test]
    fn test_private_missing_fields() {
        let val = json!({"type": "private"});
        let info: EnumChatsGetInfo = serde_json::from_value(val).unwrap();
        match info {
            EnumChatsGetInfo::Private(p) => {
                assert!(p.first_name.is_none());
                assert!(p.is_bot.is_none());
            }
            _ => panic!("Expected Private variant"),
        }
    }

    #[test]
    fn test_group_invalid_type() {
        let val = json!({"type": "group", "public": "not_bool"});
        let res: Result<EnumChatsGetInfo, _> = serde_json::from_value(val);
        assert!(res.is_err());
    }

    #[test]
    fn test_channel_empty() {
        let val = json!({"type": "channel"});
        let info: EnumChatsGetInfo = serde_json::from_value(val).unwrap();
        match info {
            EnumChatsGetInfo::Channel(c) => {
                assert!(c.title.is_none());
            }
            _ => panic!("Expected Channel variant"),
        }
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let group = ResponseChatsGroupGetInfo {
            title: Some("t".to_string()),
            about: None,
            rules: None,
            invite_link: None,
            public: Some(false),
            join_moderation: None,
        };
        let enum_val = EnumChatsGetInfo::Group(group);
        let ser = serde_json::to_string(&enum_val).unwrap();
        let de: EnumChatsGetInfo = serde_json::from_str(&ser).unwrap();
        match de {
            EnumChatsGetInfo::Group(g) => {
                assert_eq!(g.title.as_deref(), Some("t"));
                assert_eq!(g.public, Some(false));
            }
            _ => panic!("Expected Group variant"),
        }
    }
}
