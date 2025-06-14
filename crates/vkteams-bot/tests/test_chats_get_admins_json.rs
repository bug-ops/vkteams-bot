use vkteams_bot::prelude::ResponseChatsGetAdmins;

/// Integration test: deserializes ResponseChatsGetAdmins from a real JSON file and checks key fields.
#[test]
fn test_response_chats_get_admins_from_real_json() {
    let path = std::path::Path::new("tests/responds/chats_get_admins.json");
    let data = std::fs::read_to_string(path).expect("Failed to read chats_get_admins.json");
    let resp: ResponseChatsGetAdmins =
        serde_json::from_str(&data).expect("Failed to deserialize ResponseChatsGetAdmins");
    assert_eq!(resp.admins.len(), 2);
    assert_eq!(resp.admins[0].user_id.0, "u1");
    assert_eq!(resp.admins[0].creator, Some(true));
    assert_eq!(resp.admins[1].user_id.0, "u2");
    assert_eq!(resp.admins[1].creator, Some(false));
}
