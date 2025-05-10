use crate::config::CONFIG;
use crate::prelude::ChatId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};
use tokio::time::{Duration, Instant, sleep};
use tracing::{debug, warn};
/// ### Structure for storing bucket information per chat_id
///
/// A bucket is a semaphore that replenishes every second and allows a certain number of requests (limit) within a specified time period (duration)
/// - When the request limit is exceeded, the bucket does not allow obtaining a request permit
/// - When the request limit is exceeded, you can try to obtain a permit several times with a delay (retry_delay)
/// - When the request limit is exceeded, you can wait until free permits become available (retry_attempts)
/// - When the bucket is replenished, old permits are removed and new ones can be obtained
#[derive(Debug)]
struct ChatBucket {
    semaphore: Arc<Semaphore>,
    last_replenish: Arc<Mutex<Instant>>,
    active_permits: Arc<Mutex<Vec<OwnedSemaphorePermit>>>,
}

impl ChatBucket {
    #[tracing::instrument]
    fn new() -> Self {
        let cfg = &CONFIG.rate_limit;
        debug!("Creating new chat bucket with limit: {}", cfg.limit);
        Self {
            semaphore: Arc::new(Semaphore::new(cfg.limit)),
            last_replenish: Arc::new(Mutex::new(Instant::now())),
            active_permits: Arc::new(Mutex::new(Vec::new())),
        }
    }
    /// ### Obtaining a permit from the semaphore
    ///
    /// - Replenishes the semaphore every second
    /// - When the request limit is exceeded, does not allow obtaining a permit
    /// - When the semaphore is replenished, removes old permits
    /// - Returns `true` if the permit was obtained
    /// - Returns `false` if the permit was not obtained
    #[tracing::instrument(skip(self))]
    async fn acquire(&mut self) -> bool {
        let cfg = &CONFIG.rate_limit;
        let now = Instant::now();

        // Check and update the last replenishment time
        {
            let mut last_replenish = self.last_replenish.lock().await;
            let elapsed = now.duration_since(*last_replenish);

            if elapsed >= Duration::from_secs(cfg.duration) {
                debug!(
                    "Replenishing permits in semaphore from {:?}",
                    *last_replenish
                );
                *last_replenish = now;

                // Clear old permits and replenish the semaphore
                let mut active_permits = self.active_permits.lock().await;
                active_permits.drain(..).for_each(|permit| permit.forget());
                self.semaphore.add_permits(cfg.limit);
            }
        }

        // Try to obtain a permit
        match Arc::clone(&self.semaphore).try_acquire_owned() {
            Ok(permit) => {
                debug!(
                    "Acquired permit from semaphore. Available permits: {}",
                    self.semaphore.available_permits()
                );
                // Save the permit to the collection of active permits
                let mut active_permits = self.active_permits.lock().await;
                active_permits.push(permit);
                true
            }
            Err(_) => {
                warn!(
                    "No permits available in semaphore. Available permits: {}",
                    self.semaphore.available_permits()
                );
                false
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    chat_buckets: Arc<Mutex<HashMap<ChatId, ChatBucket>>>,
    last_request: Instant,
    min_interval: Duration,
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
            chat_buckets: Arc::new(Mutex::new(HashMap::new())),
            last_request: Instant::now(),
            min_interval: Duration::from_secs(CONFIG.rate_limit.duration),
        }
    }
    /// ### Check request limit for `chat_id`
    ///
    /// - Checks the request limit for `chat_id`
    /// - Returns `true` if the limit is not exceeded
    /// - Returns `false` if the limit is exceeded
    #[tracing::instrument(skip(self))]
    pub async fn check_rate_limit(&self, chat_id: ChatId) -> bool {
        let chat_buckets = self.chat_buckets.clone();
        let mut buckets = chat_buckets.lock().await;
        let bucket = buckets.entry(chat_id.clone()).or_insert_with(|| {
            debug!("Creating new chat bucket for chat_id: {}", chat_id.0);
            ChatBucket::new()
        });
        debug!("Checking rate limit for chat_id: {}", chat_id.0);
        bucket.acquire().await
    }
    /// ### Check and wait if request limit is exceeded
    ///
    /// - Checks the request limit for `chat_id`
    /// - If the limit is exceeded, waits until free permits become available
    /// - Retries several times with a delay
    /// - Returns `true` if the limit is not exceeded
    /// - Returns `false` if the limit is exceeded and a permit could not be obtained
    #[tracing::instrument(skip(self))]
    pub async fn wait_if_needed(&mut self, chat_id: Option<&ChatId>) -> bool {
        if let Some(chat_id) = chat_id {
            let cfg = &CONFIG.rate_limit;
            let mut attempts = 0;
            let retry_delay = Duration::from_millis(cfg.retry_delay);

            while attempts < cfg.retry_attempts {
                if self.check_rate_limit(chat_id.clone()).await {
                    debug!("Rate limit not exceeded for chat_id: {}", chat_id.0);
                    return true;
                }
                attempts += 1;
                debug!(
                    "Rate limit exceeded, attempt {}/{}",
                    attempts, cfg.retry_attempts
                );
                sleep(retry_delay).await;
            }
            warn!(
                "Rate limit exceeded after {} attempts for chat_id: {}",
                attempts, chat_id.0
            );
            false
        } else {
            true
        }
    }

    pub async fn wait_if_needed_new(&mut self, chat_id: Option<&ChatId>) -> bool {
        if let Some(_chat_id) = chat_id {
            let elapsed = self.last_request.elapsed();
            if elapsed < self.min_interval {
                tokio::time::sleep(self.min_interval - elapsed).await;
            }
            self.last_request = Instant::now();
        }
        true
    }
}
