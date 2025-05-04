use crate::error::Result;
use crate::prelude::*;
use std::future::Future;
/// Listen for events and execute callback function
/// ## Parameters
/// - `func` - callback function with [`Result`] type [`ResponseEventsGet`] as argument
impl Bot {
    /// Listen for events and execute callback function
    /// ## Parameters
    /// - `func` - callback function with [`Result`] type and [`ResponseEventsGet`] argument
    ///
    /// ## Errors
    /// - `BotError::Api` - API error when getting events
    /// - `BotError::Network` - network error when getting events
    /// - `BotError::Serialization` - response deserialization error
    /// - `BotError::System` - error when executing callback function
    pub async fn event_listener<F, X>(&self, func: F) -> Result<()>
    where
        F: Fn(Bot, ResponseEventsGet) -> X,
        X: Future<Output = Result<()>> + Send + Sync + 'static,
    {
        loop {
            debug!("Getting events with ID: {}", self.get_last_event_id());

            // Make a request to the API
            let req = RequestEventsGet::new(self.get_last_event_id()).with_poll_time(POLL_TIME);

            // Get response
            let res = self.send_api_request::<RequestEventsGet>(req).await?;

            match res.into_result()? {
                events if !events.events.is_empty() => {
                    debug!("Received {} events", events.events.len());

                    // Update last event id
                    let last_event_id = events.events[events.events.len() - 1].event_id;
                    self.set_last_event_id(last_event_id);
                    debug!("Updated last event ID: {}", last_event_id);

                    // Execute callback function
                    if let Err(e) = func(self.clone(), events).await {
                        error!("Error processing events: {}", e);
                        return Err(e);
                    }
                }
                _ => {
                    debug!("No events received, continuing to wait");
                    continue;
                }
            }
        }
    }
}
