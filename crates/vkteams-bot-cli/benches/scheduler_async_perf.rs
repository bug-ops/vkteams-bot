use chrono::{Duration as ChronoDuration, Utc};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;
use tokio::runtime::Runtime;
use vkteams_bot_cli::scheduler::{ScheduleType, Scheduler, TaskType};

fn benchmark_scheduler_task_management(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("scheduler_task_management");

    // Benchmark task addition
    group.bench_function("add_single_task", |b| {
        b.to_async(&rt).iter(|| async {
            let (mut scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

            black_box(
                scheduler
                    .add_task(
                        TaskType::SendText {
                            chat_id: "bench_chat".to_string(),
                            message: "bench message".to_string(),
                        },
                        ScheduleType::Once(Utc::now() + ChronoDuration::hours(1)),
                        None,
                    )
                    .await
                    .unwrap(),
            );
        });
    });

    // Benchmark bulk task addition
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("add_bulk_tasks", size),
            size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let (mut scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

                    for i in 0..size {
                        black_box(
                            scheduler
                                .add_task(
                                    TaskType::SendText {
                                        chat_id: format!("chat_{}", i),
                                        message: format!("message_{}", i),
                                    },
                                    ScheduleType::Once(
                                        Utc::now() + ChronoDuration::minutes(i as i64),
                                    ),
                                    None,
                                )
                                .await
                                .unwrap(),
                        );
                    }
                });
            },
        );
    }

    // Benchmark task listing
    group.bench_function("list_tasks_1000", |b| {
        let (scheduler, _temp_dir) = rt.block_on(async {
            let (mut scheduler, temp_dir) = Scheduler::create_test_scheduler().await;

            // Pre-populate with 1000 tasks
            for i in 0..1000 {
                scheduler
                    .add_task(
                        TaskType::SendText {
                            chat_id: format!("chat_{}", i),
                            message: format!("message_{}", i),
                        },
                        ScheduleType::Once(Utc::now() + ChronoDuration::minutes(i as i64)),
                        None,
                    )
                    .await
                    .unwrap();
            }

            (scheduler, temp_dir)
        });

        b.to_async(&rt).iter(|| async {
            black_box(scheduler.list_tasks().await);
        });
    });

    group.finish();
}

fn benchmark_scheduler_async_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("scheduler_async_operations");

    // Benchmark async file I/O operations
    group.bench_function("save_and_load_tasks", |b| {
        b.to_async(&rt).iter(|| async {
            let temp_dir = tempfile::tempdir().unwrap();
            let data_dir = temp_dir.path().to_path_buf();

            {
                let mut scheduler = Scheduler::new(Some(data_dir.clone())).await.unwrap();

                // Add some tasks
                for i in 0..10 {
                    scheduler
                        .add_task(
                            TaskType::SendText {
                                chat_id: format!("persistent_chat_{}", i),
                                message: format!("persistent_message_{}", i),
                            },
                            ScheduleType::Once(Utc::now() + ChronoDuration::hours(i as i64)),
                            None,
                        )
                        .await
                        .unwrap();
                }
            }

            // Load scheduler with persisted tasks
            let scheduler = black_box(Scheduler::new(Some(data_dir)).await.unwrap());
            assert_eq!(scheduler.list_tasks().await.len(), 10);
        });
    });

    // Benchmark concurrent task operations
    group.bench_function("concurrent_task_operations", |b| {
        b.to_async(&rt).iter(|| async {
            let (mut scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

            // Add 100 tasks
            let mut task_ids = Vec::new();
            for i in 0..100 {
                let task_id = scheduler
                    .add_task(
                        TaskType::SendText {
                            chat_id: format!("concurrent_chat_{}", i),
                            message: format!("concurrent_message_{}", i),
                        },
                        ScheduleType::Once(Utc::now() + ChronoDuration::minutes(i as i64)),
                        None,
                    )
                    .await
                    .unwrap();
                task_ids.push(task_id);
            }

            // Perform concurrent operations
            for task_id in &task_ids[0..50] {
                scheduler.disable_task(task_id).await.unwrap();
                black_box(());
            }

            for task_id in &task_ids[25..75] {
                scheduler.enable_task(task_id).await.unwrap();
                black_box(());
            }

            for task_id in &task_ids[75..100] {
                scheduler.remove_task(task_id).await.unwrap();
                black_box(());
            }
        });
    });

    group.finish();
}

fn benchmark_scheduler_queue_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("scheduler_queue_operations");

    // Benchmark queue rebuilding
    group.bench_function("rebuild_queue_1000_tasks", |b| {
        let (scheduler, _temp_dir) = rt.block_on(async {
            let (mut scheduler, temp_dir) = Scheduler::create_test_scheduler().await;

            // Pre-populate with 1000 tasks
            for i in 0..1000 {
                scheduler
                    .add_task(
                        TaskType::SendText {
                            chat_id: format!("queue_chat_{}", i),
                            message: format!("queue_message_{}", i),
                        },
                        ScheduleType::Once(Utc::now() + ChronoDuration::minutes(i as i64)),
                        None,
                    )
                    .await
                    .unwrap();
            }

            (scheduler, temp_dir)
        });

        b.to_async(&rt).iter(|| async {
            scheduler.rebuild_queue().await;
            black_box(());
        });
    });

    // Benchmark ready task extraction
    group.bench_function("extract_ready_tasks", |b| {
        let (scheduler, _temp_dir) = rt.block_on(async {
            let (mut scheduler, temp_dir) = Scheduler::create_test_scheduler().await;

            // Add tasks with past due times (ready for execution)
            for i in 0..100 {
                scheduler
                    .add_task(
                        TaskType::SendText {
                            chat_id: format!("ready_chat_{}", i),
                            message: format!("ready_message_{}", i),
                        },
                        ScheduleType::Once(Utc::now() - ChronoDuration::minutes(i as i64)), // Past due
                        None,
                    )
                    .await
                    .unwrap();
            }

            (scheduler, temp_dir)
        });

        b.to_async(&rt).iter(|| async {
            black_box(scheduler.extract_ready_tasks().await);
        });
    });

    group.finish();
}

fn benchmark_scheduler_event_driven_features(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("scheduler_event_driven");

    // Benchmark wakeup time calculation
    group.bench_function("calculate_next_wakeup", |b| {
        let (scheduler, _temp_dir) = rt.block_on(async {
            let (mut scheduler, temp_dir) = Scheduler::create_test_scheduler().await;

            // Add tasks with various schedules
            for i in 0..100 {
                scheduler
                    .add_task(
                        TaskType::SendText {
                            chat_id: format!("wakeup_chat_{}", i),
                            message: format!("wakeup_message_{}", i),
                        },
                        ScheduleType::Once(Utc::now() + ChronoDuration::minutes(i as i64)),
                        None,
                    )
                    .await
                    .unwrap();
            }

            (scheduler, temp_dir)
        });

        b.to_async(&rt).iter(|| async {
            black_box(scheduler.calculate_next_wakeup().await);
        });
    });

    // Benchmark scheduler configuration changes
    group.bench_function("scheduler_configuration", |b| {
        b.to_async(&rt).iter(|| async {
            let (mut scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

            scheduler.set_max_concurrent_tasks(20);
            black_box(());
            scheduler.set_task_timeout(Duration::from_secs(600));
            black_box(());
        });
    });

    group.finish();
}

fn benchmark_schedule_types(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("schedule_types");

    // Benchmark different schedule type calculations
    group.bench_function("cron_next_run_calculation", |b| {
        b.to_async(&rt).iter(|| async {
            let (mut scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

            black_box(
                scheduler
                    .add_task(
                        TaskType::SendText {
                            chat_id: "cron_bench_chat".to_string(),
                            message: "cron bench message".to_string(),
                        },
                        ScheduleType::Cron("0 0 0 * * *".to_string()), // Daily at midnight
                        None,
                    )
                    .await
                    .unwrap(),
            );
        });
    });

    group.bench_function("interval_next_run_calculation", |b| {
        b.to_async(&rt).iter(|| async {
            let (mut scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

            black_box(
                scheduler
                    .add_task(
                        TaskType::SendText {
                            chat_id: "interval_bench_chat".to_string(),
                            message: "interval bench message".to_string(),
                        },
                        ScheduleType::Interval {
                            duration_seconds: 3600, // 1 hour
                            start_time: Utc::now() - ChronoDuration::hours(2),
                        },
                        None,
                    )
                    .await
                    .unwrap(),
            );
        });
    });

    group.bench_function("once_schedule_calculation", |b| {
        b.to_async(&rt).iter(|| async {
            let (mut scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

            black_box(
                scheduler
                    .add_task(
                        TaskType::SendText {
                            chat_id: "once_bench_chat".to_string(),
                            message: "once bench message".to_string(),
                        },
                        ScheduleType::Once(Utc::now() + ChronoDuration::hours(1)),
                        None,
                    )
                    .await
                    .unwrap(),
            );
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_scheduler_task_management,
    benchmark_scheduler_async_operations,
    benchmark_scheduler_queue_operations,
    benchmark_scheduler_event_driven_features,
    benchmark_schedule_types
);
criterion_main!(benches);
