use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;
use tempfile::tempdir;
use tokio::fs;
use tokio::runtime::Runtime;

use vkteams_bot_cli::config::{AsyncConfigManager, Config, LockFreeConfigCache};

// Benchmark configuration loading
fn bench_config_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_loading");
    group.throughput(Throughput::Elements(1));

    // Setup test config content
    let test_config = r#"
[api]
token = "benchmark_token"
url = "https://api.icq.net"
timeout = 30
max_retries = 5

[files]
download_dir = "/tmp/downloads"
upload_dir = "/tmp/uploads"
max_file_size = 104857600
buffer_size = 65536

[logging]
level = "info"
format = "text"
colors = true

[ui]
show_progress = true
progress_style = "unicode"
progress_refresh_rate = 100

[rate_limit]
enabled = true
limit = 1000
duration = 60
retry_delay = 500
retry_attempts = 3
"#;

    // Benchmark synchronous config loading
    group.bench_function("sync_from_path", |b| {
        b.iter_batched(
            || {
                let temp_dir = tempdir().unwrap();
                let config_path = temp_dir.path().join("config.toml");
                std::fs::write(&config_path, test_config).unwrap();
                config_path
            },
            |config_path| black_box(Config::from_path(&config_path).unwrap()),
            BatchSize::SmallInput,
        )
    });

    // Benchmark asynchronous config loading
    group.bench_function("async_from_path", |b| {
        let rt = Runtime::new().unwrap();

        b.iter_batched(
            || {
                let temp_dir = tempdir().unwrap();
                let config_path = temp_dir.path().join("config.toml");
                rt.block_on(async {
                    fs::write(&config_path, test_config).await.unwrap();
                });
                config_path
            },
            |config_path| {
                rt.block_on(async {
                    black_box(Config::from_path_async(&config_path).await.unwrap())
                })
            },
            BatchSize::SmallInput,
        )
    });

    // Benchmark cached async config loading
    group.bench_function("async_cached_loading", |b| {
        let rt = Runtime::new().unwrap();

        b.iter_batched(
            || {
                let temp_dir = tempdir().unwrap();
                let config_path = temp_dir.path().join("config.toml");
                rt.block_on(async {
                    fs::write(&config_path, test_config).await.unwrap();
                });

                let mut manager = AsyncConfigManager::new(Duration::from_secs(300));
                manager.config_paths = vec![config_path];

                // Pre-load cache
                rt.block_on(async {
                    manager.load_config().await.unwrap();
                });

                manager
            },
            |manager| rt.block_on(async { black_box(manager.load_config().await.unwrap()) }),
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

// Benchmark config merging performance
fn bench_config_merging(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_merging");
    group.throughput(Throughput::Elements(1));

    // Create base and overlay configs
    let mut base_config = Config::default();
    base_config.api.token = Some("base_token".to_string());
    base_config.api.timeout = 30;
    base_config.logging.level = "info".to_string();
    base_config.files.max_file_size = 104857600;

    let mut overlay_config = Config::default();
    overlay_config.api.token = Some("overlay_token".to_string());
    overlay_config.api.timeout = 60;
    overlay_config.files.buffer_size = 32768;

    // Benchmark optimized merge_configs_efficient method
    group.bench_function("efficient_merge", |b| {
        b.iter(|| {
            black_box(AsyncConfigManager::merge_configs_efficient(
                base_config.clone(),
                overlay_config.clone(),
            ))
        })
    });

    group.finish();
}

// Benchmark lock-free cache performance
fn bench_lockfree_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("lockfree_cache");
    group.throughput(Throughput::Elements(1));

    let rt = Runtime::new().unwrap();
    let cache = LockFreeConfigCache::new(Duration::from_secs(300));

    // Pre-populate cache with some entries
    rt.block_on(async {
        for i in 0..10 {
            let key = format!("config_{i}");
            cache
                .get_or_load(&key, || async { Ok(Config::default()) })
                .await
                .unwrap();
        }
    });

    // Benchmark cache hit performance
    group.bench_function("cache_hit", |b| {
        b.iter(|| {
            rt.block_on(async {
                let key = "config_5"; // Known to be in cache
                black_box(
                    cache
                        .get_or_load(key, || async { Ok(Config::default()) })
                        .await
                        .unwrap(),
                )
            })
        })
    });

    // Benchmark cache miss performance (new keys)
    group.bench_function("cache_miss", |b| {
        let mut counter = 1000;
        b.iter(|| {
            counter += 1;
            let key = format!("new_config_{counter}");
            rt.block_on(async {
                black_box(
                    cache
                        .get_or_load(&key, || async {
                            tokio::time::sleep(Duration::from_micros(10)).await;
                            Ok(Config::default())
                        })
                        .await
                        .unwrap(),
                )
            })
        })
    });

    group.finish();
}

// Benchmark concurrent access patterns
fn bench_concurrent_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_access");
    group.throughput(Throughput::Elements(10));

    let rt = Runtime::new().unwrap();

    // Benchmark concurrent config manager access
    group.bench_function("concurrent_manager", |b| {
        b.iter_batched(
            || {
                let temp_dir = tempdir().unwrap();
                let config_path = temp_dir.path().join("config.toml");
                rt.block_on(async {
                    fs::write(&config_path, "[api]\ntoken = \"test\"\n")
                        .await
                        .unwrap();
                });

                let mut manager = AsyncConfigManager::new(Duration::from_secs(60));
                manager.config_paths = vec![config_path];
                manager
            },
            |manager| {
                rt.block_on(async {
                    let futures = (0..10).map(|_| {
                        let manager_ref = &manager;
                        async move { manager_ref.load_config().await.unwrap() }
                    });

                    let results = futures::future::join_all(futures).await;
                    black_box(results)
                })
            },
            BatchSize::SmallInput,
        )
    });

    // Benchmark concurrent cache access
    group.bench_function("concurrent_cache", |b| {
        b.iter(|| {
            let cache = LockFreeConfigCache::new(Duration::from_secs(60));
            rt.block_on(async {
                let futures = (0..10).map(|i| {
                    let cache_ref = &cache;
                    async move {
                        let key = format!("config_{}", i % 3); // Some overlap for cache hits
                        cache_ref
                            .get_or_load(&key, || async {
                                tokio::time::sleep(Duration::from_micros(1)).await;
                                Ok(Config::default())
                            })
                            .await
                            .unwrap()
                    }
                });

                let results = futures::future::join_all(futures).await;
                black_box(results)
            })
        })
    });

    group.finish();
}

// Benchmark config serialization/deserialization
fn bench_config_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_serialization");
    group.throughput(Throughput::Elements(1));

    // Create a complex config for testing
    let mut config = Config::default();
    config.api.token = Some("complex_token_value_for_benchmarking".to_string());
    config.api.url = Some("https://api.icq.net/bot/v1/".to_string());
    config.api.timeout = 45;
    config.api.max_retries = 5;
    config.files.download_dir = Some("/path/to/downloads".to_string());
    config.files.upload_dir = Some("/path/to/uploads".to_string());
    config.files.max_file_size = 209715200;
    config.files.buffer_size = 131072;
    config.logging.level = "debug".to_string();
    config.logging.format = "json".to_string();
    config.logging.colors = false;

    let serialized_config = toml::to_string_pretty(&config).unwrap();

    // Benchmark serialization
    group.bench_function("serialize_sync", |b| {
        b.iter(|| black_box(toml::to_string_pretty(&config).unwrap()))
    });

    // Benchmark async serialization (spawn_blocking)
    group.bench_function("serialize_async", |b| {
        let rt = Runtime::new().unwrap();
        b.iter(|| {
            let config_clone = config.clone();
            rt.block_on(async {
                black_box(
                    tokio::task::spawn_blocking(move || {
                        toml::to_string_pretty(&config_clone).unwrap()
                    })
                    .await
                    .unwrap(),
                )
            })
        })
    });

    // Benchmark deserialization
    group.bench_function("deserialize_sync", |b| {
        b.iter(|| black_box(toml::from_str::<Config>(&serialized_config).unwrap()))
    });

    // Benchmark async deserialization (spawn_blocking)
    group.bench_function("deserialize_async", |b| {
        let rt = Runtime::new().unwrap();
        b.iter(|| {
            let content = serialized_config.clone();
            rt.block_on(async {
                black_box(
                    tokio::task::spawn_blocking(move || {
                        toml::from_str::<Config>(&content).unwrap()
                    })
                    .await
                    .unwrap(),
                )
            })
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_config_loading,
    bench_config_merging,
    bench_lockfree_cache,
    bench_concurrent_access,
    bench_config_serialization
);
criterion_main!(benches);
