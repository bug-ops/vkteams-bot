//! Integration tests for the optimized async scheduler

use chrono::{Duration as ChronoDuration, Utc};
use std::time::Duration;
use tokio::time::timeout;
use vkteams_bot_cli::scheduler::{ScheduleType, Scheduler, TaskType};

#[tokio::test]
async fn test_scheduler_event_driven_wakeup() {
    let (mut scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

    // Test that adding a task triggers immediate wakeup calculation
    let future_time = Utc::now() + ChronoDuration::milliseconds(100);
    let task_id = scheduler
        .add_task(
            TaskType::SendText {
                chat_id: "test_chat".to_string(),
                message: "test message".to_string(),
            },
            ScheduleType::Once(future_time),
            None,
        )
        .await
        .unwrap();

    // Verify task was added
    let task = scheduler.get_task(&task_id).await.unwrap();
    assert_eq!(task.id, task_id);
    assert!(task.enabled);
}

#[tokio::test]
async fn test_scheduler_concurrent_task_management() {
    let (mut scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

    // Add multiple tasks concurrently
    let mut task_ids = Vec::new();
    for i in 0..10 {
        let task_id = scheduler
            .add_task(
                TaskType::SendText {
                    chat_id: format!("chat_{i}"),
                    message: format!("message_{i}"),
                },
                ScheduleType::Once(Utc::now() + ChronoDuration::minutes(i as i64)),
                None,
            )
            .await
            .unwrap();
        task_ids.push(task_id);
    }

    // Verify all tasks were added
    let tasks = scheduler.list_tasks().await;
    assert_eq!(tasks.len(), 10);

    // Test concurrent task operations
    for task_id in task_ids.iter().take(5) {
        scheduler.disable_task(task_id).await.unwrap();
    }

    // Verify half the tasks are disabled
    let tasks = scheduler.list_tasks().await;
    let disabled_count = tasks.iter().filter(|t| !t.enabled).count();
    assert_eq!(disabled_count, 5);
}

#[tokio::test]
async fn test_scheduler_graceful_shutdown() {
    let (scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

    // Test graceful shutdown
    scheduler.shutdown().await;

    // Verify shutdown signal is set
    assert!(
        scheduler
            .shutdown_signal
            .load(std::sync::atomic::Ordering::Relaxed)
    );
}

#[tokio::test]
async fn test_scheduler_interval_task_next_run_calculation() {
    let (mut scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

    let start_time = Utc::now() - ChronoDuration::minutes(30);
    let task_id = scheduler
        .add_task(
            TaskType::SendText {
                chat_id: "test_chat".to_string(),
                message: "interval test".to_string(),
            },
            ScheduleType::Interval {
                duration_seconds: 600, // 10 minutes
                start_time,
            },
            None,
        )
        .await
        .unwrap();

    let task = scheduler.get_task(&task_id).await.unwrap();

    // Next run should be calculated from start time + intervals
    assert!(task.next_run > Utc::now());
    assert!(task.next_run <= Utc::now() + ChronoDuration::minutes(10));
}

#[tokio::test]
async fn test_scheduler_cron_task_creation() {
    let (mut scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

    // Test valid cron expression
    let task_id = scheduler
        .add_task(
            TaskType::SendText {
                chat_id: "test_chat".to_string(),
                message: "cron test".to_string(),
            },
            ScheduleType::Cron("0 0 0 * * *".to_string()), // Daily at midnight (6-field format)
            None,
        )
        .await
        .unwrap();

    let task = scheduler.get_task(&task_id).await.unwrap();
    assert!(task.next_run > Utc::now());
}

#[tokio::test]
async fn test_scheduler_max_runs_limit() {
    let (mut scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

    let task_id = scheduler
        .add_task(
            TaskType::SendText {
                chat_id: "test_chat".to_string(),
                message: "limited runs test".to_string(),
            },
            ScheduleType::Once(Utc::now() + ChronoDuration::seconds(1)),
            Some(1), // Max 1 run
        )
        .await
        .unwrap();

    let task = scheduler.get_task(&task_id).await.unwrap();
    assert_eq!(task.max_runs, Some(1));
    assert_eq!(task.run_count, 0);
}

#[tokio::test]
async fn test_scheduler_task_persistence() {
    let temp_dir = tempfile::tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    let task_id = {
        let mut scheduler = Scheduler::new(Some(data_dir.clone())).await.unwrap();
        scheduler
            .add_task(
                TaskType::SendText {
                    chat_id: "persistent_chat".to_string(),
                    message: "persistent message".to_string(),
                },
                ScheduleType::Once(Utc::now() + ChronoDuration::hours(1)),
                None,
            )
            .await
            .unwrap()
    };

    // Create new scheduler instance and verify task persistence
    let scheduler = Scheduler::new(Some(data_dir)).await.unwrap();
    let task = scheduler.get_task(&task_id).await.unwrap();
    assert_eq!(task.id, task_id);
}

#[tokio::test]
async fn test_scheduler_task_queue_management() {
    let (mut scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

    // Add tasks with different schedule times
    let _task1 = scheduler
        .add_task(
            TaskType::SendText {
                chat_id: "chat1".to_string(),
                message: "message1".to_string(),
            },
            ScheduleType::Once(Utc::now() + ChronoDuration::minutes(1)),
            None,
        )
        .await
        .unwrap();

    let _task2 = scheduler
        .add_task(
            TaskType::SendText {
                chat_id: "chat2".to_string(),
                message: "message2".to_string(),
            },
            ScheduleType::Once(Utc::now() + ChronoDuration::minutes(2)),
            None,
        )
        .await
        .unwrap();

    // Test that queue is properly maintained
    let tasks = scheduler.list_tasks().await;
    assert_eq!(tasks.len(), 2);

    // Test task removal updates queue
    scheduler.remove_task(&_task1).await.unwrap();
    let tasks = scheduler.list_tasks().await;
    assert_eq!(tasks.len(), 1);
}

#[tokio::test]
async fn test_scheduler_ready_tasks_extraction() {
    let (scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

    // Test that past due tasks are correctly identified
    // This is a unit test for internal functionality
    let ready_tasks = scheduler.extract_ready_tasks().await;

    // Should start with no ready tasks
    assert!(ready_tasks.is_empty());
}

#[tokio::test]
async fn test_scheduler_configuration() {
    let (mut scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

    // Test scheduler configuration
    scheduler.set_max_concurrent_tasks(5);
    scheduler.set_task_timeout(Duration::from_secs(120));

    // Verify configuration was applied
    assert_eq!(scheduler.max_concurrent_tasks, 5);
    assert_eq!(scheduler.task_timeout, Duration::from_secs(120));
}

#[tokio::test]
async fn test_scheduler_async_file_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    // Test async loading
    let scheduler = timeout(
        Duration::from_secs(5),
        Scheduler::new(Some(data_dir.clone())),
    )
    .await
    .expect("Scheduler creation should not timeout")
    .unwrap();

    // Test async saving by adding a task
    let mut scheduler = scheduler;
    let _task_id = timeout(
        Duration::from_secs(5),
        scheduler.add_task(
            TaskType::SendText {
                chat_id: "async_test".to_string(),
                message: "async file test".to_string(),
            },
            ScheduleType::Once(Utc::now() + ChronoDuration::hours(1)),
            None,
        ),
    )
    .await
    .expect("Task addition should not timeout")
    .unwrap();

    // Verify file was created asynchronously
    let data_file = data_dir.join("scheduler_tasks.json");
    assert!(data_file.exists());
}
