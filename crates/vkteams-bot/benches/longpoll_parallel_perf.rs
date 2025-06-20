use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::time::sleep;
use vkteams_bot::prelude::*;

// Mock bot responses for benchmarking
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

// Mock processor function that simulates work
async fn mock_processor(_bot: Bot, events: ResponseEventsGet) -> Result<()> {
    // Simulate some processing time proportional to event count
    let processing_time = events.events.len() as u64 * 100; // 100µs per event
    sleep(Duration::from_micros(processing_time)).await;
    Ok(())
}

// Counter for tracking function calls
static CALL_COUNTER: AtomicUsize = AtomicUsize::new(0);

async fn counting_processor(_bot: Bot, events: ResponseEventsGet) -> Result<()> {
    CALL_COUNTER.fetch_add(events.events.len(), Ordering::SeqCst);
    // Simulate minimal processing time
    sleep(Duration::from_micros(50)).await;
    Ok(())
}

fn bench_event_response_creation(c: &mut Criterion) {
    c.bench_function("create_event_response_small", |b| {
        b.iter(|| {
            black_box(create_test_events(10));
        });
    });

    c.bench_function("create_event_response_medium", |b| {
        b.iter(|| {
            black_box(create_test_events(100));
        });
    });

    c.bench_function("create_event_response_large", |b| {
        b.iter(|| {
            black_box(create_test_events(1000));
        });
    });
}

fn bench_event_processing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("process_small_batch", |b| {
        b.to_async(&rt).iter(|| async {
            let bot = Bot::with_params(&APIVersionUrl::V1, "dummy", "https://dummy.com").unwrap();
            let events = create_test_events(10);
            mock_processor(black_box(bot), black_box(events))
                .await
                .unwrap();
        });
    });

    c.bench_function("process_medium_batch", |b| {
        b.to_async(&rt).iter(|| async {
            let bot = Bot::with_params(&APIVersionUrl::V1, "dummy", "https://dummy.com").unwrap();
            let events = create_test_events(100);
            mock_processor(black_box(bot), black_box(events))
                .await
                .unwrap();
        });
    });

    c.bench_function("process_large_batch", |b| {
        b.to_async(&rt).iter(|| async {
            let bot = Bot::with_params(&APIVersionUrl::V1, "dummy", "https://dummy.com").unwrap();
            let events = create_test_events(1000);
            mock_processor(black_box(bot), black_box(events))
                .await
                .unwrap();
        });
    });
}

fn bench_concurrent_processing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("concurrent_small_batches", |b| {
        b.to_async(&rt).iter(|| async {
            let bot = Bot::with_params(&APIVersionUrl::V1, "dummy", "https://dummy.com").unwrap();
            let futures: Vec<_> = (0..4)
                .map(|_| {
                    let bot = bot.clone();
                    async move {
                        let events = create_test_events(25);
                        counting_processor(bot, events).await
                    }
                })
                .collect();

            // Use futures::future::join_all for concurrent execution
            let results: Vec<Result<()>> = futures::future::join_all(futures).await;

            // Verify all succeeded
            for result in results {
                result.unwrap();
            }
        });
    });

    c.bench_function("sequential_equivalent", |b| {
        b.to_async(&rt).iter(|| async {
            let bot = Bot::with_params(&APIVersionUrl::V1, "dummy", "https://dummy.com").unwrap();
            for _ in 0..4 {
                let events = create_test_events(25);
                counting_processor(bot.clone(), events).await.unwrap();
            }
        });
    });
}

fn bench_event_cloning(c: &mut Criterion) {
    let events_small = create_test_events(10);
    let events_medium = create_test_events(100);
    let events_large = create_test_events(1000);

    c.bench_function("clone_events_small", |b| {
        b.iter(|| {
            black_box(events_small.clone());
        });
    });

    c.bench_function("clone_events_medium", |b| {
        b.iter(|| {
            black_box(events_medium.clone());
        });
    });

    c.bench_function("clone_events_large", |b| {
        b.iter(|| {
            black_box(events_large.clone());
        });
    });
}

fn bench_event_batching(c: &mut Criterion) {
    c.bench_function("batch_events_by_chunks", |b| {
        b.iter(|| {
            let events = create_test_events(1000);
            let batches: Vec<_> = events
                .events
                .chunks(50)
                .map(|chunk| ResponseEventsGet {
                    events: chunk.to_vec(),
                })
                .collect();
            black_box(batches);
        });
    });

    c.bench_function("batch_events_manual_split", |b| {
        b.iter(|| {
            let events = create_test_events(1000);
            let mut batches = Vec::new();
            let batch_size = 50;

            for i in (0..events.events.len()).step_by(batch_size) {
                let end = std::cmp::min(i + batch_size, events.events.len());
                batches.push(ResponseEventsGet {
                    events: events.events[i..end].to_vec(),
                });
            }
            black_box(batches);
        });
    });
}

criterion_group!(
    benches,
    bench_event_response_creation,
    bench_event_processing,
    bench_concurrent_processing,
    bench_event_cloning,
    bench_event_batching
);
criterion_main!(benches);
