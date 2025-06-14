use vkteams_bot::prelude::ResponseMessagesEditText;

/// Integration test: deserializes ResponseMessagesEditText from a real JSON file and checks that deserialization succeeds.
#[test]
fn test_response_messages_edit_text_from_real_json() {
    let path = std::path::Path::new("tests/responds/messages_edit_text.json");
    let data = std::fs::read_to_string(path).expect("Failed to read messages_edit_text.json");
    let _resp: ResponseMessagesEditText =
        serde_json::from_str(&data).expect("Failed to deserialize ResponseMessagesEditText");
}
