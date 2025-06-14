use vkteams_bot::prelude::ResponseMessagesDeleteMessages;

/// Integration test: deserializes ResponseMessagesDeleteMessages from a real JSON file and checks that deserialization succeeds.
#[test]
fn test_response_messages_delete_messages_from_real_json() {
    let path = std::path::Path::new("tests/responds/messages_delete_messages.json");
    let data = std::fs::read_to_string(path).expect("Failed to read messages_delete_messages.json");
    let _resp: ResponseMessagesDeleteMessages =
        serde_json::from_str(&data).expect("Failed to deserialize ResponseMessagesDeleteMessages");
}
