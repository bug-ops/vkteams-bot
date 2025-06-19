use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;
use tempfile::tempdir;
use tokio::runtime::Runtime;

use vkteams_bot_cli::config::{AsyncConfigManager, Config};

fn simple_config_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let test_config = r#"
[api]
token = "test_token"
timeout = 30
"#;

    c.bench_function("config_sync_load", |b| {
        b.iter(|| {
            let temp_dir = tempdir().unwrap();
            let config_path = temp_dir.path().join("config.toml");
            std::fs::write(&config_path, test_config).unwrap();
            black_box(Config::from_path(&config_path).unwrap())
        })
    });

    c.bench_function("config_async_load", |b| {
        b.iter(|| {
            rt.block_on(async {
                let temp_dir = tempdir().unwrap();
                let config_path = temp_dir.path().join("config.toml");
                tokio::fs::write(&config_path, test_config).await.unwrap();
                black_box(Config::from_path_async(&config_path).await.unwrap())
            })
        })
    });

    c.bench_function("config_cached_load", |b| {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        rt.block_on(async {
            tokio::fs::write(&config_path, test_config).await.unwrap();
        });

        let mut manager = AsyncConfigManager::new(Duration::from_secs(300));
        manager.config_paths = vec![config_path];

        // Pre-warm cache
        rt.block_on(async {
            manager.load_config().await.unwrap();
        });

        b.iter(|| rt.block_on(async { black_box(manager.load_config().await.unwrap()) }))
    });
}

criterion_group!(benches, simple_config_benchmark);
criterion_main!(benches);
