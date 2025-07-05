//! Daemon commands for automatic chat listening and event processing

use crate::commands::{Command, CommandResult, OutputFormat};
use crate::config::Config;
use crate::errors::{CliError, prelude::Result as CliResult};
use async_trait::async_trait;
use clap::Subcommand;
use std::sync::Arc;
use std::time::Instant;
use tokio::signal;
use tracing::{debug, error, info, warn};
use vkteams_bot::prelude::{Bot, ResponseEventsGet};
#[cfg(feature = "storage")]
use vkteams_bot::storage::StorageManager;

#[derive(Subcommand, Debug, Clone)]
pub enum DaemonCommands {
    /// Start automatic chat listener daemon
    #[command(name = "start")]
    Start {
        /// Run in foreground (don't daemonize)
        #[arg(short, long)]
        foreground: bool,

        /// PID file path
        #[arg(long, value_name = "PATH")]
        pid_file: Option<String>,

        /// Enable auto-storage of events
        #[arg(long)]
        auto_save: bool,

        /// Chat ID to listen (optional, uses config default)
        #[arg(long)]
        chat_id: Option<String>,
    },

    /// Stop daemon
    #[command(name = "stop")]
    Stop {
        /// PID file path
        #[arg(long, value_name = "PATH")]
        pid_file: Option<String>,
    },

    /// Check daemon status
    #[command(name = "status")]
    Status {
        /// PID file path
        #[arg(long, value_name = "PATH")]
        pid_file: Option<String>,
    },
}

#[async_trait]
impl Command for DaemonCommands {
    async fn execute(&self, bot: &Bot) -> CliResult<()> {
        match self {
            DaemonCommands::Start {
                foreground,
                auto_save,
                ..
            } => {
                if *foreground {
                    start_foreground_daemon(bot, *auto_save).await
                } else {
                    start_background_daemon(bot, *auto_save).await
                }
            }
            DaemonCommands::Stop { .. } => stop_daemon().await,
            DaemonCommands::Status { .. } => check_daemon_status().await,
        }
    }

    async fn execute_with_output(&self, bot: &Bot, format: &OutputFormat) -> CliResult<()> {
        let result = match self {
            DaemonCommands::Start { .. } => {
                self.execute(bot).await?;
                CommandResult::success_with_message("Daemon started successfully")
            }
            DaemonCommands::Stop { .. } => {
                self.execute(bot).await?;
                CommandResult::success_with_message("Daemon stopped successfully")
            }
            DaemonCommands::Status { pid_file } => {
                match get_daemon_status(pid_file.as_deref()).await {
                    Ok(status) => CommandResult::success_with_data(status),
                    Err(e) => CommandResult::error(format!("Failed to get daemon status: {e}")),
                }
            }
        };

        result.display(format)
    }

    fn name(&self) -> &'static str {
        "daemon"
    }
}

/// Auto-save event processor for automatic storage of events
pub struct AutoSaveEventProcessor {
    #[cfg(feature = "storage")]
    storage: Option<Arc<StorageManager>>,
    stats: Arc<ProcessorStats>,
}

pub struct ProcessorStats {
    events_processed: std::sync::atomic::AtomicU64,
    events_saved: std::sync::atomic::AtomicU64,
    events_failed: std::sync::atomic::AtomicU64,
    last_processed_time: std::sync::Mutex<Option<chrono::DateTime<chrono::Utc>>>,
    start_time: std::sync::Mutex<chrono::DateTime<chrono::Utc>>,
    bytes_processed: std::sync::atomic::AtomicU64,
}

impl Default for ProcessorStats {
    fn default() -> Self {
        Self {
            events_processed: std::sync::atomic::AtomicU64::new(0),
            events_saved: std::sync::atomic::AtomicU64::new(0),
            events_failed: std::sync::atomic::AtomicU64::new(0),
            last_processed_time: std::sync::Mutex::new(None),
            start_time: std::sync::Mutex::new(chrono::Utc::now()),
            bytes_processed: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProcessorStatsSnapshot {
    pub events_processed: u64,
    pub events_saved: u64,
    pub events_failed: u64,
    pub last_processed_time: Option<chrono::DateTime<chrono::Utc>>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub uptime_seconds: i64,
    pub bytes_processed: u64,
    pub events_per_second: f64,
}

impl AutoSaveEventProcessor {
    pub async fn new(_config: &Config) -> CliResult<Self> {
        #[cfg(feature = "storage")]
        let storage = {
            // Use default storage configuration for now
            // In future, this should read from environment variables or config file
            let storage_config = vkteams_bot::storage::StorageConfig::default();

            match StorageManager::new(&storage_config).await {
                Ok(manager) => {
                    // Initialize storage (run migrations)
                    if let Err(e) = manager.initialize().await {
                        warn!("Failed to initialize storage: {}", e);
                        None
                    } else {
                        info!("Storage manager initialized successfully");
                        Some(Arc::new(manager))
                    }
                }
                Err(e) => {
                    warn!("Failed to create storage manager: {}", e);
                    None
                }
            }
        };

        Ok(Self {
            #[cfg(feature = "storage")]
            storage,
            stats: Arc::new(ProcessorStats::default()),
        })
    }

    /// Process events batch and auto-save to storage
    pub async fn process_events(&self, _bot: Bot, events: ResponseEventsGet) -> CliResult<()> {
        let event_count = events.events.len();
        if event_count == 0 {
            return Ok(());
        }

        debug!("Auto-saving {} events to storage", event_count);

        let start_time = Instant::now();
        let mut saved_count = 0;
        let mut failed_count = 0;
        let mut total_bytes = 0;

        #[cfg(feature = "storage")]
        {
            if let Some(storage) = &self.storage {
                // Process events using real storage
                for event in events.events {
                    debug!(
                        "Processing event: {} (type: {:?})",
                        event.event_id, event.event_type
                    );

                    // Calculate event size for statistics
                    if let Ok(serialized) = serde_json::to_vec(&event) {
                        total_bytes += serialized.len();
                    }

                    // Try to store the event
                    match storage.process_event(&event).await {
                        Ok(event_id) => {
                            debug!(
                                "Successfully stored event {} with ID {}",
                                event.event_id, event_id
                            );
                            saved_count += 1;
                        }
                        Err(e) => {
                            error!("Failed to store event {}: {}", event.event_id, e);
                            failed_count += 1;
                        }
                    }
                }
            } else {
                // No storage available - just count events
                for event in events.events {
                    debug!(
                        "Processing event: {} (type: {:?}) - no storage available",
                        event.event_id, event.event_type
                    );
                    saved_count += 1;
                }
            }
        }

        #[cfg(not(feature = "storage"))]
        {
            // Storage feature not enabled - just count events
            for event in events.events {
                debug!(
                    "Processing event: {} (type: {:?}) - storage not enabled",
                    event.event_id, event.event_type
                );
                saved_count += 1;
            }
        }

        let duration = start_time.elapsed();

        // Update statistics
        self.stats
            .events_processed
            .fetch_add(event_count as u64, std::sync::atomic::Ordering::Relaxed);
        self.stats
            .events_saved
            .fetch_add(saved_count, std::sync::atomic::Ordering::Relaxed);
        self.stats
            .events_failed
            .fetch_add(failed_count, std::sync::atomic::Ordering::Relaxed);
        self.stats
            .bytes_processed
            .fetch_add(total_bytes as u64, std::sync::atomic::Ordering::Relaxed);

        if let Ok(mut last_time) = self.stats.last_processed_time.lock() {
            *last_time = Some(chrono::Utc::now());
        }

        info!(
            "Processed {} events in {:?}: {} saved, {} failed, {} bytes processed",
            event_count, duration, saved_count, failed_count, total_bytes
        );

        if failed_count > 0 {
            warn!(
                "{} events failed to save - check storage connection",
                failed_count
            );
        }

        Ok(())
    }

    /// Get processor statistics
    pub fn get_stats(&self) -> ProcessorStatsSnapshot {
        let start_time = *self.stats.start_time.lock().unwrap();
        let now = chrono::Utc::now();
        let uptime_seconds = (now - start_time).num_seconds();
        let events_processed = self
            .stats
            .events_processed
            .load(std::sync::atomic::Ordering::Relaxed);

        ProcessorStatsSnapshot {
            events_processed,
            events_saved: self
                .stats
                .events_saved
                .load(std::sync::atomic::Ordering::Relaxed),
            events_failed: self
                .stats
                .events_failed
                .load(std::sync::atomic::Ordering::Relaxed),
            last_processed_time: *self.stats.last_processed_time.lock().unwrap(),
            start_time,
            uptime_seconds,
            bytes_processed: self
                .stats
                .bytes_processed
                .load(std::sync::atomic::Ordering::Relaxed),
            events_per_second: if uptime_seconds > 0 {
                events_processed as f64 / uptime_seconds as f64
            } else {
                0.0
            },
        }
    }
}

/// Start daemon in foreground mode
async fn start_foreground_daemon(bot: &Bot, auto_save: bool) -> CliResult<()> {
    info!(
        "Starting VKTeams Bot daemon in foreground mode with auto_save={}",
        auto_save
    );

    let processor = if auto_save {
        // Load config for storage initialization
        let config = crate::config::UnifiedConfigAdapter::load()
            .map_err(|e| crate::errors::prelude::CliError::Config(e.to_string()))?;
        Some(Arc::new(AutoSaveEventProcessor::new(&config).await?))
    } else {
        None
    };

    // Setup graceful shutdown
    let shutdown_signal = setup_shutdown_signal();

    // Create the event processing function
    let event_processor = {
        let processor_clone = processor.clone();
        move |bot: Bot, events: ResponseEventsGet| {
            let processor_inner = processor_clone.clone();
            async move {
                let result = if let Some(processor) = processor_inner {
                    processor.process_events(bot, events).await
                } else {
                    debug!(
                        "Received {} events (auto-save disabled)",
                        events.events.len()
                    );
                    Ok(())
                };

                // Convert CliError to BotError for compatibility with event_listener
                result.map_err(|e| vkteams_bot::error::BotError::System(e.to_string()))
            }
        }
    };

    tokio::select! {
        result = bot.event_listener(event_processor) => {
            match result {
                Ok(_) => info!("Event listener finished successfully"),
                Err(e) => error!("Event listener error: {}", e),
            }
        }
        _ = shutdown_signal => {
            info!("Received shutdown signal, stopping daemon...");
        }
    }

    Ok(())
}

/// Start daemon in background mode (placeholder)
async fn start_background_daemon(_bot: &Bot, _auto_save: bool) -> CliResult<()> {
    // For now, just return an error - background mode would require more complex implementation
    Err(crate::errors::prelude::CliError::UnexpectedError(
        "Background daemon mode not yet implemented. Use --foreground flag.".to_string(),
    ))
}

/// Stop daemon (placeholder)
async fn stop_daemon() -> CliResult<()> {
    Err(crate::errors::prelude::CliError::UnexpectedError(
        "Daemon stop not yet implemented.".to_string(),
    ))
}

/// Check daemon status
async fn check_daemon_status() -> CliResult<()> {
    info!("Daemon status check - not yet implemented");
    Ok(())
}

/// Get daemon status with detailed information
async fn get_daemon_status(pid_file: Option<&str>) -> CliResult<serde_json::Value> {
    use chrono::{DateTime, Utc};
    use std::path::PathBuf;

    // Determine PID file path
    let pid_file_path = if let Some(path) = pid_file {
        PathBuf::from(path)
    } else {
        // Use default location
        let mut data_dir = dirs::data_dir()
            .ok_or_else(|| CliError::Config("Cannot determine data directory".to_string()))?;
        data_dir.push("vkteams-bot");
        data_dir.push("daemon.pid");
        data_dir
    };

    // Check if PID file exists
    if !pid_file_path.exists() {
        return Ok(serde_json::json!({
            "status": "not_running",
            "reason": "No PID file found",
            "pid_file": pid_file_path
        }));
    }

    // Read PID file content
    let pid_content = tokio::fs::read_to_string(&pid_file_path)
        .await
        .map_err(|e| CliError::FileError(format!("Failed to read PID file: {e}")))?;

    let lines: Vec<&str> = pid_content.trim().split('\n').collect();
    if lines.len() < 2 {
        return Ok(serde_json::json!({
            "status": "error",
            "reason": "Invalid PID file format",
            "pid_file": pid_file_path
        }));
    }

    let pid: u32 = lines[0]
        .parse()
        .map_err(|_| CliError::InputError("Invalid PID in file".to_string()))?;

    let started_at = DateTime::parse_from_rfc3339(lines[1])
        .map_err(|_| CliError::InputError("Invalid timestamp in PID file".to_string()))?
        .with_timezone(&Utc);

    // Check if process is actually running
    let is_running = is_process_running(pid);

    if is_running {
        // Calculate uptime
        let uptime = Utc::now().signed_duration_since(started_at);
        let uptime_str = format_duration(uptime);

        // Try to get memory usage (platform specific)
        let memory_usage = get_process_memory_usage(pid).unwrap_or_else(|| "unknown".to_string());

        Ok(serde_json::json!({
            "status": "running",
            "pid": pid,
            "started_at": started_at.to_rfc3339(),
            "uptime": uptime_str,
            "memory_usage": memory_usage,
            "pid_file": pid_file_path
        }))
    } else {
        Ok(serde_json::json!({
            "status": "stale",
            "reason": "PID file exists but process is not running",
            "pid": pid,
            "started_at": started_at.to_rfc3339(),
            "pid_file": pid_file_path
        }))
    }
}

/// Setup graceful shutdown signal handling
async fn setup_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut sigterm =
            signal(SignalKind::terminate()).expect("Failed to create SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to create SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => debug!("Received SIGTERM"),
            _ = sigint.recv() => debug!("Received SIGINT"),
            _ = signal::ctrl_c() => debug!("Received Ctrl+C"),
        }
    }

    #[cfg(windows)]
    {
        let _ = signal::ctrl_c().await;
        debug!("Received Ctrl+C");
    }
}

/// Check if a process with given PID is running
fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use std::process::Command;
        // On Unix systems, use kill with signal 0 to check if process exists
        match Command::new("kill").args(["-0", &pid.to_string()]).output() {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        // On Windows, use tasklist to check if process exists
        match Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV"])
            .output()
        {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                output_str.lines().count() > 1 // More than just header
            }
            Err(_) => false,
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        false // For other platforms, assume not running
    }
}

/// Format duration for human-readable display
fn format_duration(duration: chrono::Duration) -> String {
    let seconds = duration.num_seconds();
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    if days > 0 {
        format!("{}d {}h {}m", days, hours % 24, minutes % 60)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes % 60)
    } else if minutes > 0 {
        format!("{minutes}m")
    } else {
        format!("{seconds}s")
    }
}

/// Get memory usage of a process (platform specific)
fn get_process_memory_usage(pid: u32) -> Option<String> {
    #[cfg(unix)]
    {
        use std::process::Command;
        match Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "rss="])
            .output()
        {
            Ok(output) => {
                let rss_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(rss_kb) = rss_str.trim().parse::<u64>() {
                    let rss_mb = rss_kb / 1024;
                    Some(format!("{rss_mb}MB"))
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        match Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
            .output()
        {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                // Parse CSV output to extract memory usage
                if let Some(line) = output_str.lines().next() {
                    let fields: Vec<&str> = line.split(',').collect();
                    if fields.len() > 4 {
                        // Memory usage is typically in field 4 or 5
                        if let Some(mem_field) = fields.get(4) {
                            return Some(mem_field.trim_matches('"').to_string());
                        }
                    }
                }
                None
            }
            Err(_) => None,
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use std::sync::atomic::Ordering;
    use tokio;
    use vkteams_bot::prelude::{EventMessage, EventType};

    #[test]
    fn test_processor_stats() {
        let processor = AutoSaveEventProcessor {
            #[cfg(feature = "storage")]
            storage: None,
            stats: Arc::new(ProcessorStats::default()),
        };

        // Test initial stats
        let stats = processor.get_stats();
        assert_eq!(stats.events_processed, 0);
        assert_eq!(stats.events_saved, 0);
        assert_eq!(stats.events_failed, 0);
        assert!(stats.last_processed_time.is_none());

        // Test updating stats
        processor
            .stats
            .events_processed
            .store(100, Ordering::Relaxed);
        processor.stats.events_saved.store(95, Ordering::Relaxed);
        processor.stats.events_failed.store(5, Ordering::Relaxed);

        let updated_stats = processor.get_stats();
        assert_eq!(updated_stats.events_processed, 100);
        assert_eq!(updated_stats.events_saved, 95);
        assert_eq!(updated_stats.events_failed, 5);
        assert!(updated_stats.uptime_seconds >= 0);
    }

    #[test]
    fn test_daemon_command_name() {
        let cmd = DaemonCommands::Status { pid_file: None };
        assert_eq!(cmd.name(), "daemon");
    }

    #[test]
    fn test_daemon_commands_variants() {
        // Test Start command
        let start_cmd = DaemonCommands::Start {
            foreground: true,
            pid_file: Some("/tmp/test.pid".to_string()),
            auto_save: true,
            chat_id: Some("test-chat".to_string()),
        };
        assert_eq!(start_cmd.name(), "daemon");

        // Test Stop command
        let stop_cmd = DaemonCommands::Stop {
            pid_file: Some("/tmp/test.pid".to_string()),
        };
        assert_eq!(stop_cmd.name(), "daemon");

        // Test Status command
        let status_cmd = DaemonCommands::Status { pid_file: None };
        assert_eq!(status_cmd.name(), "daemon");
    }

    #[test]
    fn test_processor_stats_default() {
        let stats = ProcessorStats::default();
        assert_eq!(stats.events_processed.load(Ordering::Relaxed), 0);
        assert_eq!(stats.events_saved.load(Ordering::Relaxed), 0);
        assert_eq!(stats.events_failed.load(Ordering::Relaxed), 0);
        assert_eq!(stats.bytes_processed.load(Ordering::Relaxed), 0);
        assert!(stats.last_processed_time.lock().unwrap().is_none());
        assert!(
            Utc::now()
                .signed_duration_since(*stats.start_time.lock().unwrap())
                .num_seconds()
                >= 0
        );
    }

    #[test]
    fn test_processor_stats_snapshot_creation() {
        let processor = AutoSaveEventProcessor {
            #[cfg(feature = "storage")]
            storage: None,
            stats: Arc::new(ProcessorStats::default()),
        };

        // Set some values
        processor
            .stats
            .events_processed
            .store(50, Ordering::Relaxed);
        processor.stats.events_saved.store(45, Ordering::Relaxed);
        processor.stats.events_failed.store(5, Ordering::Relaxed);
        processor
            .stats
            .bytes_processed
            .store(1024, Ordering::Relaxed);

        // Update last processed time
        if let Ok(mut last_time) = processor.stats.last_processed_time.lock() {
            *last_time = Some(Utc::now());
        }

        let snapshot = processor.get_stats();
        assert_eq!(snapshot.events_processed, 50);
        assert_eq!(snapshot.events_saved, 45);
        assert_eq!(snapshot.events_failed, 5);
        assert_eq!(snapshot.bytes_processed, 1024);
        assert!(snapshot.last_processed_time.is_some());
        assert!(snapshot.uptime_seconds >= 0);
        assert!(snapshot.events_per_second >= 0.0);
    }

    #[tokio::test]
    async fn test_process_events_empty() {
        let processor = AutoSaveEventProcessor {
            #[cfg(feature = "storage")]
            storage: None,
            stats: Arc::new(ProcessorStats::default()),
        };

        let events = ResponseEventsGet { events: vec![] };

        let bot = vkteams_bot::Bot::with_params(
            &vkteams_bot::prelude::APIVersionUrl::V1,
            "test_token",
            "https://test.api.url",
        )
        .unwrap();
        let result = processor.process_events(bot, events).await;
        assert!(result.is_ok());

        let stats = processor.get_stats();
        assert_eq!(stats.events_processed, 0);
    }

    #[tokio::test]
    async fn test_process_events_with_events() {
        let processor = AutoSaveEventProcessor {
            #[cfg(feature = "storage")]
            storage: None,
            stats: Arc::new(ProcessorStats::default()),
        };

        use vkteams_bot::prelude::{
            Chat, ChatId, EventPayloadEditedMessage, EventPayloadNewMessage, From, MsgId,
            Timestamp, UserId,
        };

        let test_chat = Chat {
            chat_id: ChatId::from("test_chat"),
            chat_type: "private".to_string(),
            title: Some("Test Chat".to_string()),
        };

        let test_from = From {
            user_id: UserId("test_user".to_string()),
            first_name: "Test".to_string(),
            last_name: Some("User".to_string()),
        };

        let events = ResponseEventsGet {
            events: vec![
                EventMessage {
                    event_id: 1,
                    event_type: EventType::NewMessage(Box::new(EventPayloadNewMessage {
                        msg_id: MsgId("test_msg".to_string()),
                        text: "test message".to_string(),
                        chat: test_chat.clone(),
                        from: test_from.clone(),
                        format: None,
                        parts: vec![],
                        timestamp: Timestamp(1234567890),
                    })),
                },
                EventMessage {
                    event_id: 2,
                    event_type: EventType::EditedMessage(Box::new(EventPayloadEditedMessage {
                        msg_id: MsgId("test_msg_2".to_string()),
                        text: "edited message".to_string(),
                        timestamp: Timestamp(1234567890),
                        chat: test_chat,
                        from: test_from,
                        format: None,
                        edited_timestamp: Timestamp(1234567900),
                    })),
                },
            ],
        };

        let bot = vkteams_bot::Bot::with_params(
            &vkteams_bot::prelude::APIVersionUrl::V1,
            "test_token",
            "https://test.api.url",
        )
        .unwrap();
        let result = processor.process_events(bot, events).await;
        assert!(result.is_ok());

        let stats = processor.get_stats();
        assert_eq!(stats.events_processed, 2);
    }

    #[tokio::test]
    async fn test_start_background_daemon_error() {
        let bot = vkteams_bot::Bot::with_params(
            &vkteams_bot::prelude::APIVersionUrl::V1,
            "test_token",
            "https://test.api.url",
        )
        .unwrap();
        let result = start_background_daemon(&bot, false).await;
        assert!(result.is_err());
        match result {
            Err(CliError::UnexpectedError(msg)) => {
                assert!(msg.contains("Background daemon mode not yet implemented"));
            }
            _ => panic!("Expected UnexpectedError"),
        }
    }

    #[tokio::test]
    async fn test_stop_daemon_error() {
        let result = stop_daemon().await;
        assert!(result.is_err());
        match result {
            Err(CliError::UnexpectedError(msg)) => {
                assert!(msg.contains("Daemon stop not yet implemented"));
            }
            _ => panic!("Expected UnexpectedError"),
        }
    }

    #[tokio::test]
    async fn test_check_daemon_status_success() {
        let result = check_daemon_status().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_duration() {
        // Test seconds
        let duration = Duration::seconds(30);
        assert_eq!(format_duration(duration), "30s");

        // Test minutes
        let duration = Duration::minutes(5);
        assert_eq!(format_duration(duration), "5m");

        // Test hours
        let duration = Duration::hours(2);
        assert_eq!(format_duration(duration), "2h 0m");

        // Test days
        let duration = Duration::days(1) + Duration::hours(3) + Duration::minutes(15);
        assert_eq!(format_duration(duration), "1d 3h 15m");

        // Test complex duration
        let duration = Duration::days(10) + Duration::hours(5) + Duration::minutes(30);
        assert_eq!(format_duration(duration), "10d 5h 30m");
    }

    #[test]
    fn test_is_process_running() {
        // Test with current process PID (should always exist)
        let current_pid = std::process::id();
        assert!(is_process_running(current_pid));

        // Test with non-existent PID (very high number unlikely to exist)
        assert!(!is_process_running(999999));
    }

    #[test]
    fn test_get_process_memory_usage() {
        // Test with PID 1 (should exist on Unix systems)
        #[cfg(unix)]
        {
            let memory = get_process_memory_usage(1);
            // Should return Some value or None, but not panic
            assert!(memory.is_some() || memory.is_none());
        }

        // Test with non-existent PID
        let memory = get_process_memory_usage(999999);
        assert!(memory.is_none());
    }

    #[tokio::test]
    async fn test_get_daemon_status_no_pid_file() {
        let result = get_daemon_status(Some("/nonexistent/path/daemon.pid")).await;
        assert!(result.is_ok());

        let status = result.unwrap();
        assert_eq!(status["status"], "not_running");
        assert_eq!(status["reason"], "No PID file found");
    }

    #[tokio::test]
    async fn test_autosave_processor_new() {
        let config = Config::default();
        let result = AutoSaveEventProcessor::new(&config).await;
        assert!(result.is_ok());

        let processor = result.unwrap();
        let stats = processor.get_stats();
        assert_eq!(stats.events_processed, 0);
        assert_eq!(stats.events_saved, 0);
        assert_eq!(stats.events_failed, 0);
    }

    #[test]
    fn test_processor_stats_with_events_per_second() {
        let processor = AutoSaveEventProcessor {
            #[cfg(feature = "storage")]
            storage: None,
            stats: Arc::new(ProcessorStats::default()),
        };

        // Simulate some processing time
        std::thread::sleep(std::time::Duration::from_millis(100));

        processor
            .stats
            .events_processed
            .store(10, Ordering::Relaxed);

        let stats = processor.get_stats();
        assert_eq!(stats.events_processed, 10);
        assert!(stats.uptime_seconds >= 0);

        // Events per second should be calculated properly
        if stats.uptime_seconds > 0 {
            assert!(stats.events_per_second >= 0.0);
        } else {
            assert_eq!(stats.events_per_second, 0.0);
        }
    }
}
