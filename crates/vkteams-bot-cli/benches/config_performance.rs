use criterion::{Criterion, criterion_group, criterion_main};
use std::collections::HashMap;
use std::hint::black_box;
use tempfile::tempdir;
use vkteams_bot_cli::config::{
    ApiConfig, Config, FileConfig, LoggingConfig, RateLimitConfig, UiConfig,
};

/// Benchmark config loading from different sources
fn bench_config_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_loading");

    // Test loading default config
    group.bench_function("load_default", |b| {
        b.iter(|| {
            let config = black_box(Config::default());
            black_box(config)
        });
    });

    // Test loading config with all fields
    group.bench_function("load_full_config", |b| {
        b.iter(|| {
            let config = black_box(Config {
                api: ApiConfig {
                    token: Some("test_token_very_long_string_with_many_characters".to_string()),
                    url: Some("https://api.vk.com/teams/bot/v1/".to_string()),
                    timeout: 60,
                    max_retries: 5,
                },
                files: FileConfig {
                    download_dir: Some("/tmp/downloads".to_string()),
                    upload_dir: Some("/tmp/uploads".to_string()),
                    max_file_size: 100 * 1024 * 1024, // 100MB
                    buffer_size: 8192,
                },
                logging: LoggingConfig {
                    level: "info".to_string(),
                    format: "json".to_string(),
                    colors: true,
                },
                ui: UiConfig {
                    show_progress: true,
                    progress_style: "unicode".to_string(),
                    progress_refresh_rate: 100,
                },
                proxy: None,
                rate_limit: RateLimitConfig {
                    enabled: true,
                    limit: 100,
                    duration: 60,
                    retry_delay: 500,
                    retry_attempts: 3,
                },
            });
            black_box(config)
        });
    });

    group.finish();
}

/// Benchmark config serialization and deserialization
fn bench_config_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_serialization");

    let test_config = Config {
        api: ApiConfig {
            token: Some("test_token".to_string()),
            url: Some("https://api.vk.com".to_string()),
            timeout: 30,
            max_retries: 3,
        },
        ..Default::default()
    };

    // Test TOML serialization
    group.bench_function("serialize_toml", |b| {
        b.iter(|| {
            let serialized = black_box(toml::to_string(&test_config));
            black_box(serialized)
        });
    });

    // Test JSON serialization for comparison
    group.bench_function("serialize_json", |b| {
        b.iter(|| {
            let serialized = black_box(serde_json::to_string(&test_config));
            black_box(serialized)
        });
    });

    // Test TOML deserialization
    let toml_string = toml::to_string(&test_config).unwrap();
    group.bench_function("deserialize_toml", |b| {
        b.iter(|| {
            let deserialized: Result<Config, _> = black_box(toml::from_str(&toml_string));
            black_box(deserialized)
        });
    });

    // Test JSON deserialization for comparison
    let json_string = serde_json::to_string(&test_config).unwrap();
    group.bench_function("deserialize_json", |b| {
        b.iter(|| {
            let deserialized: Result<Config, _> = black_box(serde_json::from_str(&json_string));
            black_box(deserialized)
        });
    });

    group.finish();
}

/// Benchmark config file operations
fn bench_config_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_file_operations");

    let test_config = Config::default();

    // Test config file writing
    group.bench_function("write_config_file", |b| {
        b.iter(|| {
            let dir = black_box(tempdir().unwrap());
            let file_path = dir.path().join("config.toml");
            let content = toml::to_string(&test_config).unwrap();
            let result = std::fs::write(&file_path, content);
            black_box(result)
        });
    });

    // Test config file reading
    group.bench_function("read_config_file", |b| {
        // Setup: create a config file
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.toml");
        let content = toml::to_string(&test_config).unwrap();
        std::fs::write(&file_path, content).unwrap();

        b.iter(|| {
            let content = black_box(std::fs::read_to_string(&file_path));
            black_box(content)
        });
    });

    group.finish();
}

/// Benchmark config validation operations
fn bench_config_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_validation");

    // Test valid config validation
    group.bench_function("validate_valid_config", |b| {
        let config = Config {
            api: ApiConfig {
                token: Some("valid_token".to_string()),
                url: Some("https://api.vk.com".to_string()),
                timeout: 30,
                max_retries: 3,
            },
            ..Default::default()
        };

        b.iter(|| {
            // Simulate comprehensive validation
            let is_valid = black_box(
                config.api.token.is_some()
                    && config.api.url.is_some()
                    && config.api.timeout > 0
                    && config.api.max_retries > 0,
            );
            black_box(is_valid)
        });
    });

    // Test invalid config validation
    group.bench_function("validate_invalid_config", |b| {
        let config = Config {
            api: ApiConfig {
                token: None,
                url: None,
                timeout: 0,
                max_retries: 0,
            },
            ..Default::default()
        };

        b.iter(|| {
            // Simulate comprehensive validation
            let is_valid = black_box(
                config.api.token.is_some()
                    && config.api.url.is_some()
                    && config.api.timeout > 0
                    && config.api.max_retries > 0,
            );
            black_box(is_valid)
        });
    });

    group.finish();
}

/// Benchmark config merging operations
fn bench_config_merging(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_merging");

    let base_config = Config::default();
    let override_config = Config {
        api: ApiConfig {
            token: Some("override_token".to_string()),
            timeout: 60,
            ..Default::default()
        },
        ..Default::default()
    };

    group.bench_function("merge_configs", |b| {
        b.iter(|| {
            // Simulate config merging
            let mut merged = black_box(base_config.clone());
            if let Some(token) = &override_config.api.token {
                merged.api.token = Some(token.clone());
            }
            merged.api.timeout = override_config.api.timeout;
            black_box(merged)
        });
    });

    group.finish();
}

/// Benchmark environment variable parsing
fn bench_env_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("env_parsing");

    // Setup environment variables
    unsafe {
        std::env::set_var("VKTEAMS_BOT_TOKEN", "test_token");
        std::env::set_var("VKTEAMS_BOT_URL", "https://api.vk.com");
        std::env::set_var("VKTEAMS_BOT_TIMEOUT", "30");
    }

    group.bench_function("parse_env_vars", |b| {
        b.iter(|| {
            let token = black_box(std::env::var("VKTEAMS_BOT_TOKEN").ok());
            let url = black_box(std::env::var("VKTEAMS_BOT_URL").ok());
            let timeout = black_box(
                std::env::var("VKTEAMS_BOT_TIMEOUT")
                    .ok()
                    .and_then(|t| t.parse::<u64>().ok()),
            );
            black_box((token, url, timeout))
        });
    });

    // Test batch environment variable parsing
    group.bench_function("parse_multiple_env_vars", |b| {
        let env_vars = [
            "VKTEAMS_BOT_TOKEN",
            "VKTEAMS_BOT_URL",
            "VKTEAMS_BOT_TIMEOUT",
            "VKTEAMS_BOT_MAX_RETRIES",
            "VKTEAMS_BOT_LOG_LEVEL",
        ];

        b.iter(|| {
            let values: HashMap<String, String> = env_vars
                .iter()
                .filter_map(|&var| std::env::var(var).ok().map(|val| (var.to_string(), val)))
                .collect();
            black_box(values)
        });
    });

    group.finish();
}

/// Benchmark config caching operations
fn bench_config_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_caching");

    let config = Config::default();

    // Test config cloning (for caching)
    group.bench_function("clone_config", |b| {
        b.iter(|| {
            let cloned = black_box(config.clone());
            black_box(cloned)
        });
    });

    // Test config hashing (for cache invalidation)
    group.bench_function("hash_config", |b| {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        b.iter(|| {
            let mut hasher = DefaultHasher::new();
            let config_str = serde_json::to_string(&config).unwrap();
            config_str.hash(&mut hasher);
            let hash = black_box(hasher.finish());
            black_box(hash)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_config_loading,
    bench_config_serialization,
    bench_config_file_operations,
    bench_config_validation,
    bench_config_merging,
    bench_env_parsing,
    bench_config_caching
);

criterion_main!(benches);
