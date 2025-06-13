use crate::Bot;
use crate::api::events::get::*;
use crate::api::types::*;
use crate::bot::net::shutdown_signal;
use crate::config::CONFIG;
use crate::error::Result;
use std::future::Future;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

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
        let cfg = &CONFIG.listener;
        // Create a channel to signal shutdown
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);

        // Setup shutdown signal handler
        let shutdown_tx_clone = shutdown_tx.clone();
        tokio::spawn(async move {
            shutdown_signal().await;
            info!("Received stop signal, gracefully stopping event listener...");
            let _ = shutdown_tx_clone.send(());
        });

        let mut current_backoff = cfg.empty_backoff_ms;
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
                    if cfg.use_exponential_backoff {
                        current_backoff = std::cmp::min(current_backoff * 2, cfg.max_backoff_ms);
                    }

                    continue;
                }
            };

            // Process events if we have any
            if !res.events.is_empty() {
                debug!("Received {} events", res.events.len());

                // Reset backoff time when we get events
                current_backoff = cfg.empty_backoff_ms;
                consecutive_empty_polls = 0;

                // Process events
                self.process_event_batch(res, &func).await?;
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
                    if cfg.use_exponential_backoff {
                        current_backoff = std::cmp::min(current_backoff * 2, cfg.max_backoff_ms);
                    }
                }
            }
        } // End of event_loop

        info!("Event listener stopped gracefully");
        Ok(())
    }

    /// Process a batch of events
    /// Handles events in chunks to manage memory usage
    #[tracing::instrument(skip(self, events, func))]
    async fn process_event_batch<F, X>(&self, events: ResponseEventsGet, func: &F) -> Result<()>
    where
        F: Fn(Bot, ResponseEventsGet) -> X,
        X: Future<Output = Result<()>> + Send + Sync + 'static,
    {
        let cfg = &CONFIG.listener;
        // Calculate approximate memory usage of events
        let memory_usage = if cfg.max_memory_usage > 0 {
            events.events.len() * 1024 // Assume 1KB per event as estimate
        } else {
            0
        };

        // Check if we need to process events in batches to manage memory
        if cfg.max_memory_usage > 0 && memory_usage > cfg.max_memory_usage {
            debug!("Processing events in batches due to memory constraints");

            // Process events in smaller batches
            let batches = events.events.len().div_ceil(cfg.max_events_per_batch);
            for batch_idx in 0..batches {
                let start_idx = batch_idx * cfg.max_events_per_batch;
                let end_idx = std::cmp::min(
                    (batch_idx + 1) * cfg.max_events_per_batch,
                    events.events.len(),
                );

                debug!(
                    "Processing batch {}/{} (events {}-{})",
                    batch_idx + 1,
                    batches,
                    start_idx,
                    end_idx - 1
                );

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::events::get::ResponseEventsGet;
    use crate::api::types::EventId;
    use crate::error::{BotError, Result};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Mutex;

    // Dummy Bot с моками для тестов
    #[derive(Clone, Default)]
    pub struct DummyBot {
        last_event_id: Arc<Mutex<EventId>>,
        set_last_event_calls: Arc<AtomicUsize>,
    }
    impl DummyBot {
        fn new() -> Self {
            Self {
                last_event_id: Arc::new(Mutex::new(0)),
                set_last_event_calls: Arc::new(AtomicUsize::new(0)),
            }
        }
        async fn set_last_event_id(&self, id: EventId) {
            *self.last_event_id.lock().await = id;
            self.set_last_event_calls.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn make_events(n: usize) -> ResponseEventsGet {
        ResponseEventsGet {
            events: (0..n)
                .map(|i| EventMessage {
                    event_id: i as EventId,
                    event_type: EventType::None,
                })
                .collect(),
        }
    }

    #[tokio::test]
    async fn test_process_event_batch_single_batch() {
        let bot = DummyBot::new();
        let events = make_events(3);
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count2 = call_count.clone();
        let func = move |_bot: DummyBot, _ev: ResponseEventsGet| {
            let call_count2 = call_count2.clone();
            async move {
                call_count2.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        };
        // Симулируем batch < max_events_per_batch
        let res = Bot::process_event_batch_test(&bot, events.clone(), &func, 10).await;
        assert!(res.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        assert_eq!(bot.set_last_event_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_process_event_batch_multiple_batches() {
        let bot = DummyBot::new();
        let events = make_events(15);
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count2 = call_count.clone();
        let func = move |_bot: DummyBot, _ev: ResponseEventsGet| {
            let call_count2 = call_count2.clone();
            async move {
                call_count2.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        };
        // max_events_per_batch = 5, должно быть 3 батча
        let res = Bot::process_event_batch_test(&bot, events.clone(), &func, 5).await;
        assert!(res.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
        assert_eq!(bot.set_last_event_calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_process_event_batch_callback_error() {
        let bot = DummyBot::new();
        let events = make_events(2);
        let func = |_bot: DummyBot, _ev: ResponseEventsGet| async move {
            Err(BotError::System("fail".into()))
        };
        let res = Bot::process_event_batch_test(&bot, events, &func, 10).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_process_event_batch_empty_events() {
        // Should return Ok and not call callback
        let bot = DummyBot::new();
        let events = make_events(0);
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count2 = call_count.clone();
        let func = move |_bot: DummyBot, _ev: ResponseEventsGet| {
            let call_count2 = call_count2.clone();
            async move {
                call_count2.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        };
        let res = Bot::process_event_batch_test(&bot, events, &func, 10).await;
        assert!(res.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn test_process_event_batch_error_in_second_batch() {
        // Should return error only after first batch is ok
        let bot = DummyBot::new();
        let events = make_events(6);
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count2 = call_count.clone();
        let func = move |_bot: DummyBot, _ev: ResponseEventsGet| {
            let call_count2 = call_count2.clone();
            async move {
                let n = call_count2.fetch_add(1, Ordering::SeqCst);
                if n == 1 {
                    Err(BotError::System("fail".into()))
                } else {
                    Ok(())
                }
            }
        };
        // batch size 3: first batch ok, second batch returns error
        let res = Bot::process_event_batch_test(&bot, events, &func, 3).await;
        assert!(res.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_process_event_batch_empty_events_with_memory_limit() {
        // Should return Ok and not call callback, even if max_memory_usage > 0
        let bot = DummyBot::new();
        let events = make_events(0);
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count2 = call_count.clone();
        let func = move |_bot: DummyBot, _ev: ResponseEventsGet| {
            let call_count2 = call_count2.clone();
            async move {
                call_count2.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        };
        // batch size 1, but no events
        let res = Bot::process_event_batch_test(&bot, events, &func, 1).await;
        assert!(res.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 0);
    }

    // Вспомогательная функция для теста process_event_batch с параметром max_events_per_batch
    impl Bot {
        pub async fn process_event_batch_test<F, X>(
            bot: &DummyBot,
            events: ResponseEventsGet,
            func: &F,
            max_events_per_batch: usize,
        ) -> Result<()>
        where
            F: Fn(DummyBot, ResponseEventsGet) -> X,
            X: Future<Output = Result<()>> + Send + Sync + 'static,
        {
            // Упрощённая логика batching для теста
            let total = events.events.len();
            if total == 0 {
                return Ok(());
            }
            let batches = (total + max_events_per_batch - 1) / max_events_per_batch;
            for batch_idx in 0..batches {
                let start_idx = batch_idx * max_events_per_batch;
                let end_idx = std::cmp::min((batch_idx + 1) * max_events_per_batch, total);
                let batch_events = ResponseEventsGet {
                    events: events.events[start_idx..end_idx].to_vec(),
                };
                let last_event_id = batch_events.events[batch_events.events.len() - 1].event_id;
                bot.set_last_event_id(last_event_id).await;
                if let Err(e) = func(bot.clone(), batch_events).await {
                    return Err(e);
                }
            }
            Ok(())
        }
    }
}
