use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tokio::runtime::Runtime;
use vkteams_bot::Bot;
use vkteams_bot::prelude::*;

#[global_allocator]
static DHAT: dhat::Alloc = dhat::Alloc;

fn benchmark_bot_creation(c: &mut Criterion) {
    let _profiler = dhat::Profiler::new_heap();
    c.bench_function("bot_creation", |b| {
        b.iter(|| {
            let _bot = Bot::new(black_box(APIVersionUrl::V1));
        });
    });
}

fn benchmark_message_serialization(c: &mut Criterion) {
    let _profiler = dhat::Profiler::new_heap();
    let rt = Runtime::new().unwrap();

    c.bench_function("message_serialization", |b| {
        b.to_async(&rt).iter(|| async {
            let _bot = Bot::new(black_box(APIVersionUrl::V1));

            // Benchmark message preparation (without actual HTTP call)
            let message = black_box("Hello, World! This is a test message with some emoji 🚀🦀✨");
            let chat_id = black_box("test_chat_123");

            // Simulate message preparation overhead
            let prepared = format!("Sending '{}' to '{}'", message, chat_id);
            black_box(prepared)
        })
    });
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let _profiler = dhat::Profiler::new_heap();
    c.bench_function("memory_efficiency", |b| {
        b.iter(|| {
            // Create 1000 bot instances to test memory efficiency
            let bots: Vec<Bot> = (0..1000)
                .map(|_i| Bot::new(black_box(APIVersionUrl::V1)))
                .collect();
            black_box(bots)
        })
    });
}

criterion_group!(
    benches,
    benchmark_bot_creation,
    benchmark_message_serialization,
    benchmark_memory_usage
);
criterion_main!(benches);
