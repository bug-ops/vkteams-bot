use vkteams_bot::prelude::ResponseMessagesSendFile;

/// Integration test: deserializes ResponseMessagesSendFile from a real JSON file and checks key fields.
#[test]
fn test_response_messages_send_file_from_real_json() {
    let path = std::path::Path::new("tests/responds/messages_send_file.json");
    let data = std::fs::read_to_string(path).expect("Failed to read messages_send_file.json");
    let resp: ResponseMessagesSendFile =
        serde_json::from_str(&data).expect("Failed to deserialize ResponseMessagesSendFile");
    assert_eq!(resp.msg_id.as_ref().unwrap().0, "m456");
    assert_eq!(resp.file_id.as_ref().unwrap(), "file_789");
}
