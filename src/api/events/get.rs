#![allow(unused_parens)]
//! Get the events that have occurred since the last event id method `events/get`
//! [More info](https://teams.vk.com/botapi/#/events/get_events_get)
use crate::api::types::*;
use serde::{Deserialize, Serialize};

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
