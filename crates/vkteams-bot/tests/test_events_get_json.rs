use vkteams_bot::prelude::{EventType, ResponseEventsGet};

/// Integration test: deserializes ResponseEventsGet from a real JSON file and checks key fields.
#[test]
fn test_response_events_get_from_real_json() {
    let path = std::path::Path::new("tests/responds/events_get.json");
    let data = std::fs::read_to_string(path).expect("Failed to read events_get.json");
    let resp: ResponseEventsGet =
        serde_json::from_str(&data).expect("Failed to deserialize ResponseEventsGet");
    assert_eq!(resp.events.len(), 1);
    assert_eq!(resp.events[0].event_id, 1);
    // Check event type and payload fields strictly
    match &resp.events[0].event_type {
        EventType::NewMessage(payload) => {
            assert_eq!(payload.msg_id.0, "m1");
            assert_eq!(payload.text, "Hello!");
            assert_eq!(payload.chat.chat_id.0, "c1");
            assert_eq!(payload.from.first_name, "Bot");
        }
        _ => panic!("Expected NewMessage event type"),
    }
}
