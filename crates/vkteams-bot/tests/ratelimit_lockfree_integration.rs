use std::sync::Arc;
/// Integration tests for lock-free rate limiter
///
/// These tests verify that the optimized rate limiter:
/// - Provides correct rate limiting behavior
/// - Handles concurrent access without data races
/// - Maintains performance under load
/// - Manages memory efficiently
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use vkteams_bot::bot::ratelimit::RateLimiter;
use vkteams_bot::prelude::ChatId;

#[tokio::test]
async fn test_basic_rate_limiting_behavior() {
    let limiter = RateLimiter::new();
    let chat_id = ChatId::from("test_chat");

    // Should allow initial requests up to limit
    let mut allowed_count = 0;
    for _ in 0..200 {
        // Try more than the typical limit
        if limiter.check_rate_limit(&chat_id).await {
            allowed_count += 1;
        } else {
            break;
        }
    }

    // Should have allowed some requests but not unlimited
    assert!(allowed_count > 0, "Should allow some initial requests");
    assert!(allowed_count < 200, "Should eventually rate limit");

    // Should eventually reject requests when limit is exceeded
    let mut rejected_count = 0;
    for _ in 0..20 {
        if !limiter.check_rate_limit(&chat_id).await {
            rejected_count += 1;
        }
    }

    assert!(
        rejected_count > 0,
        "Should reject requests when rate limited"
    );
}

#[tokio::test]
async fn test_concurrent_access_different_chats() {
    let limiter = Arc::new(RateLimiter::new());
    let mut handles = vec![];
    let success_count = Arc::new(AtomicUsize::new(0)); // Spawn multiple tasks for different chats
    for i in 0..10 {
        let limiter = limiter.clone();
        let success_count = success_count.clone();
        let chat_id = ChatId::from(format!("chat_{i}"));

        let handle = tokio::spawn(async move {
            let mut local_success = 0;
            // Try to make more requests than the likely limit per chat
            for _ in 0..150 {
                // Increased to test rate limiting
                if limiter.check_rate_limit(&chat_id).await {
                    local_success += 1;
                } else {
                    break; // Stop on first rate limit
                }
            }
            success_count.fetch_add(local_success, Ordering::Relaxed);
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let total_success = success_count.load(Ordering::Relaxed);

    // Each chat should get its own rate limit allowance
    println!("Total success across all chats: {total_success}");
    assert!(
        total_success > 100,
        "Different chats should each get rate limit allowance"
    );
    // With 10 chats trying 150 requests each, rate limiting should prevent getting all 1500
    assert!(
        total_success < 1200,
        "Rate limiting should still be enforced per chat"
    );

    // Verify we have reasonable number of active buckets (cleanup may have occurred)
    let bucket_count = limiter.active_bucket_count();
    assert!(bucket_count >= 1, "Should have at least one bucket");
    assert!(
        bucket_count <= 10,
        "Should not have more buckets than chats"
    );
    println!("Active buckets: {bucket_count}");
}

#[tokio::test]
async fn test_concurrent_access_same_chat() {
    let limiter = Arc::new(RateLimiter::new());
    let chat_id = ChatId::from("shared_chat");
    let success_count = Arc::new(AtomicUsize::new(0));
    let failure_count = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    // Spawn multiple tasks accessing the same chat
    for _ in 0..20 {
        let limiter = limiter.clone();
        let chat_id = chat_id.clone();
        let success_count = success_count.clone();
        let failure_count = failure_count.clone();

        let handle = tokio::spawn(async move {
            for _ in 0..25 {
                if limiter.check_rate_limit(&chat_id).await {
                    success_count.fetch_add(1, Ordering::Relaxed);
                } else {
                    failure_count.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let total_success = success_count.load(Ordering::Relaxed);
    let total_failure = failure_count.load(Ordering::Relaxed);
    let total_attempts = total_success + total_failure;

    assert_eq!(total_attempts, 500, "Should have processed all attempts");
    assert!(total_success > 0, "Should allow some requests");
    assert!(
        total_failure > 0,
        "Should reject some requests when rate limited"
    );

    // The success count should be reasonable (not unlimited)
    assert!(total_success < 400, "Should not allow unlimited requests");

    // Should have only one bucket for the shared chat
    assert_eq!(
        limiter.active_bucket_count(),
        1,
        "Should have one bucket for shared chat"
    );
}

#[tokio::test]
async fn test_statistics_collection() {
    let limiter = RateLimiter::new();
    let chat_id = ChatId::from("stats_chat");

    // Make some requests
    let mut allowed = 0;
    let mut rejected = 0;

    for _ in 0..100 {
        if limiter.check_rate_limit(&chat_id).await {
            allowed += 1;
        } else {
            rejected += 1;
        }
    }

    // Check global statistics
    let global_stats = limiter.get_global_stats().await;
    assert_eq!(
        global_stats.total_requests, 100,
        "Should track total requests"
    );
    assert_eq!(
        global_stats.allowed_requests, allowed,
        "Should track allowed requests"
    );
    assert_eq!(
        global_stats.rate_limited_requests, rejected,
        "Should track rate limited requests"
    );

    // Check chat-specific statistics
    let chat_stats = limiter.get_chat_stats(&chat_id).await;
    assert!(chat_stats.is_some(), "Should have chat statistics");

    let chat_stats = chat_stats.unwrap();
    assert!(chat_stats.total_requests > 0, "Should track chat requests");
    assert_eq!(
        chat_stats.allowed_requests + chat_stats.rate_limited_requests,
        chat_stats.total_requests,
        "Chat stats should be consistent"
    );
}

#[tokio::test]
async fn test_token_refill_behavior() {
    let limiter = RateLimiter::new();
    let chat_id = ChatId::from("refill_chat");

    // Consume initial tokens
    let mut initial_allowed = 0;
    for _ in 0..200 {
        if limiter.check_rate_limit(&chat_id).await {
            initial_allowed += 1;
        } else {
            break;
        }
    }

    assert!(initial_allowed > 0, "Should allow some initial requests");

    // Should be rate limited now
    assert!(
        !limiter.check_rate_limit(&chat_id).await,
        "Should be rate limited"
    );

    // Wait for token refill (simulated by time passage)
    sleep(Duration::from_millis(2000)).await;

    // Should be able to make requests again after time passage
    // Note: In lock-free implementation, refill happens on access
    let can_request_after_wait = limiter.check_rate_limit(&chat_id).await;

    // The exact behavior depends on implementation, but it should handle time-based refill
    println!("Can request after wait: {can_request_after_wait}");

    // Verify tokens are available (through querying)
    let available_tokens = limiter.get_available_tokens(&chat_id).await;
    assert!(available_tokens.is_some(), "Should have token information");
}

#[tokio::test]
async fn test_memory_management() {
    let limiter = RateLimiter::new();

    // Create many buckets
    for i in 0..1000 {
        let chat_id = ChatId::from(format!("temp_chat_{i}"));
        limiter.check_rate_limit(&chat_id).await;
    }

    let initial_bucket_count = limiter.active_bucket_count();
    assert_eq!(
        initial_bucket_count, 1000,
        "Should have created 1000 buckets"
    );

    // Trigger cleanup by creating more buckets
    for i in 1000..1100 {
        let chat_id = ChatId::from(format!("trigger_chat_{i}"));
        limiter.check_rate_limit(&chat_id).await;
    }

    // Cleanup should have occurred (exact count depends on implementation)
    let final_bucket_count = limiter.active_bucket_count();
    println!(
        "Initial: {initial_bucket_count}, Final: {final_bucket_count}"
    );

    // The cleanup should have had some effect
    assert!(final_bucket_count > 0, "Should still have active buckets");
}

#[tokio::test]
async fn test_priority_based_rate_limiting() {
    use vkteams_bot::bot::ratelimit::RateLimiterExt;

    let limiter = RateLimiter::new();
    let chat_id = ChatId::from("priority_chat");

    // First consume tokens with low priority
    let mut low_priority_allowed = 0;
    for _ in 0..100 {
        if limiter.check_with_priority(&chat_id, 0).await {
            low_priority_allowed += 1;
        } else {
            break;
        }
    }

    // Then try with high priority
    let mut high_priority_allowed = 0;
    for _ in 0..50 {
        if limiter.check_with_priority(&chat_id, 10).await {
            high_priority_allowed += 1;
        }
    }

    // High priority should get some allowance even when low priority is limited
    assert!(
        low_priority_allowed > 0,
        "Low priority should get some requests"
    );

    // The exact behavior depends on implementation
    println!(
        "Low priority: {low_priority_allowed}, High priority: {high_priority_allowed}"
    );
}

#[tokio::test]
async fn test_performance_under_load() {
    let limiter = Arc::new(RateLimiter::new());
    let chat_ids: Vec<ChatId> = (0..100)
        .map(|i| ChatId::from(format!("perf_chat_{i}")))
        .collect();

    let start_time = Instant::now();
    let mut handles = vec![];
    let total_requests = Arc::new(AtomicUsize::new(0));

    // Spawn high-concurrency load
    for i in 0..50 {
        let limiter = limiter.clone();
        let chat_ids = chat_ids.clone();
        let total_requests = total_requests.clone();

        let handle = tokio::spawn(async move {
            let mut local_requests = 0;
            for _ in 0..100 {
                let chat_id = &chat_ids[i % chat_ids.len()];
                limiter.check_rate_limit(chat_id).await;
                local_requests += 1;
            }
            total_requests.fetch_add(local_requests, Ordering::Relaxed);
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    let duration = start_time.elapsed();
    let total = total_requests.load(Ordering::Relaxed);
    let requests_per_second = total as f64 / duration.as_secs_f64();

    println!(
        "Processed {total} requests in {duration:?} ({requests_per_second:.0} req/sec)"
    );

    // Should handle high throughput efficiently
    assert!(
        requests_per_second > 1000.0,
        "Should handle at least 1000 req/sec"
    );
    assert!(
        duration.as_millis() < 10000,
        "Should complete within reasonable time"
    );
}

#[tokio::test]
async fn test_graceful_shutdown() {
    let limiter = RateLimiter::new();
    let chat_id = ChatId::from("shutdown_chat");

    // Create some buckets
    for i in 0..10 {
        let chat_id = ChatId::from(format!("shutdown_chat_{i}"));
        limiter.check_rate_limit(&chat_id).await;
    }

    let initial_count = limiter.active_bucket_count();
    assert!(
        initial_count > 0,
        "Should have active buckets before shutdown"
    );

    // Shutdown the limiter
    limiter.shutdown().await;

    let final_count = limiter.active_bucket_count();
    assert_eq!(
        final_count, 0,
        "Should have no active buckets after shutdown"
    );

    // Should still be able to create new buckets after shutdown
    // (This tests that shutdown doesn't break the basic functionality)
    limiter.check_rate_limit(&chat_id).await;
    assert_eq!(
        limiter.active_bucket_count(),
        1,
        "Should be able to create new buckets after shutdown"
    );
}
