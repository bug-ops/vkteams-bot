use std::time::Duration;
use tempfile::tempdir;
use tokio::fs;
use tokio::time::Instant;

use vkteams_bot_cli::config::{AsyncConfigManager, Config, LockFreeConfigCache};
use vkteams_bot_cli::errors::prelude::Result as CliResult;

/// Test async configuration loading performance
#[tokio::test]
async fn test_async_config_loading_performance() -> CliResult<()> {
    // Create a temporary config file
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let test_config = r#"
[api]
token = "test_token"
url = "https://api.icq.net"
timeout = 30
max_retries = 3

[files]
download_dir = "/tmp/downloads"
upload_dir = "/tmp/uploads"
max_file_size = 104857600
buffer_size = 65536

[logging]
level = "info"
format = "text"
colors = true
"#;

    fs::write(&config_path, test_config).await?;

    // Test async loading performance
    let start = Instant::now();
    let config = Config::from_path_async(&config_path).await?;
    let async_duration = start.elapsed();

    // Verify config content
    assert_eq!(config.api.token, Some("test_token".to_string()));
    assert_eq!(config.api.timeout, 30);
    assert_eq!(config.logging.level, "info");

    println!("Async config loading took: {async_duration:?}");

    // Test that async loading is faster than sync loading (due to non-blocking I/O)
    let start = Instant::now();
    let _sync_config = Config::from_path(&config_path)?;
    let sync_duration = start.elapsed();

    println!("Sync config loading took: {sync_duration:?}");

    // In most cases, async should be comparable or faster
    assert!(async_duration <= sync_duration + Duration::from_millis(50));

    Ok(())
}

/// Test AsyncConfigManager caching functionality
#[tokio::test]
async fn test_async_config_manager_caching() -> CliResult<()> {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let test_config = r#"
[api]
token = "cached_token"
timeout = 60
"#;

    fs::write(&config_path, test_config).await?;

    // Create config manager with short TTL for testing
    let manager = AsyncConfigManager::new(Duration::from_secs(1));

    // First load should hit the file system
    let start = Instant::now();
    let config1 = manager.load_config().await?;
    let first_load_duration = start.elapsed();

    // Second load should hit the cache
    let start = Instant::now();
    let config2 = manager.load_config().await?;
    let cached_load_duration = start.elapsed();

    // Verify both configs are identical
    assert_eq!(config1.api.token, config2.api.token);
    assert_eq!(config1.api.timeout, config2.api.timeout);

    // Cached load should be significantly faster
    assert!(cached_load_duration < first_load_duration);
    assert!(cached_load_duration < Duration::from_millis(10));

    println!("First load: {first_load_duration:?}, Cached load: {cached_load_duration:?}");

    Ok(())
}

/// Test LockFreeConfigCache performance under concurrent access
#[tokio::test]
async fn test_lockfree_cache_concurrent_access() -> CliResult<()> {
    let cache = LockFreeConfigCache::new(Duration::from_secs(10));

    // Test concurrent access
    let start = Instant::now();
    let mut handles = Vec::new();

    for i in 0..50 {
        let cache_clone = cache.clone();
        let handle = tokio::spawn(async move {
            let key = format!("config_{}", i % 5); // Use 5 different keys to test caching
            cache_clone
                .get_or_load(&key, || async {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    Ok(Config::default())
                })
                .await
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let results = futures::future::join_all(handles).await;
    let concurrent_duration = start.elapsed();

    // Verify all tasks completed successfully
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }

    println!("50 concurrent cache operations took: {concurrent_duration:?}");

    // Should complete much faster than 50 * 5ms = 250ms due to caching
    assert!(concurrent_duration < Duration::from_millis(100));

    // Check cache stats
    let (cache_size, timestamp_size) = cache.stats();
    assert_eq!(cache_size, 5); // Should have 5 different keys cached
    assert_eq!(timestamp_size, 5);

    Ok(())
}

/// Test async config save and load round-trip
#[tokio::test]
async fn test_async_save_load_roundtrip() -> CliResult<()> {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("roundtrip_config.toml");

    // Create a test config
    let mut original_config = Config::default();
    original_config.api.token = Some("roundtrip_token".to_string());
    original_config.api.timeout = 120;
    original_config.logging.level = "debug".to_string();
    original_config.files.max_file_size = 52428800; // 50MB

    // Save asynchronously
    let start = Instant::now();
    original_config.save_async(Some(&config_path)).await?;
    let save_duration = start.elapsed();

    // Load asynchronously
    let start = Instant::now();
    let loaded_config = Config::from_path_async(&config_path).await?;
    let load_duration = start.elapsed();

    // Verify round-trip integrity
    assert_eq!(original_config.api.token, loaded_config.api.token);
    assert_eq!(original_config.api.timeout, loaded_config.api.timeout);
    assert_eq!(original_config.logging.level, loaded_config.logging.level);
    assert_eq!(
        original_config.files.max_file_size,
        loaded_config.files.max_file_size
    );

    println!("Async save: {save_duration:?}, Async load: {load_duration:?}");

    // Both operations should be reasonably fast
    assert!(save_duration < Duration::from_millis(100));
    assert!(load_duration < Duration::from_millis(100));

    Ok(())
}

/// Test config merging efficiency
#[tokio::test]
async fn test_efficient_config_merging() -> CliResult<()> {
    let temp_dir = tempdir().unwrap();

    // Create multiple config files
    let base_config_path = temp_dir.path().join("base.toml");
    let override_config_path = temp_dir.path().join("override.toml");

    let base_config = r#"
[api]
token = "base_token"
timeout = 30
max_retries = 3

[logging]
level = "info"
colors = true
"#;

    let override_config = r#"
[api]
token = "override_token"
timeout = 60

[files]
max_file_size = 209715200
"#;

    fs::write(&base_config_path, base_config).await?;
    fs::write(&override_config_path, override_config).await?;

    // Load both configs
    let base = Config::from_path_async(&base_config_path).await?;
    let overlay = Config::from_path_async(&override_config_path).await?;

    // Test efficient merging
    let start = Instant::now();
    let merged = AsyncConfigManager::merge_configs_efficient(base, overlay);
    let merge_duration = start.elapsed();

    // Verify merging results
    assert_eq!(merged.api.token, Some("override_token".to_string())); // Overridden
    assert_eq!(merged.api.timeout, 60); // Overridden
    assert_eq!(merged.api.max_retries, 3); // From base
    assert_eq!(merged.logging.level, "info"); // From base
    assert!(merged.logging.colors); // From base
    assert_eq!(merged.files.max_file_size, 209715200); // From override

    println!("Config merging took: {merge_duration:?}");

    // Merging should be very fast (microseconds)
    assert!(merge_duration < Duration::from_millis(1));

    Ok(())
}

/// Test parallel config loading from multiple sources
#[tokio::test]
async fn test_parallel_config_loading() -> CliResult<()> {
    let temp_dir = tempdir().unwrap();

    // Create multiple config files
    let config_files = vec![
        ("config1.toml", "[api]\ntoken = \"token1\"\n"),
        ("config2.toml", "[api]\ntimeout = 45\n"),
        ("config3.toml", "[logging]\nlevel = \"debug\"\n"),
    ];

    for (filename, content) in &config_files {
        let path = temp_dir.path().join(filename);
        fs::write(&path, content).await?;
    }

    // Create manager and test parallel loading
    let mut manager = AsyncConfigManager::new(Duration::from_secs(300));
    manager.config_paths = config_files
        .iter()
        .map(|(filename, _)| temp_dir.path().join(filename))
        .collect();

    let start = Instant::now();
    let merged_config = manager.load_from_sources().await?;
    let parallel_load_duration = start.elapsed();

    // Verify that all configs were merged
    assert_eq!(merged_config.api.token, Some("token1".to_string()));
    assert_eq!(merged_config.api.timeout, 45);
    assert_eq!(merged_config.logging.level, "debug");

    println!("Parallel loading of 3 configs took: {parallel_load_duration:?}");

    // Parallel loading should be faster than sequential
    // (though for small files the difference might be minimal)
    assert!(parallel_load_duration < Duration::from_millis(50));

    Ok(())
}

/// Benchmark comparison between sync and async operations
#[tokio::test]
async fn benchmark_sync_vs_async_operations() -> CliResult<()> {
    const NUM_OPERATIONS: usize = 100;

    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("benchmark.toml");

    let test_config = r#"
[api]
token = "benchmark_token"
timeout = 30

[logging]
level = "info"
"#;

    fs::write(&config_path, test_config).await?;

    // Benchmark sync operations
    let start = Instant::now();
    for _ in 0..NUM_OPERATIONS {
        let _config = Config::from_path(&config_path)?;
    }
    let sync_total = start.elapsed();

    // Benchmark async operations
    let start = Instant::now();
    for _ in 0..NUM_OPERATIONS {
        let _config = Config::from_path_async(&config_path).await?;
    }
    let async_total = start.elapsed();

    // Benchmark cached async operations
    let manager = AsyncConfigManager::new(Duration::from_secs(300));
    let start = Instant::now();
    for _ in 0..NUM_OPERATIONS {
        let _config = manager.load_config().await?;
    }
    let cached_total = start.elapsed();

    println!("Sync {NUM_OPERATIONS} operations: {sync_total:?}");
    println!("Async {NUM_OPERATIONS} operations: {async_total:?}");
    println!("Cached {NUM_OPERATIONS} operations: {cached_total:?}");

    // Cached operations should be significantly faster
    assert!(cached_total < sync_total);
    assert!(cached_total < async_total);

    // Report performance improvements
    let cache_speedup = sync_total.as_nanos() as f64 / cached_total.as_nanos() as f64;
    println!("Cache speedup: {cache_speedup:.2}x");

    assert!(cache_speedup > 2.0); // At least 2x speedup expected

    Ok(())
}
