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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_events_get_serialize() {
        let req = RequestEventsGet {
            last_event_id: 42,
            poll_time: Some(10),
        };
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["lastEventId"], 42);
        assert_eq!(val["pollTime"], 10);
    }

    #[test]
    fn test_request_events_get_deserialize() {
        let val = json!({"lastEventId": 43, "pollTime": 15});
        let req: RequestEventsGet = serde_json::from_value(val).unwrap();
        assert_eq!(req.last_event_id, 43);
        assert_eq!(req.poll_time, Some(15));
    }

    #[test]
    fn test_request_events_get_missing_required() {
        let val = json!({"pollTime": 10});
        let req = serde_json::from_value::<RequestEventsGet>(val);
        assert!(req.is_err());
    }

    #[test]
    fn test_request_events_get_invalid_types() {
        let val = json!({"lastEventId": "foo", "pollTime": "bar"});
        let req = serde_json::from_value::<RequestEventsGet>(val);
        assert!(req.is_err());
    }

    #[test]
    fn test_response_events_get_serialize() {
        let resp = ResponseEventsGet { events: vec![] };
        let val = serde_json::to_value(&resp).unwrap();
        assert!(val["events"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_response_events_get_deserialize() {
        let val = json!({"events": []});
        let resp: ResponseEventsGet = serde_json::from_value(val).unwrap();
        assert!(resp.events.is_empty());
    }

    #[test]
    fn test_response_events_get_missing_events() {
        let val = json!({});
        let resp = serde_json::from_value::<ResponseEventsGet>(val);
        assert!(resp.is_err());
    }

    #[test]
    fn test_response_events_get_invalid_event() {
        let val = json!({"events": [{"foo": "bar"}]});
        let resp = serde_json::from_value::<ResponseEventsGet>(val);
        assert!(resp.is_err());
    }
}
