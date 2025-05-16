use crate::bot::net::shutdown_signal;
use crate::error::Result;
use crate::prelude::*;
use std::future::Future;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

/// Maximum number of events to process in a single batch
static DEFAULT_MAX_EVENTS_PER_BATCH: usize = 50;

/// Default backoff time in milliseconds when no events received
static DEFAULT_EMPTY_BACKOFF_MS: u64 = 500;

/// Default maximum backoff time in milliseconds
static DEFAULT_MAX_BACKOFF_MS: u64 = 10000;

/// Configuration for event listener
#[derive(Debug, Clone)]
pub struct EventListenerConfig {
    /// Maximum number of events to process in a single batch
    pub max_events_per_batch: usize,
    /// Backoff time in milliseconds when no events received
    pub empty_backoff_ms: u64,
    /// Maximum backoff time in milliseconds
    pub max_backoff_ms: u64,
    /// Whether to use exponential backoff when no events received
    pub use_exponential_backoff: bool,
    /// Maximum memory usage for event processing in bytes (0 means no limit)
    pub max_memory_usage: usize,
}

impl Default for EventListenerConfig {
    fn default() -> Self {
        Self {
            max_events_per_batch: DEFAULT_MAX_EVENTS_PER_BATCH,
            empty_backoff_ms: DEFAULT_EMPTY_BACKOFF_MS,
            max_backoff_ms: DEFAULT_MAX_BACKOFF_MS,
            use_exponential_backoff: true,
            max_memory_usage: 0,
        }
    }
}

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
        self.event_listener_with_config(func, EventListenerConfig::default()).await
    }

    /// Listen for events with custom configuration
    /// ## Parameters
    /// - `func` - callback function with [`Result`] type and [`ResponseEventsGet`] argument
    /// - `config` - configuration for event listener
    ///
    /// ## Errors
    /// - `BotError::Api` - API error when getting events
    /// - `BotError::Network` - network error when getting events
    /// - `BotError::Serialization` - response deserialization error
    /// - `BotError::System` - error when executing callback function
    pub async fn event_listener_with_config<F, X>(&self, func: F, config: EventListenerConfig) -> Result<()>
    where
        F: Fn(Bot, ResponseEventsGet) -> X,
        X: Future<Output = Result<()>> + Send + Sync + 'static,
    {
        // Create a channel to signal shutdown
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);
        
        // Setup shutdown signal handler
        let shutdown_tx_clone = shutdown_tx.clone();
        tokio::spawn(async move {
            shutdown_signal().await;
            info!("Received stop signal, gracefully stopping event listener...");
            let _ = shutdown_tx_clone.send(());
        });
        
        let mut current_backoff = config.empty_backoff_ms;
        let mut consecutive_empty_polls = 0u32;
        
        'event_loop: loop {
            // Check if we received a shutdown signal
            if shutdown_rx.try_recv().is_ok() {
                info!("Processing shutdown request");
                break 'event_loop;
            }
            let start_time = Instant::now();
            debug!("Getting events with ID: {}", self.get_last_event_id().await);

            // Make a request to the API
            let req =
                RequestEventsGet::new(self.get_last_event_id().await).with_poll_time(POLL_TIME);

            // Get response, with error handling for network issues
            let res = match self.send_api_request::<RequestEventsGet>(req).await {
                Ok(res) => res,
                Err(e) => {
                    error!("Error getting events: {}", e);
                    
                    // Apply backoff before retrying
                    let backoff = Duration::from_millis(current_backoff);
                    warn!("Backing off for {:?} before retrying", backoff);
                    sleep(backoff).await;
                    
                    // Increase backoff time for next failure, with maximum limit
                    if config.use_exponential_backoff {
                        current_backoff = std::cmp::min(current_backoff * 2, config.max_backoff_ms);
                    }
                    
                    continue;
                }
            };

            // Try to extract the events result, with error handling
            let events = match res.into_result() {
                Ok(events) => events,
                Err(e) => {
                    error!("API error when getting events: {}", e);
                    sleep(Duration::from_millis(current_backoff)).await;
                    continue;
                }
            };

            // Process events if we have any
            if !events.events.is_empty() {
                debug!("Received {} events", events.events.len());
                
                // Reset backoff time when we get events
                current_backoff = config.empty_backoff_ms;
                consecutive_empty_polls = 0;

                // Process events
                self.process_event_batch(events, &func, &config).await?;
            } else {
                debug!("No events received, continuing to wait");
                consecutive_empty_polls += 1;
                
                // Apply backoff when no events received
                if consecutive_empty_polls > 1 {
                    // Calculate how much time we should back off
                    let elapsed = start_time.elapsed();
                    let backoff_time = Duration::from_millis(current_backoff);
                    
                    // Only sleep if we need to wait longer than we already have
                    if elapsed < backoff_time {
                        let sleep_time = backoff_time - elapsed;
                        debug!("Backing off for {:?}", sleep_time);
                        sleep(sleep_time).await;
                    }
                    
                    // Increase backoff time for next empty poll, with maximum limit
                    if config.use_exponential_backoff {
                        current_backoff = std::cmp::min(current_backoff * 2, config.max_backoff_ms);
                    }
                }
            }
        } // End of event_loop
        
        info!("Event listener stopped gracefully");
        Ok(())
    }
    
    /// Process a batch of events
    /// Handles events in chunks to manage memory usage
    async fn process_event_batch<F, X>(&self, events: ResponseEventsGet, func: &F, config: &EventListenerConfig) -> Result<()>
    where
        F: Fn(Bot, ResponseEventsGet) -> X,
        X: Future<Output = Result<()>> + Send + Sync + 'static,
    {
        // Calculate approximate memory usage of events
        let memory_usage = if config.max_memory_usage > 0 {
            events.events.len() * 1024 // Assume 1KB per event as estimate
        } else {
            0
        };

        // Check if we need to process events in batches to manage memory
        if config.max_memory_usage > 0 && memory_usage > config.max_memory_usage {
            debug!("Processing events in batches due to memory constraints");
            
            // Process events in smaller batches
            let batches = (events.events.len() + config.max_events_per_batch - 1) / config.max_events_per_batch;
            for batch_idx in 0..batches {
                let start_idx = batch_idx * config.max_events_per_batch;
                let end_idx = std::cmp::min((batch_idx + 1) * config.max_events_per_batch, events.events.len());
                
                debug!("Processing batch {}/{} (events {}-{})", 
                       batch_idx + 1, batches, start_idx, end_idx - 1);
                
                // Create a batch of events
                let batch_events = ResponseEventsGet {
                    events: events.events[start_idx..end_idx].to_vec(),
                };
                
                // Get the last event ID in this batch
                let last_event_id = batch_events.events[batch_events.events.len() - 1].event_id;
                
                // Update last event ID
                self.set_last_event_id(last_event_id).await;
                debug!("Updated last event ID: {}", last_event_id);
                
                // Process this batch of events
                if let Err(e) = func(self.clone(), batch_events).await {
                    error!("Error processing events batch: {}", e);
                    return Err(e);
                }
                
                // Brief pause between batches to allow GC to run
                sleep(Duration::from_millis(10)).await;
            }
        } else {
            // Process all events at once (original behavior)
            // Update last event id
            let last_event_id = events.events[events.events.len() - 1].event_id;
            self.set_last_event_id(last_event_id).await;
            debug!("Updated last event ID: {}", last_event_id);

            // Execute callback function
            if let Err(e) = func(self.clone(), events).await {
                error!("Error processing events: {}", e);
                return Err(e);
            }
        }
        
        Ok(())
    }
}
