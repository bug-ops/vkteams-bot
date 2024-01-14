use crate::api::types::*;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::EventsGet`]
///
/// [`SendMessagesAPIMethods::EventsGet`]: enum.SendMessagesAPIMethods.html#variant.EventsGet
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestEventsGet {
    pub last_event_id: u64,
    pub poll_time: u64,
}
/// Response for method [`SendMessagesAPIMethods::EventsGet`]
///
/// [`SendMessagesAPIMethods::EventsGet`]: enum.SendMessagesAPIMethods.html#variant.EventsGet
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ResponseEventsGet {
    pub events: Vec<EventMessage>,
    pub ok: bool,
}
impl BotRequest for RequestEventsGet {
    const METHOD: &'static str = "events/get";
    type RequestType = Self;
    type ResponseType = ResponseEventsGet;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::EventsGet(last_event_id) => Self {
                last_event_id: *last_event_id,
                poll_time: POLL_TIME,
            },
            _ => panic!("Wrong API method for RequestEventsGet"),
        }
    }
}
