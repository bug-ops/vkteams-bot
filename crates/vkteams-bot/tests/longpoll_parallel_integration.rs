use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use vkteams_bot::prelude::*;

// Integration test for longpoll parallel processing
// Demonstrates the performance benefits of parallel event processing
// Create a mock bot for testing (without requiring environment variables)
fn create_test_bot() -> Bot {
    Bot::with_params(&APIVersionUrl::V1, "test_token", "https://test.api.com").unwrap()
}

// Mock processor that simulates work and tracks events
async fn test_processor_with_counter(
    _bot: Bot,
    events: ResponseEventsGet,
    counter: Arc<AtomicUsize>,
) -> Result<()> {
    let event_count = events.events.len();

    // Simulate processing time (higher for more events)
    let processing_time = event_count as u64 * 50; // 50µs per event
    sleep(Duration::from_micros(processing_time)).await;

    // Track processed events
    counter.fetch_add(event_count, Ordering::SeqCst);

    Ok(())
}

// Create test events for benchmarking
fn create_test_events(count: usize) -> ResponseEventsGet {
    ResponseEventsGet {
        events: (0..count)
            .map(|i| EventMessage {
                event_id: i as u32,
                event_type: EventType::None,
            })
            .collect(),
    }
}

#[tokio::test]
async fn test_parallel_event_processing_performance() {
    let bot = create_test_bot();
    let total_events = 100;
    let batch_size = 10;
    let batches = total_events / batch_size;

    println!(
        "Testing parallel processing of {total_events} events in {batches} batches"
    );

    // Create a counter for this test
    let counter = Arc::new(AtomicUsize::new(0));

    let start_time = Instant::now();

    // Create concurrent tasks for each batch
    let mut tasks = Vec::new();
    for _ in 0..batches {
        let bot_clone = bot.clone();
        let batch_events = create_test_events(batch_size);
        let counter_clone = counter.clone();

        let task = tokio::spawn(async move {
            test_processor_with_counter(bot_clone, batch_events, counter_clone).await
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    for task in tasks {
        task.await.unwrap().unwrap();
    }

    let parallel_duration = start_time.elapsed();
    let processed_count = counter.load(Ordering::SeqCst);

    println!(
        "Parallel processing: {processed_count} events in {parallel_duration:?}"
    );
    assert_eq!(processed_count, total_events);

    // Now test sequential processing for comparison
    let sequential_counter = Arc::new(AtomicUsize::new(0));
    let start_time = Instant::now();

    for _ in 0..batches {
        let batch_events = create_test_events(batch_size);
        test_processor_with_counter(bot.clone(), batch_events, sequential_counter.clone())
            .await
            .unwrap();
    }

    let sequential_duration = start_time.elapsed();
    let sequential_processed = sequential_counter.load(Ordering::SeqCst);

    println!(
        "Sequential processing: {sequential_processed} events in {sequential_duration:?}"
    );
    assert_eq!(sequential_processed, total_events);

    // Parallel should be faster than sequential for our simulated workload
    println!(
        "Performance improvement: {:.2}x faster",
        sequential_duration.as_nanos() as f64 / parallel_duration.as_nanos() as f64
    );

    // For the test workload with artificial delays, parallel should be noticeably faster
    assert!(
        parallel_duration < sequential_duration,
        "Parallel processing should be faster than sequential for this workload"
    );
}

#[tokio::test]
async fn test_large_batch_processing() {
    println!("Testing large batch processing");

    let bot = create_test_bot();
    let large_batch = create_test_events(1000);
    let counter = Arc::new(AtomicUsize::new(0));

    let start_time = Instant::now();
    test_processor_with_counter(bot, large_batch, counter.clone())
        .await
        .unwrap();
    let duration = start_time.elapsed();

    println!("Processed 1000 events in {duration:?}");
    assert_eq!(counter.load(Ordering::SeqCst), 1000);

    // Should complete within reasonable time (adjust based on test environment)
    assert!(
        duration < Duration::from_millis(500),
        "Large batch processing should be efficient"
    );
}

#[tokio::test]
async fn test_event_batching_efficiency() {
    println!("Testing event batching efficiency");

    let events = create_test_events(1000);
    let batch_size = 50;

    let start_time = Instant::now();

    // Test chunking performance
    let batches: Vec<ResponseEventsGet> = events
        .events
        .chunks(batch_size)
        .map(|chunk| ResponseEventsGet {
            events: chunk.to_vec(),
        })
        .collect();

    let batching_duration = start_time.elapsed();

    println!(
        "Created {} batches from 1000 events in {:?}",
        batches.len(),
        batching_duration
    );

    assert_eq!(batches.len(), 20); // 1000 / 50
    assert!(
        batching_duration < Duration::from_millis(10),
        "Batching should be very fast"
    );

    // Verify all events are preserved
    let total_events: usize = batches.iter().map(|b| b.events.len()).sum();
    assert_eq!(total_events, 1000);
}

#[tokio::test]
async fn test_concurrent_bot_operations() {
    println!("Testing concurrent Bot operations");

    let bot = create_test_bot();
    let num_concurrent = 5;
    let events_per_task = 20;
    let counter = Arc::new(AtomicUsize::new(0));

    let start_time = Instant::now();

    // Create multiple concurrent tasks that use the same bot instance
    let tasks: Vec<_> = (0..num_concurrent)
        .map(|i| {
            let bot_clone = bot.clone();
            let counter_clone = counter.clone();
            tokio::spawn(async move {
                let events = create_test_events(events_per_task);
                test_processor_with_counter(bot_clone, events, counter_clone)
                    .await
                    .unwrap();
                i // Return task id for verification
            })
        })
        .collect();

    // Wait for all tasks
    let mut results = Vec::new();
    for task in tasks {
        results.push(task.await.unwrap());
    }

    let duration = start_time.elapsed();

    println!(
        "Completed {num_concurrent} concurrent operations in {duration:?}"
    );

    // Verify all tasks completed
    results.sort();
    assert_eq!(results, (0..num_concurrent).collect::<Vec<_>>());

    // Verify all events were processed
    assert_eq!(
        counter.load(Ordering::SeqCst),
        num_concurrent * events_per_task
    );

    // Should complete efficiently
    assert!(
        duration < Duration::from_millis(200),
        "Concurrent operations should be efficient"
    );
}

#[test]
fn test_event_creation_performance() {
    println!("Testing event creation performance");

    let start = Instant::now();
    let small_events = create_test_events(10);
    let small_duration = start.elapsed();

    let start = Instant::now();
    let medium_events = create_test_events(100);
    let medium_duration = start.elapsed();

    let start = Instant::now();
    let large_events = create_test_events(1000);
    let large_duration = start.elapsed();

    println!("Event creation times:");
    println!("  10 events: {small_duration:?}");
    println!("  100 events: {medium_duration:?}");
    println!("  1000 events: {large_duration:?}");

    assert_eq!(small_events.events.len(), 10);
    assert_eq!(medium_events.events.len(), 100);
    assert_eq!(large_events.events.len(), 1000);

    // Event creation should be very fast
    assert!(small_duration < Duration::from_millis(1));
    assert!(medium_duration < Duration::from_millis(10));
    assert!(large_duration < Duration::from_millis(50));
}
