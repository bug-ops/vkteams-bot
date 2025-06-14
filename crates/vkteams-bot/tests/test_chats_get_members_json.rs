use vkteams_bot::prelude::ResponseChatsGetMembers;

/// Integration test: deserializes ResponseChatsGetMembers from a real JSON file and checks key fields.
#[test]
fn test_response_chats_get_members_from_real_json() {
    let path = std::path::Path::new("tests/responds/chats_get_members.json");
    let data = std::fs::read_to_string(path).expect("Failed to read chats_get_members.json");
    let resp: ResponseChatsGetMembers =
        serde_json::from_str(&data).expect("Failed to deserialize ResponseChatsGetMembers");
    assert_eq!(resp.members.len(), 2);
    assert_eq!(resp.members[0].user_id.0, "u1");
    assert_eq!(resp.members[0].creator, Some(true));
    assert_eq!(resp.members[0].admin, Some(false));
    assert_eq!(resp.members[1].user_id.0, "u2");
    assert_eq!(resp.members[1].creator, Some(false));
    assert_eq!(resp.members[1].admin, Some(true));
    assert_eq!(resp.cursor, Some(123));
}
