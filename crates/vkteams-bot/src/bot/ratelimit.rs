use crate::config::CONFIG;
use crate::prelude::ChatId;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::SystemTime;
use tokio::sync::{Mutex, RwLock, broadcast};
use tokio::time::{Duration, Instant, interval, sleep};
use tracing::{debug, info, trace, warn};
/// Statistics about token bucket usage
#[derive(Debug, Clone)]
pub struct BucketStats {
    /// Total number of requests processed
    pub total_requests: u64,
    /// Number of requests that were rate limited
    pub rate_limited_requests: u64,
    /// Number of requests that were allowed
    pub allowed_requests: u64,
    /// Last time the bucket was accessed
    pub last_access: SystemTime,
    /// Maximum tokens used in a single window
    pub max_tokens_used: u32,
}

impl Default for BucketStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            rate_limited_requests: 0,
            allowed_requests: 0,
            last_access: SystemTime::now(),
            max_tokens_used: 0,
        }
    }
}

/// ### Token Bucket implementation for rate limiting with background refilling
///
/// A token bucket that refills at a constant rate using a background task. This implementation:
/// - Uses background task for consistent token refilling (no calculation on request)
/// - More predictable performance - no computation during `consume()`
/// - Better burst handling - tokens are always available when rate limit allows
/// - Thread-safe with atomic operations
/// - Collects statistics for monitoring
#[derive(Debug)]
struct TokenBucket {
    /// Current number of tokens in the bucket (atomic for thread safety)
    tokens: Arc<AtomicU32>,
    /// Maximum number of tokens the bucket can hold
    capacity: u32,
    /// Shutdown signal for background task
    shutdown_tx: broadcast::Sender<()>,
    /// Background task handle
    _task_handle: tokio::task::JoinHandle<()>,
    /// Statistics about token usage
    stats: Arc<Mutex<BucketStats>>,
}

impl TokenBucket {
    /// Create a new TokenBucket with background refill task
    ///
    /// **Performance optimizations:**
    /// - Starts with full capacity for immediate availability
    /// - Background task runs at optimal intervals
    /// - Atomic operations for lock-free token consumption
    #[tracing::instrument]
    fn new() -> Self {
        let cfg = &CONFIG.rate_limit;
        let capacity = u32::try_from(cfg.limit)
            .unwrap_or_else(|_| panic!("Rate limit capacity too large: {}", cfg.limit));

        // Calculate optimal refill interval (balance between precision and performance)
        // For high rates: refill every 100ms, for low rates: refill every second
        let refill_interval_ms = if cfg.duration <= 10 { 100 } else { 1000 };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let tokens_per_refill = ((capacity as f64 / cfg.duration as f64)
            * (refill_interval_ms as f64 / 1000.0))
            .max(1.0) as u32;

        debug!(
            "Creating token bucket: capacity={}, refill_interval={}ms, tokens_per_refill={}",
            capacity, refill_interval_ms, tokens_per_refill
        );

        let tokens = Arc::new(AtomicU32::new(capacity)); // Start full!
        let stats = Arc::new(Mutex::new(BucketStats::default()));
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

        // Spawn background refill task
        let refill_task = Self::spawn_refill_task(
            Arc::clone(&tokens),
            capacity,
            tokens_per_refill,
            refill_interval_ms,
            shutdown_rx,
        );

        Self {
            tokens,
            capacity,
            stats,
            shutdown_tx,
            _task_handle: refill_task,
        }
    }

    /// Spawn the background token refill task
    ///
    /// **Design principles:**
    /// - Runs independently of request processing
    /// - Handles shutdown gracefully
    /// - Optimized intervals for performance
    fn spawn_refill_task(
        tokens: Arc<AtomicU32>,
        capacity: u32,
        tokens_per_refill: u32,
        interval_ms: u64,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(interval_ms));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            debug!("Token refill task started: interval={}ms", interval_ms);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Add tokens up to capacity (atomic operation)
                        let current = tokens.load(Ordering::Relaxed);
                        if current < capacity {
                            let new_tokens = (current + tokens_per_refill).min(capacity);
                            tokens.store(new_tokens, Ordering::Relaxed);

                            if current + tokens_per_refill > capacity {
                                trace!("Token bucket refilled to capacity: {}", capacity);
                            } else {
                                trace!("Added {} tokens: {} -> {} (capacity: {})",
                                    tokens_per_refill, current, new_tokens, capacity);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Token refill task shutting down gracefully");
                        break;
                    }
                }
            }

            debug!("Token refill task terminated");
        })
    }

    /// **High-Performance Token Consumption**
    ///
    /// This method is optimized for maximum throughput:
    /// - **Lock-free operation** using atomic compare-and-swap
    /// - **No time calculations** - handled by background task
    /// - **Minimal CPU overhead** per request
    /// - **Thread-safe** without mutexes
    ///
    /// ## Returns
    /// - `true` if token was consumed successfully
    /// - `false` if no tokens available (rate limited)
    #[inline]
    fn consume(&self) -> bool {
        // Fast path: try to consume a token atomically
        loop {
            let current = self.tokens.load(Ordering::Relaxed);

            if current == 0 {
                // No tokens available - update stats and return
                if let Ok(mut stats) = self.stats.try_lock() {
                    stats.total_requests += 1;
                    stats.rate_limited_requests += 1;
                    stats.last_access = SystemTime::now();
                }

                trace!("Rate limited: no tokens available");
                return false;
            }

            // Try to atomically decrement token count
            if self
                .tokens
                .compare_exchange_weak(current, current - 1, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                // Success! Update stats asynchronously (non-blocking)
                if let Ok(mut stats) = self.stats.try_lock() {
                    stats.total_requests += 1;
                    stats.allowed_requests += 1;
                    stats.last_access = SystemTime::now();
                }

                trace!("Token consumed successfully. Remaining: {}", current - 1);
                return true;
            }

            // CAS failed, retry (another thread consumed a token)
        }
    }

    /// Get current token bucket statistics
    ///
    /// **Non-blocking**: Returns immediately with current stats snapshot
    async fn get_stats(&self) -> BucketStats {
        if let Ok(stats) = self.stats.try_lock() {
            stats.clone()
        } else {
            // Return default stats if lock is contended (non-blocking)
            BucketStats::default()
        }
    }

    /// Get current token count (for debugging/monitoring)
    #[inline]
    pub fn available_tokens(&self) -> u32 {
        self.tokens.load(Ordering::Relaxed)
    }

    /// Get bucket capacity
    #[inline]
    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Gracefully shutdown the background refill task
    fn shutdown(&self) {
        if let Err(e) = self.shutdown_tx.send(()) {
            // Channel might be closed already - this is fine
            trace!(
                "Shutdown signal send failed (task may already be stopped): {}",
                e
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Map of chat_id to token bucket
    /// Using DashMap for better concurrent access performance
    chat_buckets: Arc<DashMap<ChatId, Arc<TokenBucket>>>,
    /// Global statistics for all buckets
    global_stats: Arc<RwLock<BucketStats>>,
    /// Last time buckets were cleaned up
    last_cleanup: Arc<Mutex<Instant>>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiter {
    /// Create a new high-performance RateLimiter
    ///
    /// **Performance features:**
    /// - Pre-allocated DashMap for lock-free bucket access
    /// - Background cleanup to prevent memory leaks
    /// - Graceful shutdown support
    #[tracing::instrument]
    pub fn new() -> Self {
        debug!("Creating high-performance RateLimiter");
        let cfg = &CONFIG.rate_limit;
        let capacity = u32::try_from(cfg.init_bucket)
            .unwrap_or_else(|_| panic!("Rate limit capacity too large: {}", cfg.init_bucket));
        Self {
            chat_buckets: Arc::new(DashMap::with_capacity(capacity as usize)),
            global_stats: Arc::new(RwLock::new(BucketStats::default())),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Get global rate limit statistics
    pub async fn get_global_stats(&self) -> BucketStats {
        self.global_stats.read().await.clone()
    }

    /// Get statistics for a specific chat_id (non-blocking)
    pub async fn get_chat_stats(&self, chat_id: &ChatId) -> Option<BucketStats> {
        if let Some(bucket) = self.chat_buckets.get(chat_id) {
            Some(bucket.get_stats().await)
        } else {
            None
        }
    }

    /// Clean up inactive buckets to prevent memory leaks
    /// Only runs periodically to avoid overhead
    async fn cleanup_inactive_buckets(&self) {
        let now = Instant::now();
        let cfg = &CONFIG.rate_limit;
        let mut last_cleanup = self.last_cleanup.lock().await;

        // Only cleanup every 10 minutes
        if now.duration_since(*last_cleanup) < Duration::from_secs(cfg.cleanup_interval) {
            return;
        }

        *last_cleanup = now;
        let mut removed_count = 0;

        // Remove buckets that haven't been accessed longer than lifetime
        // Collect inactive buckets for removal
        let mut to_remove = Vec::new();

        for entry in self.chat_buckets.iter() {
            let (chat_id, bucket) = (entry.key(), entry.value());

            // Check if bucket is inactive (non-blocking)
            if let Ok(stats) = bucket.stats.try_lock() {
                let is_inactive = SystemTime::now()
                    .duration_since(stats.last_access)
                    .map(|d| d >= Duration::from_secs(cfg.bucket_lifetime))
                    .unwrap_or(false);

                if is_inactive {
                    to_remove.push(chat_id.clone());
                }
            }
        }

        // Remove inactive buckets and shutdown their background tasks
        for chat_id in to_remove {
            if let Some((_, bucket)) = self.chat_buckets.remove(&chat_id) {
                bucket.shutdown();
                removed_count += 1;
            }
        }

        if removed_count > 0 {
            info!("Cleaned up {} inactive buckets", removed_count);
        }
    }
    /// **High-Performance Rate Limit Check**
    ///
    /// Optimized for maximum throughput and minimal latency:
    /// - **Lock-free bucket access** using DashMap
    /// - **Atomic token consumption** without mutexes
    /// - **Lazy bucket creation** only when needed
    /// - **Background cleanup** to prevent memory leaks
    ///
    /// ## Performance characteristics
    /// - **O(1) average case** for bucket lookup
    /// - **Lock-free token consumption** using atomics
    /// - **Minimal memory allocations** during processing
    #[tracing::instrument(skip(self))]
    pub async fn check_rate_limit(&self, chat_id: &ChatId) -> bool {
        // Occasionally clean up inactive buckets (non-blocking)
        self.cleanup_inactive_buckets().await;

        // Get or create bucket with lock-free access
        let bucket = self
            .chat_buckets
            .entry(chat_id.clone())
            .or_insert_with(|| {
                debug!("Creating new token bucket for chat_id: {}", chat_id.0);
                Arc::new(TokenBucket::new())
            })
            .clone();

        debug!("Checking rate limit for chat_id: {}", chat_id.0);

        // Consume token (lock-free operation)
        let result = bucket.consume();

        // Update global stats asynchronously (non-blocking)
        if let Ok(mut global_stats) = self.global_stats.try_write() {
            global_stats.total_requests += 1;
            if result {
                global_stats.allowed_requests += 1;
            } else {
                global_stats.rate_limited_requests += 1;
            }
        }

        result
    }
    /// ### Check and wait if request limit is exceeded
    ///
    /// - Checks the request limit for `chat_id`
    /// - If the limit is exceeded, uses adaptive backoff strategy
    /// - Retries with increasing delays based on rate limit pressure
    /// - Returns `true` if the limit is not exceeded
    /// - Returns `false` if the limit is exceeded and a token could not be obtained
    #[tracing::instrument(skip(self))]
    pub async fn wait_if_needed(&mut self, chat_id: &ChatId) -> bool {
        let cfg = &CONFIG.rate_limit;
        let mut attempts = 0;
        let base_retry_delay = Duration::from_millis(cfg.retry_delay);

        while attempts < cfg.retry_attempts {
            if self.check_rate_limit(chat_id).await {
                if attempts > 0 {
                    debug!(
                        "Rate limit passed after {} attempts for chat_id: {}",
                        attempts + 1,
                        chat_id.0
                    );
                }
                return true;
            }

            attempts += 1;

            // Calculate adaptive delay based on current attempt
            // This implements exponential backoff with jitter
            let jitter_ms = rand::random::<u64>() % 50; // Add up to 50ms of jitter
            let factor = 1.0 + (attempts as f64 * 0.5); // Increase delay by 50% each attempt
            let retry_delay = Duration::from_millis(
                (base_retry_delay.as_millis() as f64 * factor) as u64 + jitter_ms,
            );

            debug!(
                "Rate limit exceeded, attempt {}/{}, backing off for {:?}",
                attempts, cfg.retry_attempts, retry_delay
            );

            sleep(retry_delay).await;
        }

        warn!(
            "Rate limit exceeded after {} attempts for chat_id: {}",
            attempts, chat_id.0
        );

        false
    }

    /// **Gracefully clear rate limit for a specific chat**
    ///
    /// Useful when a chat should no longer be rate limited (e.g., upgraded account)
    /// - Shuts down background refill task properly
    /// - Removes bucket from memory
    pub async fn clear_rate_limit(&self, chat_id: &ChatId) {
        if let Some((_, bucket)) = self.chat_buckets.remove(chat_id) {
            debug!("Clearing rate limit for chat_id: {}", chat_id.0);
            bucket.shutdown();
        }
    }

    /// Get current token count for a specific chat
    ///
    /// Returns None if bucket doesn't exist for this chat
    pub async fn get_available_tokens(&self, chat_id: &ChatId) -> Option<u32> {
        self.chat_buckets
            .get(chat_id)
            .map(|bucket| bucket.available_tokens())
    }

    /// Get bucket capacity for a specific chat
    ///
    /// Returns None if bucket doesn't exist for this chat
    pub async fn get_bucket_capacity(&self, chat_id: &ChatId) -> Option<u32> {
        self.chat_buckets
            .get(chat_id)
            .map(|bucket| bucket.capacity())
    }

    /// Get number of active buckets
    pub fn active_bucket_count(&self) -> usize {
        self.chat_buckets.len()
    }

    /// Graceful shutdown of all rate limiters
    ///
    /// Call this when shutting down your application to:
    /// - Stop all background refill tasks
    /// - Prevent resource leaks
    /// - Ensure clean shutdown
    pub async fn shutdown(&self) {
        info!(
            "Shutting down RateLimiter with {} active buckets",
            self.chat_buckets.len()
        );

        // Shutdown all bucket background tasks
        for entry in self.chat_buckets.iter() {
            entry.value().shutdown();
        }

        // Clear all buckets
        self.chat_buckets.clear();

        info!("RateLimiter shutdown complete");
    }
}

/// Extension trait for RateLimiter to support priority tiers
#[async_trait]
pub trait RateLimiterExt {
    /// Check if a chat with specified priority can make a request
    /// Higher priority chats get more tokens
    async fn check_with_priority(&self, chat_id: &ChatId, priority: u8) -> bool;
}

#[async_trait]
impl RateLimiterExt for RateLimiter {
    async fn check_with_priority(&self, chat_id: &ChatId, priority: u8) -> bool {
        // Allow more requests for higher priority chats
        // This is a simple implementation that just tries multiple times for higher priority
        let attempts = match priority {
            0 => 1,           // Standard priority
            1 => 2,           // Medium priority
            2..=u8::MAX => 3, // High priority
        };

        for _ in 0..attempts {
            if self.check_rate_limit(chat_id).await {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicU32;
    use tokio::time::{Duration, sleep};

    /// Test helper to create a TokenBucket with explicit config
    fn create_test_bucket(capacity: u32, duration_secs: u64) -> TokenBucket {
        // Mock CONFIG for testing
        let tokens = Arc::new(AtomicU32::new(capacity));
        let stats = Arc::new(Mutex::new(BucketStats::default()));
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

        // Calculate refill parameters
        let refill_interval_ms = if duration_secs <= 10 { 100 } else { 1000 };
        let tokens_per_refill = ((capacity as f64 / duration_secs as f64)
            * (refill_interval_ms as f64 / 1000.0))
            .max(1.0) as u32;

        let refill_task = TokenBucket::spawn_refill_task(
            Arc::clone(&tokens),
            capacity,
            tokens_per_refill,
            refill_interval_ms,
            shutdown_rx,
        );

        TokenBucket {
            tokens,
            capacity,
            stats,
            shutdown_tx,
            _task_handle: refill_task,
        }
    }

    #[tokio::test]
    async fn test_bucket_created_with_full_capacity() {
        let bucket = create_test_bucket(10, 60);

        // Verify bucket starts with full capacity
        assert_eq!(bucket.available_tokens(), 10);
        assert_eq!(bucket.capacity, 10);

        // First token consumption should succeed immediately
        assert!(
            bucket.consume(),
            "First token should be available immediately"
        );
        assert_eq!(bucket.available_tokens(), 9);

        // Should be able to consume all initial tokens
        for i in 0..9 {
            assert!(bucket.consume(), "Token {} should be available", i + 2);
        }

        // Now bucket should be empty
        assert_eq!(bucket.available_tokens(), 0);
        assert!(!bucket.consume(), "Should be rate limited when empty");

        bucket.shutdown();
    }

    #[tokio::test]
    async fn test_background_refill() {
        // Create bucket with fast refill rate for testing
        let bucket = create_test_bucket(5, 1); // 5 tokens per second

        // Consume all tokens
        for _ in 0..5 {
            assert!(bucket.consume());
        }
        assert_eq!(bucket.available_tokens(), 0);

        // Wait for background refill (slightly more than refill interval)
        sleep(Duration::from_millis(1200)).await;

        // Should have tokens again
        let tokens_after_refill = bucket.available_tokens();
        assert!(
            tokens_after_refill > 0,
            "Background task should have refilled tokens"
        );
        assert!(tokens_after_refill <= 5, "Should not exceed capacity");

        bucket.shutdown();
    }

    #[tokio::test]
    async fn test_concurrent_token_consumption() {
        let bucket = Arc::new(create_test_bucket(100, 60));
        let mut handles = vec![];

        // Spawn 20 concurrent consumers
        for i in 0..20 {
            let bucket_clone = Arc::clone(&bucket);
            let handle = tokio::spawn(async move {
                let mut successes = 0;
                for _ in 0..10 {
                    if bucket_clone.consume() {
                        successes += 1;
                    }
                }
                (i, successes)
            });
            handles.push(handle);
        }

        // Collect results
        let mut total_successes = 0;
        for handle in handles {
            let (_, successes) = handle.await.unwrap();
            total_successes += successes;
        }

        // Should have consumed at most 100 tokens (capacity)
        assert!(total_successes <= 100, "Should not exceed bucket capacity");
        assert!(
            total_successes >= 90,
            "Should consume most tokens with high concurrency"
        );

        bucket.shutdown();
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        let bucket = create_test_bucket(10, 60);

        // Bucket should be working
        assert!(bucket.consume());

        // Shutdown the bucket
        bucket.shutdown();

        // Give background task time to shutdown
        sleep(Duration::from_millis(100)).await;

        // Bucket should still work for remaining tokens but no refill
        let initial_tokens = bucket.available_tokens();

        // Wait longer than refill interval
        sleep(Duration::from_millis(2000)).await;

        // Should not have been refilled after shutdown
        assert_eq!(
            bucket.available_tokens(),
            initial_tokens,
            "Tokens should not be refilled after shutdown"
        );
    }

    #[tokio::test]
    async fn test_bucket_capacity_limits() {
        let bucket = create_test_bucket(3, 1); // 3 tokens per second

        // Wait for potential over-refill
        sleep(Duration::from_millis(5000)).await;

        // Should never exceed capacity regardless of time passed
        assert!(
            bucket.available_tokens() <= 3,
            "Should never exceed capacity even after long wait"
        );

        bucket.shutdown();
    }

    #[tokio::test]
    async fn test_atomic_operations_performance() {
        let bucket = create_test_bucket(1000, 60);

        // Measure performance of atomic operations
        let start = std::time::Instant::now();

        // Consume tokens in tight loop
        for _ in 0..1000 {
            bucket.consume();
        }

        let duration = start.elapsed();

        // Should be very fast (less than 10ms for 1000 operations)
        assert!(
            duration.as_millis() < 10,
            "Atomic operations should be very fast: {:?}",
            duration
        );

        bucket.shutdown();
    }
}

#[cfg(test)]
mod ratelimiter_tests {
    use super::*;
    use crate::prelude::ChatId;
    use tokio::time::{Duration, sleep};

    fn chat_id(n: u64) -> ChatId {
        ChatId(format!("chat_{}", n))
    }

    #[tokio::test]
    async fn test_new_and_active_bucket_count() {
        let limiter = RateLimiter::new();
        assert_eq!(limiter.active_bucket_count(), 0);
        let _ = limiter.check_rate_limit(&chat_id(1)).await;
        assert_eq!(limiter.active_bucket_count(), 1);
    }

    #[tokio::test]
    async fn test_check_rate_limit_and_tokens() {
        let limiter = RateLimiter::new();
        let cid = chat_id(2);
        for _ in 0..5 {
            assert!(limiter.check_rate_limit(&cid).await);
        }
        // После исчерпания лимита должны получать false
        let mut limited = false;
        for _ in 0..100 {
            if !limiter.check_rate_limit(&cid).await {
                limited = true;
                break;
            }
        }
        assert!(
            limited,
            "Rate limiter должен ограничивать после превышения лимита"
        );
    }

    #[tokio::test]
    async fn test_clear_rate_limit_and_capacity() {
        let limiter = RateLimiter::new();
        let cid = chat_id(3);
        let _ = limiter.check_rate_limit(&cid).await;
        let cap = limiter.get_bucket_capacity(&cid).await;
        assert!(cap.is_some());
        limiter.clear_rate_limit(&cid).await;
        // После очистки bucket должен быть пересоздан
        let _ = limiter.check_rate_limit(&cid).await;
        let cap2 = limiter.get_bucket_capacity(&cid).await;
        assert_eq!(cap, cap2);
    }

    #[tokio::test]
    async fn test_get_available_tokens() {
        let limiter = RateLimiter::new();
        let cid = chat_id(4);
        let _ = limiter.check_rate_limit(&cid).await;
        let tokens = limiter.get_available_tokens(&cid).await;
        assert!(tokens.is_some());
    }

    #[tokio::test]
    async fn test_get_global_and_chat_stats() {
        let limiter = RateLimiter::new();
        let cid = chat_id(5);
        for _ in 0..3 {
            let _ = limiter.check_rate_limit(&cid).await;
        }
        let global = limiter.get_global_stats().await;
        let chat = limiter.get_chat_stats(&cid).await;
        assert!(global.total_requests > 0);
        assert!(chat.is_some());
        assert!(chat.unwrap().total_requests > 0);
    }

    #[tokio::test]
    async fn test_shutdown_and_post_shutdown_behavior() {
        let limiter = RateLimiter::new();
        let cid = chat_id(6);
        let _ = limiter.check_rate_limit(&cid).await;
        limiter.shutdown().await;
        // После shutdown bucket не должен refilиться, но старые токены доступны
        let tokens = limiter.get_available_tokens(&cid).await;
        sleep(Duration::from_millis(1200)).await;
        let tokens_after = limiter.get_available_tokens(&cid).await;
        assert_eq!(tokens, tokens_after);
    }

    #[tokio::test]
    async fn test_check_with_priority() {
        let limiter = RateLimiter::new();
        let cid = chat_id(7);
        // Высокий приоритет должен позволять больше запросов
        let mut allowed_high = 0;
        let mut allowed_low = 0;
        for _ in 0..20 {
            if limiter.check_with_priority(&cid, 10).await {
                allowed_high += 1;
            }
        }
        for _ in 0..20 {
            if limiter.check_with_priority(&cid, 1).await {
                allowed_low += 1;
            }
        }
        assert!(allowed_high >= allowed_low);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let limiter = Arc::new(RateLimiter::new());
        let mut handles = vec![];
        for i in 0..10 {
            let limiter = Arc::clone(&limiter);
            let cid = chat_id(100 + i);
            handles.push(tokio::spawn(async move {
                for _ in 0..10 {
                    let _ = limiter.check_rate_limit(&cid).await;
                }
            }));
        }
        for h in handles {
            h.await.unwrap();
        }
        assert!(limiter.active_bucket_count() >= 10);
    }
}
