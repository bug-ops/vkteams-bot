use crate::prelude::*;
use anyhow::Result;
use std::future::Future;
/// Listen for events and execute callback function
/// ## Parameters
/// - `func` - callback function with [`Result`] type [`ResponseEventsGet`] as argument
impl Bot {
    pub async fn event_listener<F, X>(&self, func: F) -> Result<()>
    where
        F: Fn(Bot, ResponseEventsGet) -> X,
        X: Future<Output = Result<()>> + Send + Sync + 'static,
    {
        loop {
            // Make a request to the API
            let req = RequestEventsGet::new(self.get_last_event_id()).with_poll_time(POLL_TIME);
            // Get response
            let res = self.send_api_request::<RequestEventsGet>(req).await?;
            // Update last event id
            match res {
                ApiResult::Success(events) => {
                    // If at least one event read
                    if !events.events.is_empty() {
                        // Update last event id
                        self.set_last_event_id(events.events[events.events.len() - 1].event_id);
                        // Execute callback function
                        func(self.clone(), events).await?;
                    }
                }
                ApiResult::Error { ok: _, description } => {
                    error!("Error: {:?}", description);
                    continue;
                }
            }
        }
    }
}
