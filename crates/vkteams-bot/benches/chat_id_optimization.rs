use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use vkteams_bot::prelude::ChatId;

fn benchmark_chat_id_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ChatId Creation");

    // Benchmark creating ChatId from static str (zero allocation)
    group.bench_function("from_static_str", |b| {
        b.iter(|| {
            let chat_id = ChatId::from_static(black_box("static_chat_id"));
            black_box(chat_id)
        })
    });

    // Benchmark creating ChatId from String (requires allocation)
    group.bench_function("from_string", |b| {
        b.iter(|| {
            let chat_id = ChatId::from(black_box("dynamic_chat_id".to_string()));
            black_box(chat_id)
        })
    });

    // Benchmark creating ChatId from static literal via From trait (zero allocation)
    group.bench_function("from_static_literal", |b| {
        b.iter(|| {
            let chat_id = ChatId::from(black_box("static_literal"));
            black_box(chat_id)
        })
    });

    // Benchmark creating ChatId from &str (requires allocation)
    group.bench_function("from_str_ref", |b| {
        b.iter(|| {
            let dynamic_str = black_box("str_ref_chat_id");
            let chat_id = ChatId::from_borrowed_str(dynamic_str);
            black_box(chat_id)
        })
    });

    group.finish();
}

fn benchmark_chat_id_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("ChatId Clone");

    // Static ChatId (zero-copy clone)
    let static_chat_id = ChatId::from_static("static_chat_id");
    group.bench_function("clone_static", |b| {
        b.iter(|| {
            let cloned = black_box(&static_chat_id).clone();
            black_box(cloned)
        })
    });

    // Owned ChatId (requires allocation on clone)
    let owned_chat_id = ChatId::from("owned_chat_id".to_string());
    group.bench_function("clone_owned", |b| {
        b.iter(|| {
            let cloned = black_box(&owned_chat_id).clone();
            black_box(cloned)
        })
    });

    group.finish();
}

fn benchmark_chat_id_usage_in_map(c: &mut Criterion) {
    use std::collections::HashMap;

    let mut group = c.benchmark_group("ChatId Map Operations");

    // Test with static ChatIds (optimal for rate limiter scenario - repeated chat IDs)
    group.bench_function("map_insert_static", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..100 {
                // Simulate rate limiter scenario: few repeated chat IDs (zero allocation)
                let chat_id = if i % 2 == 0 {
                    ChatId::from_static("chat_even")
                } else {
                    ChatId::from_static("chat_odd")
                };
                map.insert(black_box(chat_id), black_box(i));
            }
            black_box(map)
        })
    });

    // Test with owned ChatIds (old behavior - each ChatId requires allocation)
    group.bench_function("map_insert_owned", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..100 {
                // Every ChatId creation requires heap allocation (старое поведение)
                let chat_id = ChatId::from(format!("unique_chat_{}", i));
                map.insert(black_box(chat_id), black_box(i));
            }
            black_box(map)
        })
    });

    // Test HashMap lookups (rate limiter scenario - checking existing entries)
    group.bench_function("map_lookup_static", |b| {
        let mut map = HashMap::new();
        let static_chat_id = ChatId::from_static("frequent_chat");
        map.insert(static_chat_id.clone(), 42);

        b.iter(|| {
            // Simulate rate limiter checking: clone for lookup (should be fast for static)
            let lookup_key = black_box(static_chat_id.clone());
            let result = map.get(&lookup_key);
            black_box(result)
        })
    });

    group.bench_function("map_lookup_owned", |b| {
        let mut map = HashMap::new();
        let owned_chat_id = ChatId::from("frequent_chat".to_string());
        map.insert(owned_chat_id.clone(), 42);

        b.iter(|| {
            // Simulate rate limiter checking: clone for lookup (requires allocation for owned)
            let lookup_key = black_box(owned_chat_id.clone());
            let result = map.get(&lookup_key);
            black_box(result)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_chat_id_creation,
    benchmark_chat_id_clone,
    benchmark_chat_id_usage_in_map
);
criterion_main!(benches);
