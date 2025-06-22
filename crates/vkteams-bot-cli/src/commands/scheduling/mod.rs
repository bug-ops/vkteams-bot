//! Scheduling commands module
//!
//! This module contains all commands related to message scheduling and task management.

use crate::commands::Command;
use crate::errors::prelude::{CliError, Result as CliResult};
use crate::scheduler::{ScheduleType, Scheduler, TaskType};
use crate::utils::parse_schedule_time;
use async_trait::async_trait;
use chrono::Utc;
use clap::{Subcommand, ValueHint};
use colored::Colorize;
use std::str::FromStr;
use vkteams_bot::prelude::*;

/// All scheduling-related commands
#[derive(Subcommand, Debug, Clone)]
pub enum SchedulingCommands {
    /// Schedule a message to be sent later
    Schedule {
        #[command(subcommand)]
        message_type: ScheduleMessageType,
    },
    /// Manage the scheduler service
    Scheduler {
        #[command(subcommand)]
        action: SchedulerAction,
    },
    /// Manage scheduled tasks
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ScheduleMessageType {
    /// Schedule a text message
    Text {
        #[arg(short = 'u', long, required = true, value_name = "CHAT_ID", value_hint = ValueHint::Username)]
        chat_id: String,
        #[arg(short = 'm', long, required = true, value_name = "MESSAGE")]
        message: String,
        #[arg(short = 't', long, value_name = "TIME")]
        time: Option<String>,
        #[arg(short = 'c', long, value_name = "CRON")]
        cron: Option<String>,
        #[arg(short = 'i', long, value_name = "SECONDS")]
        interval: Option<u64>,
        #[arg(long, value_name = "RUNS")]
        max_runs: Option<u64>,
    },
    /// Schedule a file message
    File {
        #[arg(short = 'u', long, required = true, value_name = "CHAT_ID", value_hint = ValueHint::Username)]
        chat_id: String,
        #[arg(short = 'p', long, required = true, value_name = "FILE_PATH", value_hint = ValueHint::FilePath)]
        file_path: String,
        #[arg(short = 't', long, value_name = "TIME")]
        time: Option<String>,
        #[arg(short = 'c', long, value_name = "CRON")]
        cron: Option<String>,
        #[arg(short = 'i', long, value_name = "SECONDS")]
        interval: Option<u64>,
        #[arg(long, value_name = "RUNS")]
        max_runs: Option<u64>,
    },
    /// Schedule a voice message
    Voice {
        #[arg(short = 'u', long, required = true, value_name = "CHAT_ID", value_hint = ValueHint::Username)]
        chat_id: String,
        #[arg(short = 'p', long, required = true, value_name = "FILE_PATH", value_hint = ValueHint::FilePath)]
        file_path: String,
        #[arg(short = 't', long, value_name = "TIME")]
        time: Option<String>,
        #[arg(short = 'c', long, value_name = "CRON")]
        cron: Option<String>,
        #[arg(short = 'i', long, value_name = "SECONDS")]
        interval: Option<u64>,
        #[arg(long, value_name = "RUNS")]
        max_runs: Option<u64>,
    },
    /// Schedule a chat action
    Action {
        #[arg(short = 'u', long, required = true, value_name = "CHAT_ID", value_hint = ValueHint::Username)]
        chat_id: String,
        #[arg(short = 'a', long, required = true, value_name = "ACTION")]
        action: String,
        #[arg(short = 't', long, value_name = "TIME")]
        time: Option<String>,
        #[arg(short = 'c', long, value_name = "CRON")]
        cron: Option<String>,
        #[arg(short = 'i', long, value_name = "SECONDS")]
        interval: Option<u64>,
        #[arg(long, value_name = "RUNS")]
        max_runs: Option<u64>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum SchedulerAction {
    /// Start the scheduler daemon
    Start,
    /// Stop the scheduler daemon
    Stop,
    /// Show scheduler status
    Status,
    /// List all scheduled tasks
    List,
}

#[derive(Subcommand, Debug, Clone)]
pub enum TaskAction {
    /// Show details of a specific task
    Show {
        #[arg(required = true, value_name = "TASK_ID")]
        task_id: String,
    },
    /// Remove a scheduled task
    Remove {
        #[arg(required = true, value_name = "TASK_ID")]
        task_id: String,
    },
    /// Enable a disabled task
    Enable {
        #[arg(required = true, value_name = "TASK_ID")]
        task_id: String,
    },
    /// Disable an active task
    Disable {
        #[arg(required = true, value_name = "TASK_ID")]
        task_id: String,
    },
    /// Run a task immediately (one-time)
    Run {
        #[arg(required = true, value_name = "TASK_ID")]
        task_id: String,
    },
}

#[async_trait]
impl Command for SchedulingCommands {
    async fn execute(&self, bot: &Bot) -> CliResult<()> {
        match self {
            SchedulingCommands::Schedule { message_type } => {
                execute_schedule(bot, message_type).await
            }
            SchedulingCommands::Scheduler { action } => execute_scheduler_action(bot, action).await,
            SchedulingCommands::Task { action } => execute_task_action(bot, action).await,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            SchedulingCommands::Schedule { .. } => "schedule",
            SchedulingCommands::Scheduler { .. } => "scheduler",
            SchedulingCommands::Task { .. } => "task",
        }
    }

    fn validate(&self) -> CliResult<()> {
        match self {
            SchedulingCommands::Schedule { message_type } => {
                validate_schedule_command(message_type)
            }
            SchedulingCommands::Scheduler { action } => {
                validate_scheduler_action(action)
            }
            SchedulingCommands::Task { action } => {
                validate_task_action(action)
            }
        }
    }
}

// Command execution functions
async fn execute_schedule(_bot: &Bot, message_type: &ScheduleMessageType) -> CliResult<()> {
    let mut scheduler = Scheduler::new(None).await?;
    // Note: We need to create a new bot instance for the scheduler
    // since Bot doesn't implement Clone
    let token = std::env::var("VKTEAMS_BOT_API_TOKEN")
        .map_err(|_| CliError::InputError("Bot token not available".to_string()))?;
    let url = std::env::var("VKTEAMS_BOT_API_URL")
        .map_err(|_| CliError::InputError("Bot URL not available".to_string()))?;
    let scheduler_bot =
        Bot::with_params(&APIVersionUrl::V1, &token, &url).map_err(CliError::ApiError)?;
    scheduler.set_bot(scheduler_bot);

    let (task_type, schedule, max_runs) = match message_type {
        ScheduleMessageType::Text {
            chat_id,
            message,
            time,
            cron,
            interval,
            max_runs,
        } => {
            let task = TaskType::SendText {
                chat_id: chat_id.clone(),
                message: message.clone(),
            };
            let schedule = parse_schedule_args(time, cron, interval)?;
            (task, schedule, *max_runs)
        }
        ScheduleMessageType::File {
            chat_id,
            file_path,
            time,
            cron,
            interval,
            max_runs,
        } => {
            let task = TaskType::SendFile {
                chat_id: chat_id.clone(),
                file_path: file_path.clone(),
            };
            let schedule = parse_schedule_args(time, cron, interval)?;
            (task, schedule, *max_runs)
        }
        ScheduleMessageType::Voice {
            chat_id,
            file_path,
            time,
            cron,
            interval,
            max_runs,
        } => {
            let task = TaskType::SendVoice {
                chat_id: chat_id.clone(),
                file_path: file_path.clone(),
            };
            let schedule = parse_schedule_args(time, cron, interval)?;
            (task, schedule, *max_runs)
        }
        ScheduleMessageType::Action {
            chat_id,
            action,
            time,
            cron,
            interval,
            max_runs,
        } => {
            let task = TaskType::SendAction {
                chat_id: chat_id.clone(),
                action: action.clone(),
            };
            let schedule = parse_schedule_args(time, cron, interval)?;
            (task, schedule, *max_runs)
        }
    };

    let task_id = scheduler.add_task(task_type, schedule, max_runs).await?;
    println!(
        "✅ Task scheduled successfully with ID: {}",
        task_id.green()
    );
    Ok(())
}

async fn execute_scheduler_action(_bot: &Bot, action: &SchedulerAction) -> CliResult<()> {
    let mut scheduler = Scheduler::new(None).await?;
    // Note: We need to create a new bot instance for the scheduler
    // since Bot doesn't implement Clone
    let token = std::env::var("VKTEAMS_BOT_API_TOKEN")
        .map_err(|_| CliError::InputError("Bot token not available".to_string()))?;
    let url = std::env::var("VKTEAMS_BOT_API_URL")
        .map_err(|_| CliError::InputError("Bot URL not available".to_string()))?;
    let scheduler_bot =
        Bot::with_params(&APIVersionUrl::V1, &token, &url).map_err(CliError::ApiError)?;
    scheduler.set_bot(scheduler_bot);

    match action {
        SchedulerAction::Start => {
            println!("🚀 Starting scheduler daemon...");
            scheduler.run_scheduler().await?;
        }
        SchedulerAction::Stop => {
            println!("⏹️ Stopping scheduler daemon...");
            stop_scheduler_daemon().await?;
            println!("✅ Scheduler daemon stopped successfully");
        }
        SchedulerAction::Status => {
            let daemon_status = scheduler.get_daemon_status().await;
            
            println!("📊 Scheduler Status:");
            
            // Display daemon running status
            match &daemon_status.status {
                crate::scheduler::DaemonStatus::NotRunning => {
                    println!("  Daemon: {} (Not running)", "⏹️ Stopped".red());
                }
                crate::scheduler::DaemonStatus::Running { pid, started_at } => {
                    println!("  Daemon: {} (PID: {}, Started: {})", 
                        "🟢 Running".green(), 
                        pid, 
                        started_at.format("%Y-%m-%d %H:%M:%S UTC"));
                }
                crate::scheduler::DaemonStatus::Stale { pid, started_at } => {
                    println!("  Daemon: {} (Stale PID: {}, Started: {})", 
                        "⚠️ Stale".yellow(), 
                        pid, 
                        started_at.format("%Y-%m-%d %H:%M:%S UTC"));
                }
                crate::scheduler::DaemonStatus::Unknown(msg) => {
                    println!("  Daemon: {} ({})", "❓ Unknown".yellow(), msg);
                }
            }
            
            println!("  PID file: {:?}", daemon_status.pid_file_path);
            println!("  Total tasks: {}", daemon_status.total_tasks);
            println!("  Enabled tasks: {}", daemon_status.enabled_tasks.to_string().green());
            println!(
                "  Disabled tasks: {}",
                (daemon_status.total_tasks - daemon_status.enabled_tasks).to_string().yellow()
            );
        }
        SchedulerAction::List => {
            let tasks = scheduler.list_tasks().await;

            if tasks.is_empty() {
                println!("📭 No scheduled tasks found");
                return Ok(());
            }

            println!("📋 Scheduled Tasks:");
            for task in tasks {
                let status = if task.enabled {
                    "✅ Active".green()
                } else {
                    "⏸️ Disabled".yellow()
                };
                println!(
                    "  {} [{}] {}",
                    task.id,
                    status,
                    task.task_type.description()
                );
                println!("    Schedule: {}", task.schedule.description());
                println!(
                    "    Runs: {}/{}",
                    task.run_count,
                    task.max_runs.map_or("∞".to_string(), |m| m.to_string())
                );
                println!(
                    "    Next run: {}",
                    task.next_run.format("%Y-%m-%d %H:%M:%S UTC")
                );
                println!();
            }
        }
    }
    Ok(())
}

async fn execute_task_action(_bot: &Bot, action: &TaskAction) -> CliResult<()> {
    let mut scheduler = Scheduler::new(None).await?;
    // Note: We need to create a new bot instance for the scheduler
    // since Bot doesn't implement Clone
    let token = std::env::var("VKTEAMS_BOT_API_TOKEN")
        .map_err(|_| CliError::InputError("Bot token not available".to_string()))?;
    let url = std::env::var("VKTEAMS_BOT_API_URL")
        .map_err(|_| CliError::InputError("Bot URL not available".to_string()))?;
    let scheduler_bot =
        Bot::with_params(&APIVersionUrl::V1, &token, &url).map_err(CliError::ApiError)?;
    scheduler.set_bot(scheduler_bot);

    match action {
        TaskAction::Show { task_id } => {
            if let Some(task) = scheduler.get_task(task_id).await {
                println!("📋 Task Details:");
                println!("  ID: {}", task.id);
                println!("  Type: {}", task.task_type.description());
                println!("  Schedule: {}", task.schedule.description());
                println!(
                    "  Status: {}",
                    if task.enabled {
                        "✅ Active".green()
                    } else {
                        "⏸️ Disabled".yellow()
                    }
                );
                println!(
                    "  Created: {}",
                    task.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                );
                println!(
                    "  Runs: {}/{}",
                    task.run_count,
                    task.max_runs.map_or("∞".to_string(), |m| m.to_string())
                );
                println!(
                    "  Next run: {}",
                    task.next_run.format("%Y-%m-%d %H:%M:%S UTC")
                );
                if let Some(last_run) = task.last_run {
                    println!("  Last run: {}", last_run.format("%Y-%m-%d %H:%M:%S UTC"));
                }
            } else {
                return Err(CliError::InputError(format!("Task not found: {}", task_id)));
            }
        }
        TaskAction::Remove { task_id } => {
            scheduler.remove_task(task_id).await?;
            println!("🗑️ Task {} removed successfully", task_id.green());
        }
        TaskAction::Enable { task_id } => {
            scheduler.enable_task(task_id).await?;
            println!("✅ Task {} enabled successfully", task_id.green());
        }
        TaskAction::Disable { task_id } => {
            scheduler.disable_task(task_id).await?;
            println!("⏸️ Task {} disabled successfully", task_id.yellow());
        }
        TaskAction::Run { task_id } => {
            println!("🚀 Running task {} once...", task_id);
            scheduler.run_task_once(task_id).await?;
            println!("✅ Task {} executed successfully", task_id.green());
        }
    }
    Ok(())
}

// Helper function to parse schedule arguments
fn parse_schedule_args(
    time: &Option<String>,
    cron: &Option<String>,
    interval: &Option<u64>,
) -> CliResult<ScheduleType> {
    let count = [time.is_some(), cron.is_some(), interval.is_some()]
        .iter()
        .filter(|&&x| x)
        .count();

    if count == 0 {
        return Err(CliError::InputError(
            "One of --time, --cron, or --interval must be specified".to_string(),
        ));
    }

    if count > 1 {
        return Err(CliError::InputError(
            "Only one of --time, --cron, or --interval can be specified".to_string(),
        ));
    }

    if let Some(time_str) = time {
        let datetime = parse_schedule_time(time_str)?;
        Ok(ScheduleType::Once(datetime))
    } else if let Some(cron_expr) = cron {
        // Validate cron expression
        cron::Schedule::from_str(cron_expr)
            .map_err(|e| CliError::InputError(format!("Invalid cron expression: {}", e)))?;
        Ok(ScheduleType::Cron(cron_expr.clone()))
    } else if let Some(interval_secs) = interval {
        if *interval_secs == 0 {
            return Err(CliError::InputError(
                "Interval must be greater than 0".to_string(),
            ));
        }
        Ok(ScheduleType::Interval {
            duration_seconds: *interval_secs,
            start_time: Utc::now(),
        })
    } else {
        unreachable!()
    }
}

// Validation functions
fn validate_schedule_command(message_type: &ScheduleMessageType) -> CliResult<()> {
    match message_type {
        ScheduleMessageType::Text { chat_id, message, time, cron, interval, max_runs } => {
            validate_chat_id(chat_id)?;
            validate_message_content(message)?;
            // Validate schedule arguments by trying to parse them
            parse_schedule_args(time, cron, interval)?;
            validate_max_runs(max_runs)?;
        }
        ScheduleMessageType::File { chat_id, file_path, time, cron, interval, max_runs } => {
            validate_chat_id(chat_id)?;
            validate_file_path_arg(file_path)?;
            parse_schedule_args(time, cron, interval)?;
            validate_max_runs(max_runs)?;
        }
        ScheduleMessageType::Voice { chat_id, file_path, time, cron, interval, max_runs } => {
            validate_chat_id(chat_id)?;
            validate_voice_file_path(file_path)?;
            parse_schedule_args(time, cron, interval)?;
            validate_max_runs(max_runs)?;
        }
        ScheduleMessageType::Action { chat_id, action, time, cron, interval, max_runs } => {
            validate_chat_id(chat_id)?;
            validate_action_type(action)?;
            parse_schedule_args(time, cron, interval)?;
            validate_max_runs(max_runs)?;
        }
    }
    Ok(())
}

fn validate_scheduler_action(action: &SchedulerAction) -> CliResult<()> {
    // Basic validation - all scheduler actions are valid
    match action {
        SchedulerAction::Start | SchedulerAction::Stop | SchedulerAction::Status | SchedulerAction::List => Ok(())
    }
}

fn validate_task_action(action: &TaskAction) -> CliResult<()> {
    match action {
        TaskAction::Show { task_id } | 
        TaskAction::Remove { task_id } | 
        TaskAction::Enable { task_id } | 
        TaskAction::Disable { task_id } | 
        TaskAction::Run { task_id } => {
            validate_task_id(task_id)
        }
    }
}

// Helper validation functions
fn validate_chat_id(chat_id: &str) -> CliResult<()> {
    if chat_id.trim().is_empty() {
        return Err(CliError::InputError("Chat ID cannot be empty".to_string()));
    }
    if chat_id.len() > 100 {
        return Err(CliError::InputError("Chat ID is too long (max 100 characters)".to_string()));
    }
    Ok(())
}

fn validate_message_content(message: &str) -> CliResult<()> {
    if message.trim().is_empty() {
        return Err(CliError::InputError("Message content cannot be empty".to_string()));
    }
    if message.len() > 10000 {
        return Err(CliError::InputError("Message is too long (max 10000 characters)".to_string()));
    }
    Ok(())
}

fn validate_file_path_arg(file_path: &str) -> CliResult<()> {
    if file_path.trim().is_empty() {
        return Err(CliError::InputError("File path cannot be empty".to_string()));
    }
    if !std::path::Path::new(file_path).exists() {
        return Err(CliError::InputError(format!("File does not exist: {}", file_path)));
    }
    Ok(())
}

fn validate_voice_file_path(file_path: &str) -> CliResult<()> {
    validate_file_path_arg(file_path)?;
    
    // Check file extension for voice messages
    let path = std::path::Path::new(file_path);
    if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        if !["ogg", "mp3", "wav", "m4a", "aac"].contains(&ext.as_str()) {
            return Err(CliError::InputError(format!(
                "Unsupported voice file format: {}. Supported: ogg, mp3, wav, m4a, aac", 
                ext
            )));
        }
    } else {
        return Err(CliError::InputError("Voice file must have a valid extension".to_string()));
    }
    
    Ok(())
}

fn validate_action_type(action: &str) -> CliResult<()> {
    if action.trim().is_empty() {
        return Err(CliError::InputError("Action type cannot be empty".to_string()));
    }
    
    // Check supported action types
    let valid_actions = ["typing", "upload_photo", "record_video", "upload_video", "record_audio", "upload_audio", "upload_document", "find_location"];
    if !valid_actions.contains(&action) {
        return Err(CliError::InputError(format!(
            "Unsupported action type: {}. Supported: {}", 
            action,
            valid_actions.join(", ")
        )));
    }
    
    Ok(())
}

fn validate_max_runs(max_runs: &Option<u64>) -> CliResult<()> {
    if let Some(runs) = max_runs {
        if *runs == 0 {
            return Err(CliError::InputError("Max runs must be greater than 0".to_string()));
        }
        if *runs > 10000 {
            return Err(CliError::InputError("Max runs cannot exceed 10000".to_string()));
        }
    }
    Ok(())
}

fn validate_task_id(task_id: &str) -> CliResult<()> {
    if task_id.trim().is_empty() {
        return Err(CliError::InputError("Task ID cannot be empty".to_string()));
    }
    if task_id.len() > 50 {
        return Err(CliError::InputError("Task ID is too long (max 50 characters)".to_string()));
    }
    Ok(())
}

// Daemon management functions
async fn stop_scheduler_daemon() -> CliResult<()> {
    use std::fs;
    
    // Create stop signal file in temporary directory
    let temp_dir = std::env::temp_dir();
    let stop_file = temp_dir.join("vkteams_scheduler_stop.signal");
    
    // Write stop signal
    fs::write(&stop_file, "stop").map_err(|e| {
        CliError::FileError(format!("Failed to create stop signal file: {}", e))
    })?;
    
    // Wait for daemon to acknowledge stop signal (max 30 seconds)
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 300; // 30 seconds with 100ms intervals
    
    while attempts < MAX_ATTEMPTS && stop_file.exists() {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        attempts += 1;
    }
    
    if stop_file.exists() {
        // Clean up the file if daemon didn't acknowledge
        let _ = fs::remove_file(&stop_file);
        return Err(CliError::UnexpectedError(
            "Daemon did not respond to stop signal within 30 seconds".to_string()
        ));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio::runtime::Runtime;

    fn dummy_bot() -> Bot {
        Bot::with_params(&APIVersionUrl::V1, "dummy_token", "https://dummy.api.com").unwrap()
    }

    /// Helper to set required environment variables for scheduler tests
    fn set_env_vars() {
        unsafe {
            env::set_var("VKTEAMS_BOT_API_TOKEN", "dummy_token");
            env::set_var("VKTEAMS_BOT_API_URL", "https://dummy.api.com");

            // Create a temporary directory for tests
            let temp_dir = std::env::temp_dir().join("vkteams_bot_test");
            std::fs::create_dir_all(&temp_dir).ok();
            env::set_var("HOME", temp_dir.to_string_lossy().to_string());
        }
    }

    #[test]
    fn test_execute_schedule_api_error() {
        let cmd = SchedulingCommands::Schedule {
            message_type: ScheduleMessageType::Text {
                chat_id: "12345@chat".to_string(),
                message: "hello".to_string(),
                time: None,
                cron: None,
                interval: None,
                max_runs: None,
            },
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        // Ожидаем ошибку из-за отсутствия переменных окружения
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }

    #[test]
    fn test_execute_task_action_api_error() {
        let cmd = SchedulingCommands::Task {
            action: TaskAction::Show {
                task_id: "id".to_string(),
            },
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        // Ожидаем ошибку из-за отсутствия переменных окружения
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_schedule_args_time_invalid() {
        let res = parse_schedule_args(&Some("not-a-time".to_string()), &None, &None);
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_schedule_args_cron_invalid() {
        let res = parse_schedule_args(&None, &Some("* * * *".to_string()), &None);
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_schedule_args_interval_zero() {
        let res = parse_schedule_args(&None, &None, &Some(0));
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_schedule_args_all_none() {
        let res = parse_schedule_args(&None, &None, &None);
        assert!(res.is_err());
    }

    #[test]
    fn test_execute_schedule_success() {
        set_env_vars();
        let cmd = SchedulingCommands::Schedule {
            message_type: ScheduleMessageType::Text {
                chat_id: "12345@chat".to_string(),
                message: "hello".to_string(),
                time: Some("2030-01-01T00:00:00Z".to_string()),
                cron: None,
                interval: None,
                max_runs: Some(1),
            },
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        // Should succeed if environment is set and time is valid
        if let Err(e) = &res {
            eprintln!("Schedule command failed: {:?}", e);
        }
        assert!(res.is_ok(), "Schedule command failed: {:?}", res.err());
    }

    #[test]
    fn test_parse_schedule_args_time_success() {
        let res = parse_schedule_args(&Some("2030-01-01T00:00:00Z".to_string()), &None, &None);
        assert!(matches!(res, Ok(ScheduleType::Once(_))));
    }

    #[test]
    fn test_parse_schedule_args_cron_success() {
        // Use a valid 6-field cron expression (with seconds)
        let res = parse_schedule_args(&None, &Some("0 * * * * *".to_string()), &None);
        assert!(matches!(res, Ok(ScheduleType::Cron(_))));
    }

    #[test]
    fn test_parse_schedule_args_interval_success() {
        let res = parse_schedule_args(&None, &None, &Some(60));
        assert!(matches!(res, Ok(ScheduleType::Interval { .. })));
    }

    #[test]
    fn test_execute_scheduler_action_status_success() {
        set_env_vars();
        let cmd = SchedulingCommands::Scheduler {
            action: SchedulerAction::Status,
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_ok());
    }

    #[test]
    fn test_execute_scheduler_action_list_success() {
        set_env_vars();
        let cmd = SchedulingCommands::Scheduler {
            action: SchedulerAction::List,
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_ok());
    }

    #[test]
    fn test_execute_task_action_show_not_found() {
        set_env_vars();
        let cmd = SchedulingCommands::Task {
            action: TaskAction::Show {
                task_id: "nonexistent".to_string(),
            },
        };
        let bot = dummy_bot();
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(cmd.execute(&bot));
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_execute_task_action_remove_enable_disable() {
        set_env_vars();
        let mut scheduler = Scheduler::new(None).await.unwrap();
        // Add a dummy task
        let task_id = scheduler
            .add_task(
                TaskType::SendText {
                    chat_id: "12345@chat".to_string(),
                    message: "test".to_string(),
                },
                ScheduleType::Once(Utc::now()),
                Some(1),
            )
            .await
            .unwrap();
        // Remove
        assert!(scheduler.remove_task(&task_id).await.is_ok());
        // Add again
        let task_id = scheduler
            .add_task(
                TaskType::SendText {
                    chat_id: "12345@chat".to_string(),
                    message: "test".to_string(),
                },
                ScheduleType::Once(Utc::now()),
                Some(1),
            )
            .await
            .unwrap();
        // Enable
        assert!(scheduler.enable_task(&task_id).await.is_ok());
        // Disable
        assert!(scheduler.disable_task(&task_id).await.is_ok());
    }

    #[test]
    fn test_validate_chat_id() {
        assert!(validate_chat_id("valid_chat").is_ok());
        assert!(validate_chat_id("").is_err());
        assert!(validate_chat_id("   ").is_err());
        assert!(validate_chat_id(&"x".repeat(101)).is_err());
    }

    #[test]
    fn test_validate_message_content() {
        assert!(validate_message_content("Hello world").is_ok());
        assert!(validate_message_content("").is_err());
        assert!(validate_message_content("   ").is_err());
        assert!(validate_message_content(&"x".repeat(10001)).is_err());
    }

    #[test]
    fn test_validate_action_type() {
        assert!(validate_action_type("typing").is_ok());
        assert!(validate_action_type("upload_photo").is_ok());
        assert!(validate_action_type("invalid_action").is_err());
        assert!(validate_action_type("").is_err());
        assert!(validate_action_type("   ").is_err());
    }

    #[test]
    fn test_validate_max_runs() {
        assert!(validate_max_runs(&None).is_ok());
        assert!(validate_max_runs(&Some(1)).is_ok());
        assert!(validate_max_runs(&Some(100)).is_ok());
        assert!(validate_max_runs(&Some(0)).is_err());
        assert!(validate_max_runs(&Some(10001)).is_err());
    }

    #[test]
    fn test_validate_task_id() {
        assert!(validate_task_id("valid_id").is_ok());
        assert!(validate_task_id("").is_err());
        assert!(validate_task_id("   ").is_err());
        assert!(validate_task_id(&"x".repeat(51)).is_err());
    }

    #[test]
    fn test_validate_scheduler_command() {
        let valid_cmd = SchedulingCommands::Schedule {
            message_type: ScheduleMessageType::Text {
                chat_id: "test_chat".to_string(),
                message: "test message".to_string(),
                time: Some("2030-01-01T00:00:00Z".to_string()),
                cron: None,
                interval: None,
                max_runs: Some(1),
            },
        };
        assert!(valid_cmd.validate().is_ok());

        let invalid_cmd = SchedulingCommands::Schedule {
            message_type: ScheduleMessageType::Text {
                chat_id: "".to_string(), // Empty chat ID
                message: "test message".to_string(),
                time: Some("2030-01-01T00:00:00Z".to_string()),
                cron: None,
                interval: None,
                max_runs: Some(1),
            },
        };
        assert!(invalid_cmd.validate().is_err());
    }

    #[tokio::test]
    async fn test_stop_scheduler_daemon_no_running_daemon() {
        use std::fs;
        
        // Clean up any existing stop signal file first
        let temp_dir = std::env::temp_dir();
        let stop_file = temp_dir.join("vkteams_scheduler_stop.signal");
        let _ = fs::remove_file(&stop_file);
        
        // Test stop command when no daemon is running
        // Should timeout and return error
        let result = stop_scheduler_daemon().await;
        assert!(result.is_err());
        if let Err(CliError::UnexpectedError(msg)) = result {
            assert!(msg.contains("30 seconds"));
        }
    }
}
