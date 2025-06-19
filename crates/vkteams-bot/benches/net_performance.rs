use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use reqwest::StatusCode;
use std::hint::black_box;
use std::time::Duration;
use tempfile::tempdir;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use vkteams_bot::bot::net::{
    RetryableMultipartForm, calculate_backoff_duration, should_retry_status,
};

fn bench_retryable_form_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("retryable_form_creation");

    let sizes = vec![1024, 10240, 102400, 1024000]; // 1KB to 1MB

    for size in sizes {
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("from_content", size), &size, |b, &size| {
            let content = vec![0u8; size];
            let filename = "test.txt".to_string();

            b.iter(|| {
                black_box(RetryableMultipartForm::from_content(
                    filename.clone(),
                    filename.clone(),
                    content.clone(),
                ))
            });
        });
    }

    group.finish();
}

fn bench_form_conversion(c: &mut Criterion) {
    let content = vec![0u8; 10240]; // 10KB
    let filename = "test.txt".to_string();
    let retryable_form = RetryableMultipartForm::from_content(filename.clone(), filename, content);

    c.bench_function("form_conversion", |b| {
        b.iter(|| black_box(retryable_form.to_form()))
    });
}

fn bench_backoff_calculation(c: &mut Criterion) {
    let max_backoff = Duration::from_secs(60);

    c.bench_function("backoff_calculation", |b| {
        b.iter(|| {
            for attempt in 1..=10 {
                black_box(calculate_backoff_duration(
                    black_box(attempt),
                    black_box(max_backoff),
                ));
            }
        })
    });
}

fn bench_retry_status_check(c: &mut Criterion) {
    let status_codes = vec![
        StatusCode::OK,
        StatusCode::BAD_REQUEST,
        StatusCode::UNAUTHORIZED,
        StatusCode::NOT_FOUND,
        StatusCode::TOO_MANY_REQUESTS,
        StatusCode::INTERNAL_SERVER_ERROR,
        StatusCode::BAD_GATEWAY,
        StatusCode::SERVICE_UNAVAILABLE,
    ];

    c.bench_function("retry_status_check", |b| {
        b.iter(|| {
            for status in &status_codes {
                black_box(should_retry_status(black_box(status)));
            }
        })
    });
}

async fn create_temp_file(size: usize) -> (tempfile::TempDir, String) {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("bench_file.txt");
    let content = vec![0u8; size];

    let mut file = File::create(&file_path).await.unwrap();
    file.write_all(&content).await.unwrap();
    file.flush().await.unwrap();

    (temp_dir, file_path.to_string_lossy().to_string())
}

fn bench_file_loading(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("file_loading");

    let sizes = vec![1024, 10240, 102400]; // 1KB to 100KB

    for size in sizes {
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::new("from_file_path", size),
            &size,
            |b, &size| {
                let (_temp_dir, file_path) = rt.block_on(create_temp_file(size));

                b.to_async(&rt).iter(|| async {
                    black_box(
                        RetryableMultipartForm::from_file_path(file_path.clone())
                            .await
                            .unwrap(),
                    )
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_retryable_form_creation,
    bench_form_conversion,
    bench_backoff_calculation,
    bench_retry_status_check,
    bench_file_loading
);

criterion_main!(benches);
