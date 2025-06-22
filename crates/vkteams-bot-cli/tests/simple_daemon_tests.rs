//! Simple integration tests for daemon functionality
//!
//! Tests daemon command parsing and basic functionality without complex mocking

use vkteams_bot_cli::commands::daemon::{
    AutoSaveEventProcessor, DaemonCommands
};
use vkteams_bot_cli::commands::Command;
use vkteams_bot_cli::config::Config;
use clap::Parser;

#[derive(Parser)]
#[command(name = "test")]
struct TestCli {
    #[command(subcommand)]
    daemon: DaemonCommands,
}

#[test]
fn test_daemon_command_parsing() {
    // Test start command parsing
    let args = vec!["test", "start", "--foreground", "--auto-save"];
    let cli = TestCli::try_parse_from(args).unwrap();
    
    match cli.daemon {
        DaemonCommands::Start { foreground, auto_save, .. } => {
            assert!(foreground);
            assert!(auto_save);
        }
        _ => panic!("Expected Start command"),
    }
    
    // Test status command parsing
    let args = vec!["test", "status"];
    let cli = TestCli::try_parse_from(args).unwrap();
    
    match cli.daemon {
        DaemonCommands::Status { .. } => {},
        _ => panic!("Expected Status command"),
    }
    
    // Test stop command parsing
    let args = vec!["test", "stop"];
    let cli = TestCli::try_parse_from(args).unwrap();
    
    match cli.daemon {
        DaemonCommands::Stop { .. } => {},
        _ => panic!("Expected Stop command"),
    }
}

#[test]
fn test_daemon_command_name() {
    let cmd = DaemonCommands::Start {
        foreground: true,
        pid_file: None,
        auto_save: false,
        chat_id: None,
    };
    assert_eq!(cmd.name(), "daemon");
    
    let cmd = DaemonCommands::Status { pid_file: None };
    assert_eq!(cmd.name(), "daemon");
    
    let cmd = DaemonCommands::Stop { pid_file: None };
    assert_eq!(cmd.name(), "daemon");
}

#[tokio::test]
async fn test_processor_creation() {
    let config = Config::default();
    let processor_result = AutoSaveEventProcessor::new(&config).await;
    
    // Should be able to create processor
    assert!(processor_result.is_ok());
    
    let processor = processor_result.unwrap();
    let stats = processor.get_stats();
    
    // Initial stats should be zero
    assert_eq!(stats.events_processed, 0);
    assert_eq!(stats.events_saved, 0);
    assert_eq!(stats.events_failed, 0);
    assert_eq!(stats.bytes_processed, 0);
    assert!(stats.uptime_seconds >= 0);
    assert_eq!(stats.events_per_second, 0.0);
    assert!(stats.last_processed_time.is_none());
}

#[test]
fn test_daemon_command_with_pid_file() {
    let args = vec!["test", "start", "--pid-file", "/tmp/daemon.pid"];
    let cli = TestCli::try_parse_from(args).unwrap();
    
    match cli.daemon {
        DaemonCommands::Start { pid_file, .. } => {
            assert_eq!(pid_file, Some("/tmp/daemon.pid".to_string()));
        }
        _ => panic!("Expected Start command"),
    }
}

#[test]
fn test_daemon_command_with_chat_id() {
    let args = vec!["test", "start", "--chat-id", "test_chat_123"];
    let cli = TestCli::try_parse_from(args).unwrap();
    
    match cli.daemon {
        DaemonCommands::Start { chat_id, .. } => {
            assert_eq!(chat_id, Some("test_chat_123".to_string()));
        }
        _ => panic!("Expected Start command"),
    }
}

#[test]
fn test_processor_stats_serialization() {
    let config = Config::default();
    
    // Create processor in async context
    let rt = tokio::runtime::Runtime::new().unwrap();
    let processor = rt.block_on(async {
        AutoSaveEventProcessor::new(&config).await.unwrap()
    });
    
    let snapshot = processor.get_stats();
    
    // Test that snapshot can be serialized to JSON
    let json = serde_json::to_string(&snapshot).unwrap();
    assert!(json.contains("events_processed"));
    assert!(json.contains("events_saved"));
    assert!(json.contains("events_failed"));
    assert!(json.contains("uptime_seconds"));
    assert!(json.contains("events_per_second"));
    
    // Test pretty printing
    let pretty_json = serde_json::to_string_pretty(&snapshot).unwrap();
    assert!(pretty_json.contains("  \"events_processed\""));
}

#[test]
fn test_daemon_command_parsing_errors() {
    // Test invalid arguments
    let args = vec!["test", "start", "--invalid-flag"];
    let result = TestCli::try_parse_from(args);
    assert!(result.is_err());
    
    // Test missing subcommand
    let args = vec!["test"];
    let result = TestCli::try_parse_from(args);
    assert!(result.is_err());
}

#[test]
fn test_daemon_start_command_flags() {
    // Test all flags
    let args = vec![
        "test", "start", 
        "--foreground", 
        "--auto-save", 
        "--pid-file", "/tmp/test.pid",
        "--chat-id", "chat123"
    ];
    let cli = TestCli::try_parse_from(args).unwrap();
    
    match cli.daemon {
        DaemonCommands::Start { foreground, auto_save, pid_file, chat_id } => {
            assert!(foreground);
            assert!(auto_save);
            assert_eq!(pid_file, Some("/tmp/test.pid".to_string()));
            assert_eq!(chat_id, Some("chat123".to_string()));
        }
        _ => panic!("Expected Start command"),
    }
}

#[test]
fn test_daemon_stop_command_flags() {
    let args = vec!["test", "stop", "--pid-file", "/tmp/test.pid"];
    let cli = TestCli::try_parse_from(args).unwrap();
    
    match cli.daemon {
        DaemonCommands::Stop { pid_file } => {
            assert_eq!(pid_file, Some("/tmp/test.pid".to_string()));
        }
        _ => panic!("Expected Stop command"),
    }
}

#[test]
fn test_daemon_status_command_flags() {
    let args = vec!["test", "status", "--pid-file", "/tmp/test.pid"];
    let cli = TestCli::try_parse_from(args).unwrap();
    
    match cli.daemon {
        DaemonCommands::Status { pid_file } => {
            assert_eq!(pid_file, Some("/tmp/test.pid".to_string()));
        }
        _ => panic!("Expected Status command"),
    }
}

#[tokio::test]
async fn test_processor_stats_uptime_calculation() {
    let config = Config::default();
    let processor = AutoSaveEventProcessor::new(&config).await.unwrap();
    
    // Get stats immediately
    let stats1 = processor.get_stats();
    assert!(stats1.uptime_seconds >= 0);
    
    // Wait a bit and get stats again
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let stats2 = processor.get_stats();
    
    // Second measurement should have higher uptime
    assert!(stats2.uptime_seconds >= stats1.uptime_seconds);
}

#[test]
fn test_processor_stats_structure() {
    let config = Config::default();
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let processor = rt.block_on(async {
        AutoSaveEventProcessor::new(&config).await.unwrap()
    });
    
    let stats = processor.get_stats();
    
    // Verify all fields are present and have expected types
    assert!(stats.events_processed >= 0);
    assert!(stats.events_saved >= 0);
    assert!(stats.events_failed >= 0);
    assert!(stats.bytes_processed >= 0);
    assert!(stats.uptime_seconds >= 0);
    assert!(stats.events_per_second >= 0.0);
    
    // start_time should be a valid datetime
    assert!(stats.start_time <= chrono::Utc::now());
    
    // last_processed_time should initially be None
    assert!(stats.last_processed_time.is_none());
}

#[cfg(feature = "storage")]
#[tokio::test]
async fn test_processor_with_storage_integration() {
    let config = Config::default();
    let processor = AutoSaveEventProcessor::new(&config).await.unwrap();
    
    // Test that processor can be created with storage features enabled
    // Storage might not actually connect (no real database), but the structure should be valid
    let stats = processor.get_stats();
    assert_eq!(stats.events_processed, 0);
    assert_eq!(stats.events_saved, 0);
    assert_eq!(stats.events_failed, 0);
}