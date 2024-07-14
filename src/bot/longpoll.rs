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
                    let evt = events.events.clone();
                    // If at least one event read
                    if !evt.is_empty() {
                        // Update last event id
                        self.set_last_event_id(evt[evt.len() - 1].event_id);
                        // Execute callback function
                        func(self.clone(), events).await;
                    }
                }
                Err(e) => {
                    error!("Error: {:?}", e);
                }
            }
        }
    }
}
