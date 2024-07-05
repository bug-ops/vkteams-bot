//! Get the events that have occurred since the last event id method `events/get`
//! [More info](https://teams.vk.com/botapi/#/events/get_events_get)
use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// # Get the events that have occurred since the last event id request method `events/get`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestEventsGet {
    pub last_event_id: u32,
    pub poll_time: u64,
}
/// # Get the events that have occurred since the last event id response method `events/get`
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ResponseEventsGet {
    pub events: Vec<EventMessage>,
    pub ok: bool,
}
impl BotRequest for RequestEventsGet {
    const METHOD: &'static str = "events/get";
    type RequestType = Self;
    type ResponseType = ResponseEventsGet;
}
impl RequestEventsGet {
    /// Create a new [`RequestEventsGet`]
    /// ## Parameters
    /// - `last_event_id` - [`EventId`]
    pub fn new(last_event_id: EventId) -> Self {
        Self {
            last_event_id,
            poll_time: POLL_TIME,
        }
    }
}
