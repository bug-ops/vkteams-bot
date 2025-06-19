use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::sync::Arc;
use tokio::runtime::Runtime;
use vkteams_bot::bot::ratelimit::{LockFreeTokenBucket, RateLimiter};
use vkteams_bot::prelude::ChatId;

/// Benchmark lock-free token bucket operations
fn bench_lockfree_token_bucket(c: &mut Criterion) {
    let mut group = c.benchmark_group("lockfree_token_bucket");

    // Test different bucket sizes
    for capacity in [10, 100, 1000].iter() {
        let bucket = LockFreeTokenBucket::new(*capacity, *capacity / 10);

        group.bench_with_input(
            BenchmarkId::new("try_consume", capacity),
            capacity,
            |b, _| {
                b.iter(|| {
                    black_box(bucket.try_consume());
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("available_tokens", capacity),
            capacity,
            |b, _| {
                b.iter(|| {
                    black_box(bucket.available_tokens());
                });
            },
        );

        group.bench_with_input(BenchmarkId::new("get_stats", capacity), capacity, |b, _| {
            b.iter(|| {
                black_box(bucket.get_stats());
            });
        });
    }

    group.finish();
}

/// Benchmark concurrent token consumption
fn bench_concurrent_consumption(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_consumption");

    for thread_count in [1, 4, 8, 16].iter() {
        let bucket = Arc::new(LockFreeTokenBucket::new(10000, 1000));

        group.bench_with_input(
            BenchmarkId::new("concurrent_try_consume", thread_count),
            thread_count,
            |b, &thread_count| {
                b.to_async(&rt).iter(|| async {
                    let bucket = bucket.clone();
                    let mut handles = vec![];

                    for _ in 0..thread_count {
                        let bucket_clone = bucket.clone();
                        handles.push(tokio::spawn(async move {
                            for _ in 0..100 {
                                black_box(bucket_clone.try_consume());
                            }
                        }));
                    }

                    for handle in handles {
                        handle.await.unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark rate limiter operations
fn bench_rate_limiter_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("rate_limiter_operations");

    let limiter = RateLimiter::new();
    let chat_ids: Vec<ChatId> = (0..100).map(|i| ChatId(format!("chat_{}", i))).collect();

    group.bench_function("check_rate_limit_single_chat", |b| {
        let chat_id = &chat_ids[0];
        b.to_async(&rt).iter(|| async {
            black_box(limiter.check_rate_limit(chat_id).await);
        });
    });

    group.bench_function("check_rate_limit_multiple_chats", |b| {
        b.to_async(&rt).iter(|| async {
            for chat_id in &chat_ids[0..10] {
                black_box(limiter.check_rate_limit(chat_id).await);
            }
        });
    });

    group.bench_function("get_global_stats", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(limiter.get_global_stats().await);
        });
    });

    group.bench_function("get_available_tokens", |b| {
        let chat_id = &chat_ids[0];
        b.to_async(&rt).iter(|| async {
            black_box(limiter.get_available_tokens(chat_id).await);
        });
    });

    group.finish();
}

/// Benchmark memory cleanup operations
fn bench_cleanup_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("cleanup_operations");

    group.bench_function("proactive_cleanup", |b| {
        b.to_async(&rt).iter(|| async {
            let limiter = RateLimiter::new();

            // Create many buckets
            for i in 0..1000 {
                let chat_id = ChatId(format!("chat_{}", i));
                limiter.check_rate_limit(&chat_id).await;
            }

            // Trigger cleanup
            let dummy_chat = ChatId("cleanup_trigger".to_string());
            black_box(limiter.check_rate_limit(&dummy_chat).await);
        });
    });

    group.finish();
}

/// Benchmark compared to naive mutex-based implementation
fn bench_comparison_with_mutex(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("lockfree_vs_mutex");

    // Simulate a simple mutex-based rate limiter for comparison
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MutexBasedLimiter {
        buckets: Mutex<HashMap<String, u32>>,
    }

    impl MutexBasedLimiter {
        fn new() -> Self {
            Self {
                buckets: Mutex::new(HashMap::new()),
            }
        }

        fn check_rate_limit(&self, chat_id: &str) -> bool {
            let mut buckets = self.buckets.lock().unwrap();
            let tokens = buckets.entry(chat_id.to_string()).or_insert(100);
            if *tokens > 0 {
                *tokens -= 1;
                true
            } else {
                false
            }
        }
    }

    let lockfree_limiter = RateLimiter::new();
    let mutex_limiter = MutexBasedLimiter::new();
    let chat_id = ChatId("bench_chat".to_string());

    group.bench_function("lockfree_implementation", |b| {
        b.to_async(&rt).iter(|| async {
            for _ in 0..100 {
                black_box(lockfree_limiter.check_rate_limit(&chat_id).await);
            }
        });
    });

    group.bench_function("mutex_implementation", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(mutex_limiter.check_rate_limit("bench_chat"));
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_lockfree_token_bucket,
    bench_concurrent_consumption,
    bench_rate_limiter_operations,
    bench_cleanup_operations,
    bench_comparison_with_mutex
);
criterion_main!(benches);
