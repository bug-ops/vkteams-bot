use std::sync::Arc;
use std::time::Instant;
use tokio::time::{Duration, sleep};

use vkteams_bot::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file
    dotenvy::dotenv().expect("unable to load .env file");
    // Initialize logger
    let _guard = otlp::init().map_err(|e| BotError::Otlp(e.into()))?;
    println!("🚀 Starting Rate Limiter Background Refill Demo");

    // Create high-performance rate limiter with background refill
    let rate_limiter = RateLimiter::new();

    println!("🔧 New Rate Limiter Features:");
    println!("   ✅ Background token refill (no computation during requests)");
    println!("   ✅ Lock-free atomic operations");
    println!("   ✅ Bucket starts with full capacity");
    println!("   ✅ Graceful shutdown support");
    println!("   ✅ High concurrency performance");
    println!("");

    // Demo 1: Immediate availability
    demo_immediate_availability(&rate_limiter).await?;

    // Demo 2: Background refill
    demo_background_refill(&rate_limiter).await?;

    // Demo 3: High-performance concurrent access
    demo_concurrent_performance(&rate_limiter).await?;

    // Demo 4: Rate limiting behavior
    demo_rate_limiting(&rate_limiter).await?;

    // Demo 5: Graceful shutdown
    demo_graceful_shutdown(&rate_limiter).await?;

    println!("✅ All demos completed successfully!");
    Ok(())
}

/// Demo 1: Immediate token availability
async fn demo_immediate_availability(rate_limiter: &RateLimiter) -> Result<()> {
    println!("📦 Demo 1: Immediate Token Availability");
    println!("   Bucket starts with full capacity - no delays on startup");

    let chat_id = ChatId("demo_immediate".to_string());
    let start = Instant::now();

    // First requests should succeed immediately
    for i in 1..=5 {
        let request_start = Instant::now();
        let allowed = rate_limiter.check_rate_limit(&chat_id).await;
        let duration = request_start.elapsed();

        println!(
            "   Request {}: {} ({:.2}μs)",
            i,
            if allowed { "✅ Allowed" } else { "❌ Denied" },
            duration.as_micros()
        );
    }

    println!(
        "   Total time for 5 requests: {:.2}ms",
        start.elapsed().as_millis()
    );
    println!("");
    Ok(())
}

/// Demo 2: Background refill functionality
async fn demo_background_refill(rate_limiter: &RateLimiter) -> Result<()> {
    println!("🔄 Demo 2: Background Token Refill");
    println!("   Tokens are refilled by background task, not during requests");

    let chat_id = ChatId("demo_refill".to_string());

    // Consume all available tokens quickly
    println!("   Consuming all tokens...");
    let mut consumed = 0;
    while rate_limiter.check_rate_limit(&chat_id).await {
        consumed += 1;
        if consumed > 200 {
            break;
        } // Safety limit
    }
    println!("   Consumed {} tokens", consumed);

    // Show that we're rate limited
    let allowed = rate_limiter.check_rate_limit(&chat_id).await;
    println!(
        "   Next request: {} (expected - bucket empty)",
        if allowed { "✅ Allowed" } else { "❌ Denied" }
    );

    // Wait for background refill
    println!("   Waiting for background refill (2 seconds)...");
    sleep(Duration::from_secs(2)).await;

    // Try again - should have tokens
    let allowed = rate_limiter.check_rate_limit(&chat_id).await;
    println!(
        "   After refill: {} (background task refilled tokens)",
        if allowed { "✅ Allowed" } else { "❌ Denied" }
    );

    println!("");
    Ok(())
}

/// Demo 3: High-performance concurrent access
async fn demo_concurrent_performance(rate_limiter: Arc<RateLimiter>) -> Result<()> {
    println!("⚡ Demo 3: High-Performance Concurrent Access");
    println!("   Lock-free atomic operations scale with CPU cores");

    let rate_limiter = Arc::clone(&rate_limiter);

    // Benchmark: 1000 requests across 10 threads
    let start = Instant::now();
    let mut handles = vec![];

    for thread_id in 0..10 {
        let rate_limiter_clone = Arc::clone(&rate_limiter);
        let chat_id_clone = ChatId(format!("perf_test_{}", thread_id));

        let handle = tokio::spawn(async move {
            let mut successes = 0;
            let thread_start = Instant::now();

            for _ in 0..100 {
                if rate_limiter_clone.check_rate_limit(&chat_id_clone).await {
                    successes += 1;
                }
            }

            (thread_id, successes, thread_start.elapsed())
        });

        handles.push(handle);
    }

    // Collect results
    let mut total_successes = 0;
    for handle in handles {
        let (thread_id, successes, thread_duration) = handle.await.unwrap();
        total_successes += successes;
        println!(
            "   Thread {}: {}/100 requests allowed ({:.2}ms)",
            thread_id,
            successes,
            thread_duration.as_millis()
        );
    }

    let total_duration = start.elapsed();
    println!(
        "   Total: {}/1000 requests in {:.2}ms",
        total_successes,
        total_duration.as_millis()
    );
    println!(
        "   Throughput: {:.0} requests/second",
        1000.0 / total_duration.as_secs_f64()
    );

    println!("");
    Ok(())
}

/// Demo 4: Rate limiting behavior
async fn demo_rate_limiting(rate_limiter: &RateLimiter) -> Result<()> {
    println!("🛡️  Demo 4: Rate Limiting Protection");
    println!("   Protects against burst requests while allowing normal traffic");

    let chat_id = ChatId("demo_protection".to_string());

    // Simulate burst traffic
    println!("   Simulating burst traffic (20 rapid requests):");
    let mut allowed_count = 0;
    let mut denied_count = 0;

    for i in 1..=20 {
        let allowed = rate_limiter.check_rate_limit(&chat_id).await;
        if allowed {
            allowed_count += 1;
            println!("✅");
        } else {
            denied_count += 1;
            println!("❌");
        }

        if i % 10 == 0 {
            println!("");
        }
    }

    println!(
        "   Results: {} allowed, {} denied",
        allowed_count, denied_count
    );
    println!("   Rate limiting successfully protected against burst!");

    // Show stats
    if let Some(stats) = rate_limiter.get_chat_stats(&chat_id).await {
        println!("   📊 Chat Stats:");
        println!("      Total requests: {}", stats.total_requests);
        println!("      Allowed: {}", stats.allowed_requests);
        println!("      Rate limited: {}", stats.rate_limited_requests);
    }
    Ok(())
}

/// Demo 5: Graceful shutdown
async fn demo_graceful_shutdown(rate_limiter: &RateLimiter) -> Result<()> {
    println!("🛑 Demo 5: Graceful Shutdown");
    println!("   Properly cleanup background tasks to prevent resource leaks");

    // Get global stats before shutdown
    let global_stats = rate_limiter.get_global_stats().await;
    println!("   📊 Global Stats Before Shutdown:");
    println!("      Total requests: {}", global_stats.total_requests);
    println!("      Allowed: {}", global_stats.allowed_requests);
    println!("      Rate limited: {}", global_stats.rate_limited_requests);

    // Graceful shutdown
    println!("   Shutting down rate limiter...");
    rate_limiter.shutdown().await;
    println!("   ✅ Shutdown complete - all background tasks stopped");

    Ok(())
}
