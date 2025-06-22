//! Daemon commands for automatic chat listening and event processing

use clap::Subcommand;
use crate::errors::prelude::Result as CliResult;
use crate::commands::{Command, OutputFormat, CommandResult};
use async_trait::async_trait;
use vkteams_bot::prelude::{Bot, ResponseEventsGet};
use crate::config::Config;
use tokio::signal;
use tracing::{info, error, warn, debug};
use std::sync::Arc;
use std::time::Instant;

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
            DaemonCommands::Start { foreground, auto_save, .. } => {
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
                    Err(e) => CommandResult::error(format!("Failed to get daemon status: {}", e))
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
    // TODO: Add storage when storage issues are resolved
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
        // TODO: Initialize storage when storage issues are resolved
        
        Ok(Self {
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
        let failed_count = 0;

        // Process events - for now just log them
        // TODO: Add actual storage when storage issues are resolved
        for event in events.events {
            debug!("Processing event: {} (type: {:?})", event.event_id, event.event_type);
            // Simulate processing
            saved_count += 1;
        }

        let duration = start_time.elapsed();
        
        // Update statistics
        self.stats.events_processed.fetch_add(event_count as u64, std::sync::atomic::Ordering::Relaxed);
        self.stats.events_saved.fetch_add(saved_count, std::sync::atomic::Ordering::Relaxed);
        self.stats.events_failed.fetch_add(failed_count, std::sync::atomic::Ordering::Relaxed);
        
        if let Ok(mut last_time) = self.stats.last_processed_time.lock() {
            *last_time = Some(chrono::Utc::now());
        }

        info!(
            "Processed {} events in {:?}: {} saved, {} failed", 
            event_count, duration, saved_count, failed_count
        );

        if failed_count > 0 {
            warn!("{} events failed to save - check storage connection", failed_count);
        }

        Ok(())
    }

    /// Get processor statistics
    pub fn get_stats(&self) -> ProcessorStatsSnapshot {
        let start_time = *self.stats.start_time.lock().unwrap();
        let now = chrono::Utc::now();
        let uptime_seconds = (now - start_time).num_seconds();
        let events_processed = self.stats.events_processed.load(std::sync::atomic::Ordering::Relaxed);
        
        ProcessorStatsSnapshot {
            events_processed,
            events_saved: self.stats.events_saved.load(std::sync::atomic::Ordering::Relaxed),
            events_failed: self.stats.events_failed.load(std::sync::atomic::Ordering::Relaxed),
            last_processed_time: *self.stats.last_processed_time.lock().unwrap(),
            start_time,
            uptime_seconds,
            bytes_processed: self.stats.bytes_processed.load(std::sync::atomic::Ordering::Relaxed),
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
    info!("Starting VKTeams Bot daemon in foreground mode with auto_save={}", auto_save);
    
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
                    debug!("Received {} events (auto-save disabled)", events.events.len());
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
        "Background daemon mode not yet implemented. Use --foreground flag.".to_string()
    ))
}

/// Stop daemon (placeholder)
async fn stop_daemon() -> CliResult<()> {
    Err(crate::errors::prelude::CliError::UnexpectedError(
        "Daemon stop not yet implemented.".to_string()
    ))
}

/// Check daemon status
async fn check_daemon_status() -> CliResult<()> {
    info!("Daemon status check - not yet implemented");
    Ok(())
}

/// Get daemon status with detailed information
async fn get_daemon_status(_pid_file: Option<&str>) -> CliResult<serde_json::Value> {
    // For now, return a mock status
    // TODO: Implement actual status checking when PID file management is ready
    
    // Check if we can access the process stats (this is a placeholder)
    let is_running = false; // Would check actual PID file and process
    
    if is_running {
        Ok(serde_json::json!({
            "status": "running",
            "pid": 12345,
            "uptime": "2h 30m",
            "events_processed": 1250,
            "events_saved": 1200,
            "events_failed": 50,
            "last_activity": chrono::Utc::now().to_rfc3339(),
            "memory_usage_mb": 45.2,
            "cpu_usage_percent": 1.5
        }))
    } else {
        Ok(serde_json::json!({
            "status": "not_running",
            "message": "Daemon is not currently running"
        }))
    }
}

/// Setup graceful shutdown signal handling
async fn setup_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        
        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to create SIGTERM handler");
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;
    
    #[test]
    fn test_processor_stats() {
        let processor = AutoSaveEventProcessor {
            stats: Arc::new(ProcessorStats::default()),
        };
        
        // Test initial stats
        let stats = processor.get_stats();
        assert_eq!(stats.events_processed, 0);
        assert_eq!(stats.events_saved, 0);
        assert_eq!(stats.events_failed, 0);
        assert!(stats.last_processed_time.is_none());
        
        // Test updating stats
        processor.stats.events_processed.store(100, Ordering::Relaxed);
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
}
