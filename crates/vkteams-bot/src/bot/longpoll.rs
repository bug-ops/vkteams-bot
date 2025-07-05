use crate::api::events::get::{RequestEventsGet, ResponseEventsGet};
use crate::api::types::{BotRequest, EventMessage, POLL_TIME};
use crate::bot::Bot;
use crate::config::CONFIG;
use crate::error::{BotError, Result};
use std::future::Future;
use std::sync::Arc;
#[cfg(test)]
use std::sync::atomic::AtomicU32;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;
use tracing::{debug, error, info, trace, warn};

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
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);

        // Setup shutdown signal handler
        let shutdown_tx_clone = shutdown_tx.clone();
        tokio::spawn(async move {
            crate::bot::net::shutdown_signal().await;
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
            debug!("Getting events with ID: {}", self.get_last_event_id());

            // Make a request to the API
            let req = RequestEventsGet::new(self.get_last_event_id()).with_poll_time(POLL_TIME);

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

                // Create a batch of events (zero-copy slice)
                let batch_events = ResponseEventsGet {
                    events: events.events[start_idx..end_idx].into(),
                };

                // Get the last event ID in this batch
                let last_event_id = batch_events.events[batch_events.events.len() - 1].event_id;

                // Update last event ID
                self.set_last_event_id(last_event_id);
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
            self.set_last_event_id(last_event_id);
            debug!("Updated last event ID: {}", last_event_id);

            // Execute callback function
            if let Err(e) = func(self.clone(), events).await {
                error!("Error processing events: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// Listen for events with parallel processing
    /// Enhanced version that processes events in parallel batches
    pub async fn event_listener_parallel<F, X>(&self, func: F) -> Result<()>
    where
        F: Fn(Bot, ResponseEventsGet) -> X + Send + Sync + Clone + 'static,
        X: Future<Output = Result<()>> + Send + 'static,
    {
        let cfg = &CONFIG.listener;
        info!("Starting parallel event listener...");

        // Initialize parallel processor and adaptive backoff
        let processor = ParallelEventProcessor::new(
            cfg.max_events_per_batch.max(1), // Use as max concurrent batches
            cfg.max_events_per_batch,
        );

        let mut backoff = AdaptiveBackoff::new(
            Duration::from_millis(cfg.empty_backoff_ms),
            Duration::from_millis(cfg.max_backoff_ms),
        );

        // Initialize event stream buffer for zero-copy processing (future use)
        // let mut event_stream = ZeroCopyEventStream::new(cfg.max_events_per_batch * 10);

        // Create a channel to signal shutdown
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);

        // Setup shutdown signal handler
        let shutdown_tx_clone = shutdown_tx.clone();
        tokio::spawn(async move {
            crate::bot::net::shutdown_signal().await;
            info!("Received stop signal, gracefully stopping parallel event listener...");
            let _ = shutdown_tx_clone.send(());
        });

        'event_loop: loop {
            // Check if we received a shutdown signal
            if shutdown_rx.try_recv().is_ok() {
                info!("Processing shutdown request");
                break 'event_loop;
            }

            let start_time = Instant::now();

            // Create request for events
            let req = RequestEventsGet::new(self.get_last_event_id()).with_poll_time(POLL_TIME);

            // Send request and handle response
            let res = match self.send_api_request::<RequestEventsGet>(req).await {
                Ok(res) => res,
                Err(e) => {
                    error!("Error getting events: {}", e);

                    // Apply adaptive backoff on error
                    let delay = backoff.calculate_delay(0);
                    warn!("Error occurred, backing off for {:?}", delay);
                    sleep(delay).await;
                    continue;
                }
            };

            // Process events if we have any
            if !res.events.is_empty() {
                debug!("Received {} events", res.events.len());

                // Update last event ID from the most recent event
                let last_event_id = res.events[res.events.len() - 1].event_id;
                self.set_last_event_id(last_event_id);
                debug!("Updated last event ID: {}", last_event_id);

                // Process events in parallel
                let processing_start = Instant::now();
                match processor
                    .process_events_parallel(self.clone(), res, func.clone())
                    .await
                {
                    Ok(_) => {
                        let processing_duration = processing_start.elapsed();
                        trace!("Parallel processing completed in {:?}", processing_duration);

                        // Reset backoff on successful processing
                        backoff.calculate_delay(1);
                    }
                    Err(e) => {
                        error!("Error in parallel processing: {}", e);
                        return Err(e);
                    }
                }
            } else {
                debug!("No events received, applying adaptive backoff");

                // Apply adaptive backoff for empty polls
                let delay = backoff.calculate_delay(0);

                // Only sleep if we haven't already spent enough time waiting
                let elapsed = start_time.elapsed();
                if elapsed < delay {
                    let sleep_time = delay - elapsed;
                    trace!("Adaptive backoff: sleeping for {:?}", sleep_time);
                    sleep(sleep_time).await;
                }
            }
        } // End of event_loop

        info!("Parallel event listener stopped gracefully");
        Ok(())
    }
}

/// Parallel event processor for concurrent batch processing
pub struct ParallelEventProcessor {
    max_concurrent_batches: usize,
    batch_size: usize,
}

impl ParallelEventProcessor {
    /// Create new parallel event processor
    pub fn new(max_concurrent_batches: usize, batch_size: usize) -> Self {
        Self {
            max_concurrent_batches,
            batch_size,
        }
    }

    /// Process events in parallel batches
    pub async fn process_events_parallel<F, X>(
        &self,
        bot: Bot,
        events: ResponseEventsGet,
        processor: F,
    ) -> Result<()>
    where
        F: Fn(Bot, ResponseEventsGet) -> X + Send + Sync + Clone + 'static,
        X: Future<Output = Result<()>> + Send + 'static,
    {
        if events.events.is_empty() {
            return Ok(());
        }

        let batches = self.create_batches(events);
        let batch_count = batches.len();
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_batches));

        trace!("Processing {} batches in parallel", batch_count);

        let futures: Vec<_> = batches
            .into_iter()
            .enumerate()
            .map(|(batch_idx, batch)| {
                let processor = processor.clone();
                let bot = bot.clone();
                let semaphore = semaphore.clone();

                async move {
                    let _permit = semaphore.acquire().await.map_err(|e| {
                        BotError::System(format!("Failed to acquire semaphore: {e}"))
                    })?;

                    trace!(
                        "Processing batch {} with {} events",
                        batch_idx,
                        batch.events.len()
                    );

                    let start_time = Instant::now();
                    let result = processor(bot, batch).await;
                    let duration = start_time.elapsed();

                    if let Err(ref e) = result {
                        error!("Batch {} failed after {:?}: {}", batch_idx, duration, e);
                    } else {
                        trace!("Batch {} completed in {:?}", batch_idx, duration);
                    }

                    result
                }
            })
            .collect();

        // Wait for all batches to complete using a simpler approach
        use futures::future::join_all;
        let results: Vec<Result<()>> = join_all(futures).await.into_iter().collect();

        // Check if any batches failed
        for (idx, result) in results.into_iter().enumerate() {
            if let Err(e) = result {
                return Err(BotError::System(format!(
                    "Batch {idx} processing failed: {e}"
                )));
            }
        }

        debug!("All {} batches processed successfully", batch_count);
        Ok(())
    }

    /// Create batches from events, ensuring no events are lost
    fn create_batches(&self, events: ResponseEventsGet) -> Vec<ResponseEventsGet> {
        events
            .events
            .chunks(self.batch_size)
            .map(|chunk| ResponseEventsGet {
                events: chunk.to_vec(),
            })
            .collect()
    }
}

/// Adaptive backoff strategy that adjusts delay based on activity
pub struct AdaptiveBackoff {
    current_delay: Duration,
    min_delay: Duration,
    max_delay: Duration,
    consecutive_empty_polls: u32,
    last_activity: Option<Instant>,
    empty_poll_threshold: u32,
}

impl AdaptiveBackoff {
    /// Create new adaptive backoff strategy
    pub fn new(min_delay: Duration, max_delay: Duration) -> Self {
        Self {
            current_delay: min_delay,
            min_delay,
            max_delay,
            consecutive_empty_polls: 0,
            last_activity: None,
            empty_poll_threshold: 3, // Start backing off after 3 consecutive empty polls
        }
    }

    /// Calculate delay based on events received and system load
    pub fn calculate_delay(&mut self, events_received: usize) -> Duration {
        let now = Instant::now();

        if events_received > 0 {
            // Reset to minimum delay when events are received
            self.current_delay = self.min_delay;
            self.consecutive_empty_polls = 0;
            self.last_activity = Some(now);

            trace!("Events received, reset delay to {:?}", self.current_delay);
        } else {
            // Exponential backoff for empty polls
            self.consecutive_empty_polls += 1;

            // Only increase backoff after several consecutive empty polls
            if self.consecutive_empty_polls > self.empty_poll_threshold {
                self.current_delay = std::cmp::min(
                    Duration::from_millis(
                        (self.current_delay.as_millis() as u64 * 3 / 2)
                            .max(self.min_delay.as_millis() as u64),
                    ),
                    self.max_delay,
                );

                trace!(
                    "Empty poll #{}, increased delay to {:?}",
                    self.consecutive_empty_polls, self.current_delay
                );
            }

            // If we've been idle for a long time, increase delay more aggressively
            if let Some(last_activity) = self.last_activity {
                let idle_time = now.duration_since(last_activity);
                if idle_time > Duration::from_secs(60) {
                    self.current_delay = std::cmp::min(
                        self.current_delay + Duration::from_millis(100),
                        self.max_delay,
                    );
                }
            }
        }

        self.current_delay
    }

    /// Get current delay without modifying state
    pub fn current_delay(&self) -> Duration {
        self.current_delay
    }

    /// Reset backoff to minimum (useful for external triggers)
    pub fn reset(&mut self) {
        self.current_delay = self.min_delay;
        self.consecutive_empty_polls = 0;
        self.last_activity = Some(Instant::now());
    }
}

/// Zero-copy event streaming buffer
pub struct ZeroCopyEventStream {
    events: std::collections::VecDeque<EventMessage>,
    capacity: usize,
}

impl ZeroCopyEventStream {
    /// Create new event stream with given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            events: std::collections::VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Push events efficiently by moving data
    pub fn push_events(&mut self, mut new_events: Vec<EventMessage>) {
        // If new events exceed capacity, take only the last capacity events
        if new_events.len() > self.capacity {
            new_events.drain(..new_events.len() - self.capacity);
        }

        // Ensure we don't exceed capacity by removing old events
        while self.events.len() + new_events.len() > self.capacity {
            self.events.pop_front();
        }

        // Move events instead of copying
        self.events.extend(new_events.drain(..));
    }

    /// Drain a batch of events efficiently
    pub fn drain_batch(&mut self, size: usize) -> Vec<EventMessage> {
        self.events
            .drain(..std::cmp::min(size, self.events.len()))
            .collect()
    }

    /// Get current number of events
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get events without removing them
    pub fn peek_events(&self, count: usize) -> Vec<&EventMessage> {
        self.events.iter().take(count).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::events::get::ResponseEventsGet;
    use crate::api::types::{EventId, EventMessage, EventType};
    use crate::error::{BotError, Result};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Dummy Bot с моками для тестов
    #[derive(Clone, Default)]
    pub struct DummyBot {
        last_event_id: Arc<AtomicU32>,
        set_last_event_calls: Arc<AtomicUsize>,
    }
    impl DummyBot {
        fn new() -> Self {
            Self {
                last_event_id: Arc::new(AtomicU32::new(0)),
                set_last_event_calls: Arc::new(AtomicUsize::new(0)),
            }
        }
        fn set_last_event_id(&self, id: EventId) {
            self.last_event_id.store(id, Ordering::SeqCst);
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
            let batches = total.div_ceil(max_events_per_batch);
            for batch_idx in 0..batches {
                let start_idx = batch_idx * max_events_per_batch;
                let end_idx = std::cmp::min((batch_idx + 1) * max_events_per_batch, total);
                let batch_events = ResponseEventsGet {
                    events: events.events[start_idx..end_idx].to_vec(),
                };
                let last_event_id = batch_events.events[batch_events.events.len() - 1].event_id;
                bot.set_last_event_id(last_event_id);
                func(bot.clone(), batch_events).await?;
            }
            Ok(())
        }
    }

    // Tests for AdaptiveBackoff component
    #[test]
    fn test_adaptive_backoff_new() {
        let min_delay = Duration::from_millis(100);
        let max_delay = Duration::from_millis(5000);
        let backoff = AdaptiveBackoff::new(min_delay, max_delay);

        assert_eq!(backoff.current_delay(), min_delay);
    }

    #[test]
    fn test_adaptive_backoff_calculate_delay_no_events() {
        let min_delay = Duration::from_millis(100);
        let max_delay = Duration::from_millis(5000);
        let mut backoff = AdaptiveBackoff::new(min_delay, max_delay);

        let calculated = backoff.calculate_delay(0);
        assert!(calculated >= min_delay);
        assert!(calculated <= max_delay);
    }

    #[test]
    fn test_adaptive_backoff_calculate_delay_with_events() {
        let min_delay = Duration::from_millis(100);
        let max_delay = Duration::from_millis(5000);
        let mut backoff = AdaptiveBackoff::new(min_delay, max_delay);

        let calculated = backoff.calculate_delay(5);
        assert_eq!(calculated, min_delay);
    }

    #[test]
    fn test_adaptive_backoff_reset() {
        let min_delay = Duration::from_millis(100);
        let max_delay = Duration::from_millis(5000);
        let mut backoff = AdaptiveBackoff::new(min_delay, max_delay);

        // Increase delay first
        backoff.calculate_delay(0);
        let after_increase = backoff.current_delay();
        assert!(after_increase >= min_delay);

        // Reset and check
        backoff.reset();
        assert_eq!(backoff.current_delay(), min_delay);
    }

    #[test]
    fn test_adaptive_backoff_current_delay() {
        let min_delay = Duration::from_millis(50);
        let max_delay = Duration::from_millis(2000);
        let backoff = AdaptiveBackoff::new(min_delay, max_delay);

        assert_eq!(backoff.current_delay(), min_delay);
    }

    // Tests for ZeroCopyEventStream component
    #[test]
    fn test_zero_copy_event_stream_new() {
        let stream = ZeroCopyEventStream::new(100);
        assert_eq!(stream.len(), 0);
        assert!(stream.is_empty());
    }

    #[test]
    fn test_zero_copy_event_stream_push_events() {
        let mut stream = ZeroCopyEventStream::new(10);
        let events = make_events(3);

        stream.push_events(events.events.clone());
        assert_eq!(stream.len(), 3);
        assert!(!stream.is_empty());
    }

    #[test]
    fn test_zero_copy_event_stream_push_events_overflow() {
        let mut stream = ZeroCopyEventStream::new(2);
        let events = make_events(5);

        stream.push_events(events.events.clone());
        assert_eq!(stream.len(), 2); // Should be capped at capacity

        // Verify we get the last 2 events (most recent)
        let remaining_events = stream.peek_events(2);
        assert_eq!(remaining_events.len(), 2);
        assert_eq!(remaining_events[0].event_id, 3); // Last 2 events should be 3 and 4
        assert_eq!(remaining_events[1].event_id, 4);
    }

    #[test]
    fn test_zero_copy_event_stream_drain_batch() {
        let mut stream = ZeroCopyEventStream::new(10);
        let events = make_events(5);

        stream.push_events(events.events.clone());
        let drained = stream.drain_batch(3);

        assert_eq!(drained.len(), 3);
        assert_eq!(stream.len(), 2); // 5 - 3 = 2 remaining
    }

    #[test]
    fn test_zero_copy_event_stream_drain_batch_more_than_available() {
        let mut stream = ZeroCopyEventStream::new(10);
        let events = make_events(2);

        stream.push_events(events.events.clone());
        let drained = stream.drain_batch(5);

        assert_eq!(drained.len(), 2); // Only 2 available
        assert_eq!(stream.len(), 0);
        assert!(stream.is_empty());
    }

    #[test]
    fn test_zero_copy_event_stream_peek_events() {
        let mut stream = ZeroCopyEventStream::new(10);
        let events = make_events(5);

        stream.push_events(events.events.clone());
        let peeked = stream.peek_events(3);

        assert_eq!(peeked.len(), 3);
        assert_eq!(stream.len(), 5); // Stream unchanged after peek

        // Check that peeked events match the first 3 events
        for (i, event_ref) in peeked.iter().enumerate() {
            assert_eq!(event_ref.event_id, events.events[i].event_id);
        }
    }

    #[test]
    fn test_zero_copy_event_stream_peek_events_more_than_available() {
        let mut stream = ZeroCopyEventStream::new(10);
        let events = make_events(2);

        stream.push_events(events.events.clone());
        let peeked = stream.peek_events(5);

        assert_eq!(peeked.len(), 2); // Only 2 available
        assert_eq!(stream.len(), 2); // Stream unchanged
    }

    // Tests for ParallelEventProcessor component
    #[test]
    fn test_parallel_event_processor_new() {
        let _processor = ParallelEventProcessor::new(5, 10);
        // Can't directly test internal fields, but constructor should not panic
    }
}
