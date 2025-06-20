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
        // TODO: Implement validation
        Ok(())
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
            println!("⏹️ Scheduler stop command received (not implemented for daemon mode)");
            // TODO: Implement proper daemon stop mechanism
        }
        SchedulerAction::Status => {
            let tasks = scheduler.list_tasks().await;
            let enabled_count = tasks.iter().filter(|t| t.enabled).count();
            let total_count = tasks.len();

            println!("📊 Scheduler Status:");
            println!("  Total tasks: {}", total_count);
            println!("  Enabled tasks: {}", enabled_count.to_string().green());
            println!(
                "  Disabled tasks: {}",
                (total_count - enabled_count).to_string().yellow()
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
}
