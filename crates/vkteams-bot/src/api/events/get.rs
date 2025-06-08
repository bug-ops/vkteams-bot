#![allow(unused_parens)]
//! Get the events that have occurred since the last event id method `events/get`
//! [More info](https://teams.vk.com/botapi/#/events/get_events_get)
use crate::api::types::*;
bot_api_method! {
    method = "events/get",
    request = RequestEventsGet {
        required {
            last_event_id: EventId,
        },
        optional {
            poll_time: u64,
        }
    },
    response = ResponseEventsGet {
        events: Vec<EventMessage>,
    },
}

#[cfg(test)]
use crate::prelude::*;
#[test]
fn test_chats_events_get_deserialization() {
    let j = std::fs::read_to_string("tests/chats_events_get.json").unwrap();
    // Test deserialization of ResponseEventsGet
    let _ = serde_json::from_str::<ResponseEventsGet>(j.as_str()).map_err(|e| {
        eprintln!("Error deserializing response: {}", e);
        assert!(false);
    });
    // Test deserialization of ApiResponseWrapper<ResponseEventsGet>
    let _ =
        serde_json::from_str::<ApiResponseWrapper<ResponseEventsGet>>(j.as_str()).map_err(|e| {
            eprintln!("Error deserializing response: {}", e);
            assert!(false);
        });
}
