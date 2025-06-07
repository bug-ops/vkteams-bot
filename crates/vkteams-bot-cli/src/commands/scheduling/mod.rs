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
    let mut scheduler = Scheduler::new()?;
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

    let task_id = scheduler.add_task(task_type, schedule, max_runs)?;
    println!(
        "✅ Task scheduled successfully with ID: {}",
        task_id.green()
    );
    Ok(())
}

async fn execute_scheduler_action(_bot: &Bot, action: &SchedulerAction) -> CliResult<()> {
    let mut scheduler = Scheduler::new()?;
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
            let tasks = scheduler.list_tasks();
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
            let tasks = scheduler.list_tasks();

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
    let mut scheduler = Scheduler::new()?;
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
            if let Some(task) = scheduler.get_task(task_id) {
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
            scheduler.remove_task(task_id)?;
            println!("🗑️ Task {} removed successfully", task_id.green());
        }
        TaskAction::Enable { task_id } => {
            scheduler.enable_task(task_id)?;
            println!("✅ Task {} enabled successfully", task_id.green());
        }
        TaskAction::Disable { task_id } => {
            scheduler.disable_task(task_id)?;
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
