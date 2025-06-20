use criterion::{Criterion, criterion_group, criterion_main};
use reqwest::StatusCode;
use std::hint::black_box;
use std::time::Duration;
use vkteams_bot::bot::net::{
    RetryableMultipartForm, calculate_backoff_duration, should_retry_status,
};

fn bench_retryable_form_creation(c: &mut Criterion) {
    let content = vec![0u8; 1024]; // 1KB
    let filename = "test.txt".to_string();

    c.bench_function("retryable_form_creation", |b| {
        b.iter(|| {
            black_box(RetryableMultipartForm::from_content(
                filename.clone(),
                filename.clone(),
                content.clone(),
            ))
        });
    });
}

fn bench_backoff_calculation(c: &mut Criterion) {
    let max_backoff = Duration::from_secs(60);

    c.bench_function("backoff_calculation", |b| {
        b.iter(|| {
            for attempt in 1..=5 {
                black_box(calculate_backoff_duration(
                    black_box(attempt),
                    black_box(max_backoff),
                ));
            }
        })
    });
}

fn bench_retry_status_check(c: &mut Criterion) {
    let status = StatusCode::INTERNAL_SERVER_ERROR;

    c.bench_function("retry_status_check", |b| {
        b.iter(|| {
            black_box(should_retry_status(black_box(&status)));
        })
    });
}

criterion_group!(
    benches,
    bench_retryable_form_creation,
    bench_backoff_calculation,
    bench_retry_status_check
);

criterion_main!(benches);
