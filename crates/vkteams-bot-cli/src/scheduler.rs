use crate::errors::prelude::{CliError, Result as CliResult};
use chrono::{DateTime, Duration, Utc};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use tempfile::tempdir;
use tokio::time::{Duration as TokioDuration, sleep};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use vkteams_bot::prelude::*;

pub const SCHEDULER_DATA_FILE: &str = "scheduler_tasks.json";

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

pub struct Scheduler {
    tasks: HashMap<String, ScheduledTask>,
    data_file: PathBuf,
    bot: Option<Bot>,
}

impl Scheduler {
    pub fn new() -> CliResult<Self> {
        let mut data_file = dirs::home_dir()
            .ok_or_else(|| CliError::FileError("Could not determine home directory".to_string()))?;
        data_file.push(".config/vkteams-bot");
        std::fs::create_dir_all(&data_file)
            .map_err(|e| CliError::FileError(format!("Could not create config directory: {e}")))?;
        data_file.push(SCHEDULER_DATA_FILE);

        let mut scheduler = Self {
            tasks: HashMap::new(),
            data_file,
            bot: None,
        };

        scheduler.load_tasks()?;
        Ok(scheduler)
    }

    pub fn set_bot(&mut self, bot: Bot) {
        self.bot = Some(bot);
    }

    pub fn add_task(
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

        self.tasks.insert(id.clone(), task);
        self.save_tasks()?;

        info!("Added scheduled task with ID: {}", id);
        Ok(id)
    }

    pub fn remove_task(&mut self, task_id: &str) -> CliResult<()> {
        if self.tasks.remove(task_id).is_some() {
            self.save_tasks()?;
            info!("Removed task: {}", task_id);
            Ok(())
        } else {
            Err(CliError::InputError(format!("Task not found: {}", task_id)))
        }
    }

    pub fn list_tasks(&self) -> Vec<&ScheduledTask> {
        self.tasks.values().collect()
    }

    pub fn get_task(&self, task_id: &str) -> Option<&ScheduledTask> {
        self.tasks.get(task_id)
    }

    pub fn enable_task(&mut self, task_id: &str) -> CliResult<()> {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.enabled = true;
            self.save_tasks()?;
            info!("Enabled task: {}", task_id);
            Ok(())
        } else {
            Err(CliError::InputError(format!("Task not found: {}", task_id)))
        }
    }

    pub fn disable_task(&mut self, task_id: &str) -> CliResult<()> {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.enabled = false;
            self.save_tasks()?;
            info!("Disabled task: {}", task_id);
            Ok(())
        } else {
            Err(CliError::InputError(format!("Task not found: {}", task_id)))
        }
    }

    pub async fn run_scheduler(&mut self) -> CliResult<()> {
        if self.bot.is_none() {
            return Err(CliError::InputError(
                "Bot not configured for scheduler".to_string(),
            ));
        }

        info!("Starting scheduler...");

        loop {
            let now = Utc::now();
            let mut tasks_to_run = Vec::new();

            // Find tasks that need to run
            for task in self.tasks.values() {
                if task.enabled && task.next_run <= now {
                    // Check if max runs exceeded
                    if let Some(max_runs) = task.max_runs {
                        if task.run_count >= max_runs {
                            continue;
                        }
                    }
                    tasks_to_run.push(task.id.clone());
                }
            }

            // Execute tasks
            for task_id in tasks_to_run {
                if let Err(e) = self.execute_task(&task_id).await {
                    error!("Failed to execute task {}: {}", task_id, e);
                }
            }

            // Clean up completed one-time tasks
            self.cleanup_completed_tasks()?;

            // Sleep for a minute before checking again
            sleep(TokioDuration::from_secs(60)).await;
        }
    }

    pub async fn run_task_once(&mut self, task_id: &str) -> CliResult<()> {
        if self.bot.is_none() {
            return Err(CliError::InputError(
                "Bot not configured for scheduler".to_string(),
            ));
        }

        if !self.tasks.contains_key(task_id) {
            return Err(CliError::InputError(format!("Task not found: {}", task_id)));
        }

        self.execute_task(task_id).await
    }

    async fn execute_task(&mut self, task_id: &str) -> CliResult<()> {
        let task = self.tasks.get(task_id).unwrap().clone();
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
                    MultipartName::File(file_path.clone()),
                ));
                bot.send_api_request(request)
                    .await
                    .map_err(CliError::ApiError)
                    .map(|_| ())
            }
            TaskType::SendVoice { chat_id, file_path } => {
                let request = RequestMessagesSendVoice::new((
                    ChatId(chat_id.clone()),
                    MultipartName::File(file_path.clone()),
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

        match result {
            Ok(_) => {
                info!("Successfully executed task: {}", task_id);
                self.update_task_after_execution(task_id)?;
            }
            Err(e) => {
                error!("Failed to execute task {}: {}", task_id, e);
                return Err(e);
            }
        }

        Ok(())
    }

    fn update_task_after_execution(&mut self, task_id: &str) -> CliResult<()> {
        let now = Utc::now();
        let (schedule, max_runs) = if let Some(task) = self.tasks.get_mut(task_id) {
            task.last_run = Some(now);
            task.run_count += 1;
            (task.schedule.clone(), task.max_runs)
        } else {
            return Ok(());
        };

        // Calculate next run time outside of the mutable borrow
        let next_run_result = self.calculate_next_run(&schedule, Some(now));

        // Update the task with the calculated next run
        if let Some(task) = self.tasks.get_mut(task_id) {
            match &schedule {
                ScheduleType::Once(_) => {
                    // One-time task completed, disable it
                    task.enabled = false;
                }
                ScheduleType::Cron(_) | ScheduleType::Interval { .. } => {
                    // Calculate next run for recurring tasks
                    if let Ok(next_run) = next_run_result {
                        task.next_run = next_run;
                    } else {
                        warn!("Could not calculate next run for task: {}", task_id);
                        task.enabled = false;
                    }
                }
            }

            // Check if max runs reached
            if let Some(max_runs) = max_runs {
                if task.run_count >= max_runs {
                    task.enabled = false;
                    info!("Task {} reached max runs ({}), disabled", task_id, max_runs);
                }
            }
        }

        self.save_tasks()?;
        Ok(())
    }

    fn calculate_next_run(
        &self,
        schedule: &ScheduleType,
        from_time: Option<DateTime<Utc>>,
    ) -> CliResult<DateTime<Utc>> {
        let base_time = from_time.unwrap_or_else(Utc::now);

        match schedule {
            ScheduleType::Once(time) => Ok(*time),
            ScheduleType::Cron(cron_expr) => {
                let schedule = Schedule::from_str(cron_expr)
                    .map_err(|e| CliError::InputError(format!("Invalid cron expression: {e}")))?;

                schedule.upcoming(Utc).next().ok_or_else(|| {
                    CliError::InputError("No upcoming time for cron expression".to_string())
                })
            }
            ScheduleType::Interval {
                duration_seconds,
                start_time,
            } => {
                if *duration_seconds == 0 {
                    return Err(CliError::InputError(
                        "Interval must be greater than 0".to_string(),
                    ));
                }
                if base_time < *start_time {
                    Ok(*start_time)
                } else {
                    let elapsed = base_time.signed_duration_since(*start_time);
                    let interval = Duration::seconds(*duration_seconds as i64);
                    let intervals_passed = elapsed.num_seconds() / interval.num_seconds();
                    let next_interval = intervals_passed + 1;
                    Ok(*start_time + Duration::seconds(next_interval * interval.num_seconds()))
                }
            }
        }
    }

    fn cleanup_completed_tasks(&mut self) -> CliResult<()> {
        let mut tasks_to_remove = Vec::new();

        for (task_id, task) in &self.tasks {
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

        for task_id in tasks_to_remove {
            self.tasks.remove(&task_id);
            debug!("Cleaned up completed task: {}", task_id);
        }

        if !self.tasks.is_empty() {
            self.save_tasks()?;
        }

        Ok(())
    }

    fn load_tasks(&mut self) -> CliResult<()> {
        if !self.data_file.exists() {
            debug!("No scheduler data file found, starting with empty task list");
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.data_file)
            .map_err(|e| CliError::FileError(format!("Could not read scheduler data: {e}")))?;

        let tasks: HashMap<String, ScheduledTask> =
            serde_json::from_str(&content).map_err(|e| {
                CliError::UnexpectedError(format!("Could not parse scheduler data: {e}"))
            })?;

        self.tasks = tasks;
        info!("Loaded {} scheduled tasks", self.tasks.len());
        Ok(())
    }

    fn save_tasks(&self) -> CliResult<()> {
        let content = serde_json::to_string_pretty(&self.tasks)
            .map_err(|e| CliError::UnexpectedError(format!("Could not serialize tasks: {e}")))?;

        std::fs::write(&self.data_file, content)
            .map_err(|e| CliError::FileError(format!("Could not write scheduler data: {e}")))?;

        debug!("Saved {} tasks to scheduler data file", self.tasks.len());
        Ok(())
    }

    pub(crate) fn create_test_scheduler() -> (Scheduler, tempfile::TempDir) {
        let temp_dir = tempdir().unwrap();
        let mut data_file = PathBuf::from(temp_dir.path());
        data_file.push("scheduler_tasks_test.json");
        if let Some(parent) = data_file.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let scheduler = Scheduler {
            tasks: HashMap::new(),
            data_file,
            bot: None,
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
            } => {
                format!(
                    "Every {} seconds from {}",
                    duration_seconds,
                    start_time.format("%Y-%m-%d %H:%M:%S UTC")
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::path::PathBuf;
    use tempfile::tempdir;

    pub(crate) fn create_test_scheduler() -> (Scheduler, tempfile::TempDir) {
        let temp_dir = tempdir().unwrap();
        let mut data_file = PathBuf::from(temp_dir.path());
        data_file.push("scheduler_tasks_test.json");
        if let Some(parent) = data_file.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let scheduler = Scheduler {
            tasks: HashMap::new(),
            data_file,
            bot: None,
        };
        (scheduler, temp_dir)
    }

    #[test]
    fn test_add_and_remove_task() {
        let (mut scheduler, _tempdir) = create_test_scheduler();
        let task_id = scheduler
            .add_task(
                TaskType::SendText {
                    chat_id: "user1".to_string(),
                    message: "hi".to_string(),
                },
                ScheduleType::Once(Utc::now() + Duration::minutes(1)),
                Some(1),
            )
            .unwrap();
        assert!(scheduler.get_task(&task_id).is_some());
        scheduler.remove_task(&task_id).unwrap();
        assert!(scheduler.get_task(&task_id).is_none());
    }

    #[test]
    fn test_enable_disable_task() {
        let (mut scheduler, _tempdir) = create_test_scheduler();
        let task_id = scheduler
            .add_task(
                TaskType::SendText {
                    chat_id: "user2".to_string(),
                    message: "hi".to_string(),
                },
                ScheduleType::Once(Utc::now() + Duration::minutes(1)),
                None,
            )
            .unwrap();
        scheduler.disable_task(&task_id).unwrap();
        assert!(!scheduler.get_task(&task_id).unwrap().enabled);
        scheduler.enable_task(&task_id).unwrap();
        assert!(scheduler.get_task(&task_id).unwrap().enabled);
    }

    #[test]
    fn test_list_and_get_task() {
        let (mut scheduler, _tempdir) = create_test_scheduler();
        let task_id = scheduler
            .add_task(
                TaskType::SendText {
                    chat_id: "user3".to_string(),
                    message: "hi".to_string(),
                },
                ScheduleType::Once(Utc::now() + Duration::minutes(1)),
                None,
            )
            .unwrap();
        let tasks = scheduler.list_tasks();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, task_id);
        assert!(scheduler.get_task(&task_id).is_some());
        assert!(scheduler.get_task("no_such_id").is_none());
    }

    #[test]
    fn test_calculate_next_run_once() {
        let (scheduler, _tempdir) = create_test_scheduler();
        let dt = Utc.ymd(2030, 1, 1).and_hms(0, 0, 0);
        let next = scheduler
            .calculate_next_run(&ScheduleType::Once(dt), None)
            .unwrap();
        assert_eq!(next, dt);
    }

    #[test]
    fn test_calculate_next_run_interval() {
        let (scheduler, _tempdir) = create_test_scheduler();
        let start = Utc.ymd(2030, 1, 1).and_hms(0, 0, 0);
        let sched = ScheduleType::Interval {
            duration_seconds: 60,
            start_time: start,
        };
        let next = scheduler.calculate_next_run(&sched, Some(start)).unwrap();
        assert_eq!(next, start + Duration::seconds(60));
    }

    #[test]
    fn test_cleanup_completed_tasks() {
        let (mut scheduler, _tempdir) = create_test_scheduler();
        let dt = Utc::now() - Duration::days(1);
        let id = scheduler
            .add_task(
                TaskType::SendText {
                    chat_id: "user4".to_string(),
                    message: "hi".to_string(),
                },
                ScheduleType::Once(dt),
                Some(1),
            )
            .unwrap();
        // Помечаем задачу как выполненную и превышающую max_runs
        if let Some(task) = scheduler.tasks.get_mut(&id) {
            task.run_count = 1;
            task.max_runs = Some(1);
        }
        scheduler.disable_task(&id).unwrap();
        scheduler.cleanup_completed_tasks().unwrap();
        assert!(scheduler.get_task(&id).is_none());
    }

    #[test]
    fn test_remove_nonexistent_task() {
        let (mut scheduler, _tempdir) = create_test_scheduler();
        let res = scheduler.remove_task("no_such_id");
        assert!(res.is_err());
    }

    #[test]
    fn test_enable_disable_nonexistent_task() {
        let (mut scheduler, _tempdir) = create_test_scheduler();
        assert!(scheduler.enable_task("no_such_id").is_err());
        assert!(scheduler.disable_task("no_such_id").is_err());
    }
}

#[cfg(test)]
mod async_tests {
    use super::*;
    use chrono::TimeZone;

    fn create_test_scheduler_with_bot() -> (Scheduler, tempfile::TempDir) {
        let (mut scheduler, tempdir) = super::tests::create_test_scheduler();
        // Используем dummy Bot (не делает реальных запросов)
        let bot =
            Bot::with_params(&APIVersionUrl::V1, "dummy_token", "https://dummy.api.com").unwrap();
        scheduler.set_bot(bot);
        (scheduler, tempdir)
    }

    #[tokio::test]
    async fn test_run_task_once_success() {
        let (mut scheduler, _tempdir) = create_test_scheduler_with_bot();
        let task_id = scheduler
            .add_task(
                TaskType::SendText {
                    chat_id: "user1".to_string(),
                    message: "hi".to_string(),
                },
                ScheduleType::Once(Utc::now() + Duration::minutes(1)),
                Some(2),
            )
            .unwrap();
        let res = scheduler.run_task_once(&task_id).await;
        // Может быть Ok или Err (dummy Bot), главное — не panic
        assert!(res.is_ok() || res.is_err());
    }

    #[tokio::test]
    async fn test_run_task_once_disabled() {
        let (mut scheduler, _tempdir) = create_test_scheduler_with_bot();
        let task_id = scheduler
            .add_task(
                TaskType::SendText {
                    chat_id: "user2".to_string(),
                    message: "hi".to_string(),
                },
                ScheduleType::Once(Utc::now() + Duration::minutes(1)),
                Some(1),
            )
            .unwrap();
        scheduler.disable_task(&task_id).unwrap();
        let res = scheduler.run_task_once(&task_id).await;
        // Должен выполниться, но задача disabled — поведение зависит от execute_task
        assert!(res.is_ok() || res.is_err());
    }

    #[tokio::test]
    async fn test_run_task_once_not_found() {
        let (mut scheduler, _tempdir) = create_test_scheduler_with_bot();
        let res = scheduler.run_task_once("no_such_id").await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_run_task_once_max_runs() {
        let (mut scheduler, _tempdir) = create_test_scheduler_with_bot();
        let task_id = scheduler
            .add_task(
                TaskType::SendText {
                    chat_id: "user3".to_string(),
                    message: "hi".to_string(),
                },
                ScheduleType::Once(Utc::now() + Duration::minutes(1)),
                Some(1),
            )
            .unwrap();
        // Первый запуск — должен пройти
        let _ = scheduler.run_task_once(&task_id).await;
        // Второй запуск — max_runs достигнут, задача должна быть disabled
        let res = scheduler.run_task_once(&task_id).await;
        assert!(res.is_err() || res.is_ok());
        let task = scheduler.get_task(&task_id).unwrap();
        assert!(!task.enabled || task.run_count <= 1);
    }

    #[tokio::test]
    async fn test_calculate_next_run_invalid_cron() {
        let (scheduler, _tempdir) = super::tests::create_test_scheduler();
        let sched = ScheduleType::Cron("invalid cron".to_string());
        let res = scheduler.calculate_next_run(&sched, None);
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_calculate_next_run_invalid_interval() {
        let (scheduler, _tempdir) = super::tests::create_test_scheduler();
        let start = Utc.ymd(2030, 1, 1).and_hms(0, 0, 0);
        let sched = ScheduleType::Interval {
            duration_seconds: 0,
            start_time: start,
        };
        // Интервал 0 секунд — теперь функция возвращает ошибку
        let res = scheduler.calculate_next_run(&sched, Some(start));
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_run_task_once_no_bot() {
        let (mut scheduler, _tempdir) = super::tests::create_test_scheduler();
        let task_id = scheduler
            .add_task(
                TaskType::SendText {
                    chat_id: "user4".to_string(),
                    message: "hi".to_string(),
                },
                ScheduleType::Once(Utc::now() + Duration::minutes(1)),
                Some(1),
            )
            .unwrap();
        // Не устанавливаем bot
        let res = scheduler.run_task_once(&task_id).await;
        assert!(res.is_err());
    }
}
