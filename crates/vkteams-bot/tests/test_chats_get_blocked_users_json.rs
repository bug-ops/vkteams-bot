use vkteams_bot::prelude::ResponseChatsGetBlockedUsers;

/// Integration test: deserializes ResponseChatsGetBlockedUsers from a real JSON file and checks key fields.
#[test]
fn test_response_chats_get_blocked_users_from_real_json() {
    let path = std::path::Path::new("tests/responds/chats_get_blocked_users.json");
    let data = std::fs::read_to_string(path).expect("Failed to read chats_get_blocked_users.json");
    let resp: ResponseChatsGetBlockedUsers =
        serde_json::from_str(&data).expect("Failed to deserialize ResponseChatsGetBlockedUsers");
    assert_eq!(resp.users.len(), 2);
    assert_eq!(resp.users[0].user_id.0, "u1");
    assert_eq!(resp.users[1].user_id.0, "u2");
}
