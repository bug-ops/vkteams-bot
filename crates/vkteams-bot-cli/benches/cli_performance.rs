use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;
use tokio::runtime::Runtime;
use vkteams_bot_cli::config::{ApiConfig, Config};

/// Benchmark CLI argument parsing performance
fn bench_cli_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("cli_parsing");

    let test_args = vec![
        vec![
            "vkteams-bot-cli",
            "send-text",
            "--chat-id",
            "test",
            "--message",
            "Hello",
        ],
        vec!["vkteams-bot-cli", "chat", "info"],
        vec!["vkteams-bot-cli", "config", "show"],
        vec!["vkteams-bot-cli", "--verbose", "diagnostic", "status"],
        vec!["vkteams-bot-cli", "--output", "json", "chat", "members"],
    ];

    for (i, args) in test_args.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("parse_args", i), args, |b, args| {
            b.iter(|| {
                // Simulate CLI argument parsing
                let _parsed = black_box(args.len());
                let _validation = black_box(args.iter().all(|arg| !arg.is_empty()));
                black_box(true)
            });
        });
    }

    group.finish();
}

/// Benchmark configuration loading and parsing
fn bench_config_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_operations");

    // Test config creation
    group.bench_function("create_default_config", |b| {
        b.iter(|| {
            let config = black_box(Config::default());
            black_box(config)
        });
    });

    // Test config serialization
    group.bench_function("serialize_config", |b| {
        let config = Config::default();
        b.iter(|| {
            let serialized = black_box(toml::to_string(&config));
            black_box(serialized)
        });
    });

    // Test config validation
    group.bench_function("validate_config", |b| {
        let config = Config {
            api: ApiConfig {
                token: Some("test_token".to_string()),
                url: Some("https://api.vk.com".to_string()),
                timeout: 30,
                max_retries: 3,
            },
            ..Default::default()
        };

        b.iter(|| {
            let valid = black_box(config.clone());
            // Simulate validation logic
            let is_valid = valid.api.token.is_some() && valid.api.url.is_some();
            black_box(is_valid)
        });
    });

    group.finish();
}

/// Benchmark command validation performance
fn bench_command_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("command_validation");

    // Test different command validations
    let long_message =
        "This is a very long message that should be validated for length and content. ".repeat(10);
    let test_cases = vec![
        ("chat_id_validation", "123456789"),
        (
            "long_chat_id_validation",
            "very_long_chat_id_with_many_characters_123456789",
        ),
        ("message_validation", "Hello, world!"),
        ("long_message_validation", long_message.as_str()),
    ];

    for (name, input) in test_cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                // Simulate validation logic
                let input = black_box(input);
                let is_valid = !input.is_empty() && input.len() < 10000;
                black_box(is_valid)
            });
        });
    }

    group.finish();
}

/// Benchmark string operations used in CLI
fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    // Test message formatting
    let long_message =
        "Very long message that needs to be processed and formatted properly. ".repeat(50);
    let sample_messages = vec![
        "Simple message",
        "Message with emoji 🚀",
        long_message.as_str(),
    ];

    for (i, message) in sample_messages.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("format_message", i), message, |b, msg| {
            b.iter(|| {
                let formatted = black_box(format!("[CLI] {}", msg));
                black_box(formatted)
            });
        });
    }

    // Test JSON serialization
    group.bench_function("json_serialization", |b| {
        let data = serde_json::json!({
            "message": "Hello, world!",
            "chat_id": "123456789",
            "timestamp": "2024-01-01T00:00:00Z"
        });

        b.iter(|| {
            let serialized = black_box(serde_json::to_string(&data));
            black_box(serialized)
        });
    });

    group.finish();
}

/// Benchmark output formatting performance
fn bench_output_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("output_formatting");

    let test_data = serde_json::json!({
        "status": "success",
        "data": {
            "chat_id": "123456789",
            "message_id": "987654321",
            "timestamp": "2024-01-01T00:00:00Z"
        }
    });

    // Test different output formats
    group.bench_function("format_pretty", |b| {
        b.iter(|| {
            let pretty = black_box(serde_json::to_string_pretty(&test_data));
            black_box(pretty)
        });
    });

    group.bench_function("format_compact", |b| {
        b.iter(|| {
            let compact = black_box(serde_json::to_string(&test_data));
            black_box(compact)
        });
    });

    group.finish();
}

/// Benchmark throughput for batch operations
fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");

    // Set throughput for batch message processing
    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("process_messages", size),
            size,
            |b, &size| {
                let messages: Vec<String> = (0..size).map(|i| format!("Message {}", i)).collect();

                b.iter(|| {
                    let processed: Vec<_> = messages
                        .iter()
                        .map(|msg| black_box(format!("[PROCESSED] {}", msg)))
                        .collect();
                    black_box(processed)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark async runtime performance
fn bench_async_runtime(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_runtime");

    group.bench_function("create_runtime", |b| {
        b.iter(|| {
            let rt = black_box(Runtime::new().unwrap());
            black_box(rt)
        });
    });

    group.bench_function("simple_async_task", |b| {
        let rt = Runtime::new().unwrap();

        b.iter(|| {
            rt.block_on(async {
                let result = black_box(async_operation().await);
                black_box(result)
            })
        });
    });

    group.finish();
}

/// Simple async operation for benchmarking
async fn async_operation() -> String {
    tokio::time::sleep(Duration::from_nanos(1)).await;
    "completed".to_string()
}

criterion_group!(
    benches,
    bench_cli_parsing,
    bench_config_operations,
    bench_command_validation,
    bench_string_operations,
    bench_output_formatting,
    bench_batch_operations,
    bench_async_runtime
);

criterion_main!(benches);
