use vkteams_bot::prelude::ResponseSelfGet;

#[test]
fn test_response_self_get_from_real_json() {
    let path = std::path::Path::new("tests/responds/chats_self_get.json");
    let data = std::fs::read_to_string(path).expect("Failed to read chats_self_get.json");
    let resp: ResponseSelfGet =
        serde_json::from_str(&data).expect("Failed to deserialize ResponseSelfGet");
    assert_eq!(resp.user_id.0, "1000000001");
    assert_eq!(resp.nick, "testbot");
    assert_eq!(resp.first_name, "testbot");
    // Fields about and photo may be None/absent if optional
}
