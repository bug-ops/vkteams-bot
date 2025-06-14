use vkteams_bot::prelude::ResponseChatsGetInfo;

/// Integration test: deserializes ResponseChatsGetInfo from a real JSON file and checks key fields.
#[test]
fn test_response_chats_get_info_from_real_json() {
    let path = std::path::Path::new("tests/responds/chats_get_info.json");
    let data = std::fs::read_to_string(path).expect("Failed to read chats_get_info.json");
    let resp: ResponseChatsGetInfo =
        serde_json::from_str(&data).expect("Failed to deserialize ResponseChatsGetInfo");
    // Check that the type is Private and fields are correct (fields are available via .types)
    // Since EnumChatsGetInfo is not public, just check that first_name is present and correct using Debug output
    let debug_str = format!("{:?}", resp.types);
    assert!(
        debug_str.contains("Ivan"),
        "Expected first_name 'Ivan' in the response"
    );
}
