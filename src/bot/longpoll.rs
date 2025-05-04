use crate::prelude::*;
use std::future::Future;
/// Listen for events and execute callback function
/// ## Parameters
/// - `func` - callback function with [`Result`] type [`ResponseEventsGet`] as argument
impl Bot {
    pub async fn event_listener<F, X>(&self, func: F)
    where
        F: Fn(Bot, ResponseEventsGet) -> X,
        X: Future<Output = ()> + Send + Sync + 'static,
    {
        loop {
            // Make a request to the API
            let req = RequestEventsGet::new(self.get_last_event_id());
            // Get response
            let res = self.send_api_request::<RequestEventsGet>(req).await;
            // Update last event id
            match res {
                Ok(events) => {
                    match events {
                        ApiResult::Success(events) => {
                            // If at least one event read
                            if !events.events.is_empty() {
                                // Update last event id
                                self.set_last_event_id(
                                    events.events[events.events.len() - 1].event_id,
                                );
                                // Execute callback function
                                func(self.clone(), events).await;
                            }
                        }
                        ApiResult::Error { ok: _, description } => {
                            error!("Error: {:?}", description);
                            continue;
                        }
                    };
                }
                Err(e) => {
                    error!("Error: {:?}", e);
                }
            }
        }
    }
}
