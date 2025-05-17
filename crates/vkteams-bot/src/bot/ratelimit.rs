use crate::config::CONFIG;
use crate::prelude::ChatId;
use dashmap::DashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{Duration, Instant, sleep};
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

/// ### Token Bucket implementation for rate limiting
///
/// A token bucket that refills at a constant rate over time. This implementation:
/// - Uses token bucket algorithm instead of semaphore for better performance
/// - Calculates tokens based on time elapsed since last request
/// - More precise than periodic replenishing
/// - Thread-safe with minimal locking
/// - Collects statistics for monitoring
#[derive(Debug)]
struct TokenBucket {
    /// Current number of tokens in the bucket
    tokens: f64,
    /// Maximum number of tokens the bucket can hold
    capacity: u32,
    /// Rate at which tokens are added to the bucket (tokens per second)
    refill_rate: f64,
    /// Last time the bucket was refilled
    last_refill: Instant,
    /// Statistics about token usage
    stats: BucketStats,
    /// Recent request timestamps for adaptive rate limiting
    recent_requests: VecDeque<Instant>,
}

impl TokenBucket {
    #[tracing::instrument]
    fn new() -> Self {
        let cfg = &CONFIG.rate_limit;
        let capacity = cfg.limit as u32;

        // Calculate tokens per second based on configuration
        // If duration is 60 seconds and limit is 100, then refill_rate is 100/60 = 1.67 tokens per second
        let refill_rate = capacity as f64 / cfg.duration as f64;

        debug!(
            "Creating new token bucket with capacity: {}, refill rate: {:.2} tokens/sec",
            capacity, refill_rate
        );

        Self {
            tokens: capacity as f64,
            capacity,
            refill_rate,
            last_refill: Instant::now(),
            stats: BucketStats {
                total_requests: 0,
                rate_limited_requests: 0,
                allowed_requests: 0,
                last_access: SystemTime::now(),
                max_tokens_used: 0,
            },
            recent_requests: VecDeque::with_capacity(32), // Track recent requests for burst detection
        }
    }

    /// Try to consume a token from the bucket
    ///
    /// - Refills tokens based on time elapsed since last request
    /// - More accurate and efficient than periodic replenishing
    /// - Returns `true` if a token was consumed
    /// - Returns `false` if no tokens are available
    #[tracing::instrument(skip(self))]
    fn consume(&mut self) -> bool {
        let cfg = &CONFIG.rate_limit;
        let now = Instant::now();
        self.stats.last_access = SystemTime::now();
        self.stats.total_requests += 1;

        // Calculate how many tokens to add based on time elapsed
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let new_tokens = elapsed * self.refill_rate;

        // Add new tokens, up to capacity
        self.tokens = (self.tokens + new_tokens).min(self.capacity as f64);
        self.last_refill = now;

        // Record this request timestamp
        self.recent_requests.push_back(now);

        // Remove old request timestamps (older than the rate limit duration)
        while let Some(timestamp) = self.recent_requests.front() {
            if now.duration_since(*timestamp).as_secs() > cfg.duration {
                self.recent_requests.pop_front();
            } else {
                break;
            }
        }

        // Check for potential burst and adjust tokens if needed
        let recent_count = self.recent_requests.len() as u32;
        self.stats.max_tokens_used = self.stats.max_tokens_used.max(recent_count);

        // If we have at least one token, consume it
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            self.stats.allowed_requests += 1;
            trace!(
                "Token consumed. Remaining tokens: {:.2}, Recent requests: {}",
                self.tokens, recent_count
            );
            true
        } else {
            self.stats.rate_limited_requests += 1;
            trace!(
                "No tokens available. Recent requests: {}, Max tokens used: {}",
                recent_count, self.stats.max_tokens_used
            );
            false
        }
    }

    /// Get current token bucket statistics
    fn get_stats(&self) -> BucketStats {
        self.stats.clone()
    }
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Map of chat_id to token bucket
    /// Using DashMap for better concurrent access performance
    chat_buckets: Arc<DashMap<ChatId, Arc<Mutex<TokenBucket>>>>,
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
    #[tracing::instrument]
    pub fn new() -> Self {
        debug!("Creating new RateLimiter");
        Self {
            chat_buckets: Arc::new(DashMap::with_capacity(100)),
            global_stats: Arc::new(RwLock::new(BucketStats::default())),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Get global rate limit statistics
    pub async fn get_global_stats(&self) -> BucketStats {
        self.global_stats.read().await.clone()
    }

    /// Get statistics for a specific chat_id
    pub async fn get_chat_stats(&self, chat_id: &ChatId) -> Option<BucketStats> {
        if let Some(bucket) = self.chat_buckets.get(chat_id) {
            let bucket_lock = bucket.lock().await;
            Some(bucket_lock.get_stats())
        } else {
            None
        }
    }

    /// Clean up inactive buckets to prevent memory leaks
    /// Only runs periodically to avoid overhead
    async fn cleanup_inactive_buckets(&self) {
        let now = Instant::now();
        let mut last_cleanup = self.last_cleanup.lock().await;

        // Only cleanup every 10 minutes
        if now.duration_since(*last_cleanup) < Duration::from_secs(600) {
            return;
        }

        *last_cleanup = now;
        let mut removed_count = 0;

        // Remove buckets that haven't been accessed in an hour
        self.chat_buckets.retain(|_, bucket| {
            // Try to get a lock, if we can't (someone is using it), keep it
            if let Ok(bucket_lock) = bucket.try_lock() {
                let stats = bucket_lock.get_stats();
                let is_active = SystemTime::now()
                    .duration_since(stats.last_access)
                    .map(|d| d < Duration::from_secs(3600))
                    .unwrap_or(true);

                if !is_active {
                    removed_count += 1;
                    return false;
                }
            }
            true
        });

        if removed_count > 0 {
            info!("Cleaned up {} inactive buckets", removed_count);
        }
    }
    /// ### Check request limit for `chat_id`
    ///
    /// - Checks the request limit for `chat_id` using token bucket algorithm
    /// - Returns `true` if the limit is not exceeded
    /// - Returns `false` if the limit is exceeded
    /// - Updates statistics for monitoring
    #[tracing::instrument(skip(self))]
    pub async fn check_rate_limit(&self, chat_id: &ChatId) -> bool {
        // Occasionally clean up inactive buckets
        self.cleanup_inactive_buckets().await;

        // Get or create bucket for this chat_id
        let bucket = self
            .chat_buckets
            .entry(chat_id.clone())
            .or_insert_with(|| {
                debug!("Creating new token bucket for chat_id: {}", chat_id.0);
                Arc::new(Mutex::new(TokenBucket::new()))
            })
            .clone();

        debug!("Checking rate limit for chat_id: {}", chat_id.0);

        // Try to consume a token
        let result = {
            let mut bucket_lock = bucket.lock().await;
            bucket_lock.consume()
        };

        // Update global stats
        {
            let mut global_stats = self.global_stats.write().await;
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

    /// Clear rate limit cache for a specific chat
    /// Useful when a chat should no longer be rate limited (e.g., upgraded account)
    pub async fn clear_rate_limit(&self, chat_id: &ChatId) {
        if self.chat_buckets.contains_key(chat_id) {
            debug!("Clearing rate limit for chat_id: {}", chat_id.0);
            self.chat_buckets.remove(chat_id);
        }
    }
}

/// Extension trait for RateLimiter to support priority tiers
pub trait RateLimiterExt {
    /// Check if a chat with specified priority can make a request
    /// Higher priority chats get more tokens
    fn check_with_priority(
        &self,
        chat_id: &ChatId,
        priority: u8,
    ) -> impl std::future::Future<Output = bool> + Send;
}

impl RateLimiterExt for RateLimiter {
    fn check_with_priority(
        &self,
        chat_id: &ChatId,
        priority: u8,
    ) -> impl std::future::Future<Output = bool> + Send {
        async move {
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
}
