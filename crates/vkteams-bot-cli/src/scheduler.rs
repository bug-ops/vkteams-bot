use crate::errors::prelude::{CliError, Result as CliResult};
use chrono::{DateTime, Duration, Utc};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering as CmpOrdering;
use std::collections::{BinaryHeap, HashMap};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tempfile::tempdir;
use tokio::sync::{RwLock, Semaphore, mpsc};
use tokio::time::{Duration as TokioDuration, Instant, sleep_until};
use tracing::{debug, error, info};
use uuid::Uuid;
use vkteams_bot::prelude::*;

pub const SCHEDULER_DATA_FILE: &str = "scheduler_tasks.json";

/// Events that can trigger scheduler wakeup
#[derive(Debug, Clone)]
pub enum SchedulerEvent {
    TaskAdded(String),
    TaskModified(String),
    TaskRemoved(String),
    ForceWakeup,
    Shutdown,
}

/// Wrapper for scheduled tasks in priority queue
#[derive(Debug, Clone)]
struct ScheduledTaskWrapper {
    #[allow(dead_code)] // Used for future task execution logic
    task_id: String,
    next_run_instant: Instant,
}

impl PartialEq for ScheduledTaskWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.next_run_instant == other.next_run_instant
    }
}

impl Eq for ScheduledTaskWrapper {}

impl PartialOrd for ScheduledTaskWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledTaskWrapper {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        // Reverse ordering for min-heap behavior
        other.next_run_instant.cmp(&self.next_run_instant)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub task_type: TaskType,
    pub schedule: ScheduleType,
    pub created_at: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: DateTime<Utc>,
    pub enabled: bool,
    pub run_count: u64,
    pub max_runs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    SendText { chat_id: String, message: String },
    SendFile { chat_id: String, file_path: String },
    SendVoice { chat_id: String, file_path: String },
    SendAction { chat_id: String, action: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleType {
    Once(DateTime<Utc>),
    Cron(String),
    Interval {
        duration_seconds: u64,
        start_time: DateTime<Utc>,
    },
}

/// High-performance event-driven scheduler
pub struct Scheduler {
    tasks: Arc<RwLock<HashMap<String, ScheduledTask>>>,
    task_queue: Arc<RwLock<BinaryHeap<ScheduledTaskWrapper>>>,
    data_file: PathBuf,
    bot: Option<Bot>,
    event_tx: mpsc::UnboundedSender<SchedulerEvent>,
    event_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<SchedulerEvent>>>>,
    pub shutdown_signal: Arc<AtomicBool>,
    pub max_concurrent_tasks: usize,
    pub task_timeout: TokioDuration,
}

impl Scheduler {
    pub async fn new(data_dir: Option<PathBuf>) -> CliResult<Self> {
        let data_dir = data_dir.unwrap_or_else(|| {
            dirs::data_dir()
                .map(|d| d.join("vkteams-bot-cli"))
                .unwrap_or_else(|| PathBuf::from("."))
        });

        // Ensure the data directory exists
        tokio::fs::create_dir_all(&data_dir)
            .await
            .map_err(|e| CliError::FileError(format!("Could not create data directory: {e}")))?;

        let data_file = data_dir.join(SCHEDULER_DATA_FILE);

        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let mut scheduler = Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            data_file,
            bot: None,
            event_tx,
            event_rx: Arc::new(RwLock::new(Some(event_rx))),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            max_concurrent_tasks: 10,
            task_timeout: TokioDuration::from_secs(300), // 5 minutes
        };

        scheduler.load_tasks_async().await?;
        Ok(scheduler)
    }

    pub fn set_bot(&mut self, bot: Bot) {
        self.bot = Some(bot);
    }

    pub fn set_max_concurrent_tasks(&mut self, max: usize) {
        self.max_concurrent_tasks = max;
    }

    pub fn set_task_timeout(&mut self, timeout: TokioDuration) {
        self.task_timeout = timeout;
    }

    pub async fn add_task(
        &mut self,
        task_type: TaskType,
        schedule: ScheduleType,
        max_runs: Option<u64>,
    ) -> CliResult<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let next_run = self.calculate_next_run(&schedule, None)?;

        let task = ScheduledTask {
            id: id.clone(),
            task_type,
            schedule,
            created_at: now,
            last_run: None,
            next_run,
            enabled: true,
            run_count: 0,
            max_runs,
        };

        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(id.clone(), task.clone());
        }

        self.add_to_queue(task).await;
        self.save_tasks_async().await?;

        // Notify scheduler about new task
        let _ = self.event_tx.send(SchedulerEvent::TaskAdded(id.clone()));

        info!("Added scheduled task with ID: {}", id);
        Ok(id)
    }

    pub async fn remove_task(&mut self, task_id: &str) -> CliResult<()> {
        let removed = {
            let mut tasks = self.tasks.write().await;
            tasks.remove(task_id)
        };

        if removed.is_some() {
            self.rebuild_queue().await;
            self.save_tasks_async().await?;

            // Notify scheduler about task removal
            let _ = self
                .event_tx
                .send(SchedulerEvent::TaskRemoved(task_id.to_string()));

            info!("Removed task: {}", task_id);
            Ok(())
        } else {
            Err(CliError::InputError(format!("Task not found: {}", task_id)))
        }
    }

    pub async fn list_tasks(&self) -> Vec<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    pub async fn get_task(&self, task_id: &str) -> Option<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    pub async fn enable_task(&mut self, task_id: &str) -> CliResult<()> {
        let mut modified = false;
        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(task_id) {
                task.enabled = true;
                modified = true;
            }
        }

        if modified {
            self.rebuild_queue().await;
            self.save_tasks_async().await?;

            // Notify scheduler about task modification
            let _ = self
                .event_tx
                .send(SchedulerEvent::TaskModified(task_id.to_string()));

            info!("Enabled task: {}", task_id);
            Ok(())
        } else {
            Err(CliError::InputError(format!("Task not found: {}", task_id)))
        }
    }

    pub async fn disable_task(&mut self, task_id: &str) -> CliResult<()> {
        let mut modified = false;
        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(task_id) {
                task.enabled = false;
                modified = true;
            }
        }

        if modified {
            self.rebuild_queue().await;
            self.save_tasks_async().await?;

            // Notify scheduler about task modification
            let _ = self
                .event_tx
                .send(SchedulerEvent::TaskModified(task_id.to_string()));

            info!("Disabled task: {}", task_id);
            Ok(())
        } else {
            Err(CliError::InputError(format!("Task not found: {}", task_id)))
        }
    }

    /// Event-driven reactive scheduler main loop
    pub async fn run_scheduler(&mut self) -> CliResult<()> {
        if self.bot.is_none() {
            return Err(CliError::InputError(
                "Bot not configured for scheduler".to_string(),
            ));
        }

        info!("Starting event-driven scheduler...");

        let mut event_rx = {
            let mut rx_guard = self.event_rx.write().await;
            rx_guard
                .take()
                .ok_or_else(|| CliError::InputError("Scheduler already running".to_string()))?
        };

        loop {
            let next_wakeup = self.calculate_next_wakeup().await;

            tokio::select! {
                // Handle timer-based wakeups
                _ = sleep_until(next_wakeup) => {
                    if let Err(e) = self.process_due_tasks().await {
                        error!("Error processing due tasks: {}", e);
                    }
                }

                // Handle dynamic events
                event = event_rx.recv() => {
                    match event {
                        Some(SchedulerEvent::TaskAdded(_)) |
                        Some(SchedulerEvent::TaskModified(_)) => {
                            // Recalculate next wakeup time
                            continue;
                        }
                        Some(SchedulerEvent::ForceWakeup) => {
                            if let Err(e) = self.process_due_tasks().await {
                                error!("Error processing forced tasks: {}", e);
                            }
                        }
                        Some(SchedulerEvent::Shutdown) | None => {
                            info!("Scheduler shutting down...");
                            break;
                        }
                        _ => {}
                    }
                }

                // Handle graceful shutdown signal
                _ = self.wait_for_shutdown() => {
                    info!("Received shutdown signal");
                    break;
                }
            }

            // Clean up completed tasks periodically
            if let Err(e) = self.cleanup_completed_tasks().await {
                error!("Error cleaning up tasks: {}", e);
            }
        }

        Ok(())
    }

    pub async fn run_task_once(&mut self, task_id: &str) -> CliResult<()> {
        if self.bot.is_none() {
            return Err(CliError::InputError(
                "Bot not configured for scheduler".to_string(),
            ));
        }

        let task_exists = {
            let tasks = self.tasks.read().await;
            tasks.contains_key(task_id)
        };

        if !task_exists {
            return Err(CliError::InputError(format!("Task not found: {}", task_id)));
        }

        self.execute_task(task_id).await
    }

    /// Process all tasks that are due for execution
    async fn process_due_tasks(&mut self) -> CliResult<()> {
        let ready_tasks = self.extract_ready_tasks().await;

        if ready_tasks.is_empty() {
            return Ok(());
        }

        info!("Processing {} due tasks", ready_tasks.len());

        // Execute tasks in parallel with timeout and concurrency control
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_tasks));
        let timeout = self.task_timeout;

        // Execute tasks sequentially to avoid borrow checker issues
        for task_id in ready_tasks {
            let _permit = semaphore
                .acquire()
                .await
                .map_err(|_| CliError::InputError("Semaphore acquire failed".to_string()))?;

            let result = tokio::time::timeout(timeout, self.execute_task(&task_id))
                .await
                .map_err(|_| CliError::InputError(format!("Task {} execution timeout", task_id)))?;

            if let Err(e) = result {
                error!("Task execution failed: {}", e);
            }
        }

        Ok(())
    }

    async fn execute_task(&mut self, task_id: &str) -> CliResult<()> {
        let task = {
            let tasks = self.tasks.read().await;
            tasks.get(task_id).cloned()
        };

        let task =
            task.ok_or_else(|| CliError::InputError(format!("Task not found: {}", task_id)))?;

        let bot = self.bot.as_ref().unwrap();

        debug!(
            "Executing task: {} ({})",
            task_id,
            task.task_type.description()
        );

        let result: CliResult<()> = match &task.task_type {
            TaskType::SendText { chat_id, message } => {
                let parser =
                    MessageTextParser::new().add(MessageTextFormat::Plain(message.clone()));
                let request = RequestMessagesSendText::new(ChatId(chat_id.clone()))
                    .set_text(parser)
                    .map_err(|e| CliError::InputError(format!("Failed to create message: {e}")))?;
                bot.send_api_request(request)
                    .await
                    .map_err(CliError::ApiError)
                    .map(|_| ())
            }
            TaskType::SendFile { chat_id, file_path } => {
                let request = RequestMessagesSendFile::new((
                    ChatId(chat_id.clone()),
                    MultipartName::FilePath(file_path.clone()),
                ));
                bot.send_api_request(request)
                    .await
                    .map_err(CliError::ApiError)
                    .map(|_| ())
            }
            TaskType::SendVoice { chat_id, file_path } => {
                let request = RequestMessagesSendVoice::new((
                    ChatId(chat_id.clone()),
                    MultipartName::FilePath(file_path.clone()),
                ));
                bot.send_api_request(request)
                    .await
                    .map_err(CliError::ApiError)
                    .map(|_| ())
            }
            TaskType::SendAction { chat_id, action } => {
                let chat_action = match action.as_str() {
                    "typing" => ChatActions::Typing,
                    "looking" => ChatActions::Looking,
                    _ => return Err(CliError::InputError(format!("Unknown action: {}", action))),
                };
                let request = RequestChatsSendAction::new((ChatId(chat_id.clone()), chat_action));
                bot.send_api_request(request)
                    .await
                    .map_err(CliError::ApiError)
                    .map(|_| ())
            }
        };

        // Update task statistics after execution
        if result.is_ok() {
            info!("Successfully executed task: {}", task_id);
            self.update_task_after_execution(task_id).await?;
        } else {
            error!("Failed to execute task {}: {:?}", task_id, result);
        }

        result
    }

    async fn update_task_after_execution(&mut self, task_id: &str) -> CliResult<()> {
        let now = Utc::now();
        let mut should_save = false;
        let mut task_to_remove = None;

        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(task_id) {
                task.last_run = Some(now);
                task.run_count += 1;

                // Check if max runs exceeded
                if let Some(max_runs) = task.max_runs {
                    if task.run_count >= max_runs {
                        info!("Task {} completed all {} runs", task_id, max_runs);
                        task_to_remove = Some(task_id.to_string());
                    }
                }

                // Calculate next run time for recurring tasks
                if task_to_remove.is_none() {
                    match task.schedule {
                        ScheduleType::Once(_) => {
                            // One-time task completed
                            task_to_remove = Some(task_id.to_string());
                        }
                        _ => {
                            // Recurring task - calculate next run
                            if let Ok(next_run) = self.calculate_next_run(&task.schedule, Some(now))
                            {
                                task.next_run = next_run;
                            } else {
                                error!("Failed to calculate next run for task {}", task_id);
                                task_to_remove = Some(task_id.to_string());
                            }
                        }
                    }
                }

                should_save = true;
            }
        }

        // Remove completed tasks
        if let Some(task_id_to_remove) = task_to_remove {
            let mut tasks = self.tasks.write().await;
            tasks.remove(&task_id_to_remove);
            info!("Removed completed task: {}", task_id_to_remove);
        }

        if should_save {
            self.rebuild_queue().await;
            self.save_tasks_async().await?;
        }

        Ok(())
    }

    pub async fn extract_ready_tasks(&self) -> Vec<String> {
        let now = Utc::now();
        let mut ready_tasks = Vec::new();

        let tasks = self.tasks.read().await;
        for task in tasks.values() {
            if task.enabled && task.next_run <= now {
                // Check if max runs exceeded
                if let Some(max_runs) = task.max_runs {
                    if task.run_count >= max_runs {
                        continue;
                    }
                }
                ready_tasks.push(task.id.clone());
            }
        }

        ready_tasks
    }

    pub async fn calculate_next_wakeup(&self) -> Instant {
        let now = Instant::now();
        let mut next_wakeup = now + TokioDuration::from_secs(60); // Default: 1 minute

        let tasks = self.tasks.read().await;
        for task in tasks.values() {
            if task.enabled {
                if let Some(max_runs) = task.max_runs {
                    if task.run_count >= max_runs {
                        continue;
                    }
                }

                // Calculate time until this task should run
                let duration_until_run = task.next_run.signed_duration_since(Utc::now());
                if let Ok(tokio_duration) = duration_until_run.to_std() {
                    let task_instant = now + tokio_duration;
                    if task_instant < next_wakeup {
                        next_wakeup = task_instant;
                    }
                }
            }
        }

        next_wakeup
    }

    async fn wait_for_shutdown(&self) -> () {
        while !self.shutdown_signal.load(Ordering::Relaxed) {
            tokio::time::sleep(TokioDuration::from_millis(100)).await;
        }
    }

    pub async fn shutdown(&self) {
        self.shutdown_signal.store(true, Ordering::Relaxed);
        let _ = self.event_tx.send(SchedulerEvent::Shutdown);
    }

    async fn add_to_queue(&self, task: ScheduledTask) {
        if !task.enabled {
            return;
        }

        let duration_until_run = task.next_run.signed_duration_since(Utc::now());
        if let Ok(tokio_duration) = duration_until_run.to_std() {
            let next_run_instant = Instant::now() + tokio_duration;
            let wrapper = ScheduledTaskWrapper {
                task_id: task.id.clone(),
                next_run_instant,
            };

            let mut queue = self.task_queue.write().await;
            queue.push(wrapper);
        }
    }

    pub async fn rebuild_queue(&self) {
        let mut queue = self.task_queue.write().await;
        queue.clear();

        let tasks = self.tasks.read().await;
        for task in tasks.values() {
            if task.enabled {
                let duration_until_run = task.next_run.signed_duration_since(Utc::now());
                if let Ok(tokio_duration) = duration_until_run.to_std() {
                    let next_run_instant = Instant::now() + tokio_duration;
                    let wrapper = ScheduledTaskWrapper {
                        task_id: task.id.clone(),
                        next_run_instant,
                    };
                    queue.push(wrapper);
                }
            }
        }
    }

    async fn cleanup_completed_tasks(&mut self) -> CliResult<()> {
        let mut tasks_to_remove = Vec::new();

        {
            let tasks = self.tasks.read().await;
            for (task_id, task) in tasks.iter() {
                if !task.enabled {
                    match &task.schedule {
                        ScheduleType::Once(_) => {
                            // Remove completed one-time tasks
                            tasks_to_remove.push(task_id.clone());
                        }
                        _ => {
                            // Keep disabled recurring tasks (user might want to re-enable them)
                        }
                    }
                }
            }
        }

        if !tasks_to_remove.is_empty() {
            let mut tasks = self.tasks.write().await;
            for task_id in &tasks_to_remove {
                tasks.remove(task_id);
                debug!("Cleaned up completed task: {}", task_id);
            }

            if !tasks.is_empty() {
                drop(tasks); // Release the lock before async operations
                self.save_tasks_async().await?;
            }
        }

        Ok(())
    }

    async fn load_tasks_async(&mut self) -> CliResult<()> {
        if !self.data_file.exists() {
            debug!("No scheduler data file found, starting with empty task list");
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&self.data_file)
            .await
            .map_err(|e| CliError::FileError(format!("Could not read scheduler data: {e}")))?;

        let tasks: HashMap<String, ScheduledTask> =
            tokio::task::spawn_blocking(move || serde_json::from_str(&content))
                .await
                .map_err(|e| CliError::UnexpectedError(format!("JSON parsing task failed: {e}")))?
                .map_err(|e| {
                    CliError::UnexpectedError(format!("Could not parse scheduler data: {e}"))
                })?;

        {
            let mut task_guard = self.tasks.write().await;
            *task_guard = tasks;
        }

        self.rebuild_queue().await;

        let task_count = {
            let tasks = self.tasks.read().await;
            tasks.len()
        };

        info!("Loaded {} scheduled tasks", task_count);
        Ok(())
    }

    async fn save_tasks_async(&self) -> CliResult<()> {
        let tasks = {
            let tasks = self.tasks.read().await;
            tasks.clone()
        };

        let task_count = tasks.len();
        let data_file = self.data_file.clone();

        let content = tokio::task::spawn_blocking(move || serde_json::to_string_pretty(&tasks))
            .await
            .map_err(|e| CliError::UnexpectedError(format!("JSON serialization task failed: {e}")))?
            .map_err(|e| CliError::UnexpectedError(format!("Could not serialize tasks: {e}")))?;

        tokio::fs::write(&data_file, content)
            .await
            .map_err(|e| CliError::FileError(format!("Could not write scheduler data: {e}")))?;

        debug!("Saved {} tasks to scheduler data file", task_count);
        Ok(())
    }

    fn calculate_next_run(
        &self,
        schedule: &ScheduleType,
        from_time: Option<DateTime<Utc>>,
    ) -> CliResult<DateTime<Utc>> {
        let now = from_time.unwrap_or_else(Utc::now);

        match schedule {
            ScheduleType::Once(time) => Ok(*time),
            ScheduleType::Cron(expr) => {
                let schedule = Schedule::from_str(expr)
                    .map_err(|e| CliError::InputError(format!("Invalid cron expression: {e}")))?;
                schedule
                    .upcoming(Utc)
                    .next()
                    .ok_or_else(|| CliError::InputError("No upcoming cron execution".to_string()))
            }
            ScheduleType::Interval {
                duration_seconds,
                start_time,
            } => {
                let interval = Duration::seconds(*duration_seconds as i64);
                let mut next_run = *start_time;

                while next_run <= now {
                    next_run = next_run + interval;
                }

                Ok(next_run)
            }
        }
    }

    #[allow(dead_code)]
    pub async fn create_test_scheduler() -> (Scheduler, tempfile::TempDir) {
        let temp_dir = tempdir().unwrap();
        let mut data_file = PathBuf::from(temp_dir.path());
        data_file.push("scheduler_tasks_test.json");
        if let Some(parent) = data_file.parent() {
            tokio::fs::create_dir_all(parent).await.unwrap();
        }

        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let scheduler = Scheduler {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            data_file,
            bot: None,
            event_tx,
            event_rx: Arc::new(RwLock::new(Some(event_rx))),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            max_concurrent_tasks: 10,
            task_timeout: TokioDuration::from_secs(300),
        };
        (scheduler, temp_dir)
    }
}

impl TaskType {
    pub fn description(&self) -> String {
        match self {
            TaskType::SendText { chat_id, .. } => format!("Send text to {}", chat_id),
            TaskType::SendFile { chat_id, file_path } => {
                format!("Send file {} to {}", file_path, chat_id)
            }
            TaskType::SendVoice { chat_id, file_path } => {
                format!("Send voice {} to {}", file_path, chat_id)
            }
            TaskType::SendAction { chat_id, action } => {
                format!("Send {} action to {}", action, chat_id)
            }
        }
    }
}

impl ScheduleType {
    pub fn description(&self) -> String {
        match self {
            ScheduleType::Once(time) => format!("Once at {}", time.format("%Y-%m-%d %H:%M:%S UTC")),
            ScheduleType::Cron(expr) => format!("Cron: {}", expr),
            ScheduleType::Interval {
                duration_seconds,
                start_time,
            } => format!(
                "Every {} seconds starting from {}",
                duration_seconds,
                start_time.format("%Y-%m-%d %H:%M:%S UTC")
            ),
        }
    }

    pub fn next_run_time(&self, from_time: DateTime<Utc>) -> CliResult<DateTime<Utc>> {
        match self {
            ScheduleType::Once(time) => Ok(*time),
            ScheduleType::Cron(expr) => {
                let schedule = Schedule::from_str(expr)
                    .map_err(|e| CliError::InputError(format!("Invalid cron expression: {e}")))?;
                schedule
                    .upcoming(Utc)
                    .next()
                    .ok_or_else(|| CliError::InputError("No upcoming cron execution".to_string()))
            }
            ScheduleType::Interval {
                duration_seconds,
                start_time,
            } => {
                let interval = Duration::seconds(*duration_seconds as i64);
                let mut next_run = *start_time;

                while next_run <= from_time {
                    next_run = next_run + interval;
                }

                Ok(next_run)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let (scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;
        assert_eq!(scheduler.list_tasks().await.len(), 0);
    }

    #[tokio::test]
    async fn test_add_and_list_tasks() {
        let (mut scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

        let task_id = scheduler
            .add_task(
                TaskType::SendText {
                    chat_id: "test_chat".to_string(),
                    message: "test message".to_string(),
                },
                ScheduleType::Once(Utc::now() + Duration::hours(1)),
                None,
            )
            .await
            .unwrap();

        let tasks = scheduler.list_tasks().await;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, task_id);
    }

    #[tokio::test]
    async fn test_task_enabling_disabling() {
        let (mut scheduler, _temp_dir) = Scheduler::create_test_scheduler().await;

        let task_id = scheduler
            .add_task(
                TaskType::SendText {
                    chat_id: "test_chat".to_string(),
                    message: "test message".to_string(),
                },
                ScheduleType::Once(Utc::now() + Duration::hours(1)),
                None,
            )
            .await
            .unwrap();

        // Disable task
        scheduler.disable_task(&task_id).await.unwrap();
        let task = scheduler.get_task(&task_id).await.unwrap();
        assert!(!task.enabled);

        // Enable task
        scheduler.enable_task(&task_id).await.unwrap();
        let task = scheduler.get_task(&task_id).await.unwrap();
        assert!(task.enabled);
    }

    #[tokio::test]
    async fn test_cron_schedule() {
        let schedule = ScheduleType::Cron("0 0 0 * * *".to_string()); // Daily at midnight (sec min hour day month dayofweek)
        let next_run = schedule.next_run_time(Utc::now()).unwrap();
        assert!(next_run > Utc::now());
    }

    #[tokio::test]
    async fn test_interval_schedule() {
        let start_time = Utc::now() - Duration::hours(1);
        let schedule = ScheduleType::Interval {
            duration_seconds: 3600, // 1 hour
            start_time,
        };
        let next_run = schedule.next_run_time(Utc::now()).unwrap();
        assert!(next_run > Utc::now());
        assert!(next_run <= Utc::now() + Duration::hours(1));
    }

    #[tokio::test]
    async fn test_task_persistence() {
        let temp_dir = tempdir().unwrap();
        let _temp_dir = tempdir().unwrap();

        {
            let mut scheduler = Scheduler::new(Some(temp_dir.path().to_path_buf()))
                .await
                .unwrap();
            scheduler
                .add_task(
                    TaskType::SendText {
                        chat_id: "test_chat".to_string(),
                        message: "test message".to_string(),
                    },
                    ScheduleType::Once(Utc::now() + Duration::hours(1)),
                    None,
                )
                .await
                .unwrap();
        }

        // Create new scheduler instance
        let scheduler = Scheduler::new(Some(temp_dir.path().to_path_buf()))
            .await
            .unwrap();
        assert_eq!(scheduler.list_tasks().await.len(), 1);
    }
}
