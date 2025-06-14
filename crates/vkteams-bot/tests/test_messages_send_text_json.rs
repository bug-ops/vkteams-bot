use vkteams_bot::prelude::ResponseMessagesSendText;

/// Integration test: deserializes ResponseMessagesSendText from a real JSON file and checks key fields.
#[test]
fn test_response_messages_send_text_from_real_json() {
    let path = std::path::Path::new("tests/responds/messages_send_text.json");
    let data = std::fs::read_to_string(path).expect("Failed to read messages_send_text.json");
    let resp: ResponseMessagesSendText =
        serde_json::from_str(&data).expect("Failed to deserialize ResponseMessagesSendText");
    assert_eq!(resp.msg_id.0, "m123");
}
