use vkteams_bot::prelude::ResponseMessagesSendVoice;

/// Integration test: deserializes ResponseMessagesSendVoice from a real JSON file and checks key fields.
#[test]
fn test_response_messages_send_voice_from_real_json() {
    let path = std::path::Path::new("tests/responds/messages_send_voice.json");
    let data = std::fs::read_to_string(path).expect("Failed to read messages_send_voice.json");
    let resp: ResponseMessagesSendVoice =
        serde_json::from_str(&data).expect("Failed to deserialize ResponseMessagesSendVoice");
    assert_eq!(resp.msg_id.as_ref().unwrap().0, "m789");
    assert_eq!(resp.file_id.as_ref().unwrap(), "voice_123");
}
