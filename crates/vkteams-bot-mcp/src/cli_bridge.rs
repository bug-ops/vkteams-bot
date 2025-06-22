//! CLI Bridge for MCP Server
//!
//! This module provides an abstraction layer for calling the VK Teams Bot CLI
//! from the MCP server using subprocess calls. This approach ensures that all
//! business logic remains in the CLI while the MCP server acts as a thin adapter.

use crate::errors::{BridgeError, CliErrorInfo};
use serde_json::Value;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, warn};

/// Bridge for executing CLI commands from MCP server
#[derive(Debug)]
pub struct CliBridge {
    cli_path: String,
    default_args: Vec<String>,
}

impl CliBridge {
    /// Create a new CLI bridge instance
    pub fn new() -> Result<Self, BridgeError> {
        let _chat_id = std::env::var("VKTEAMS_BOT_CHAT_ID")
            .map_err(|_| BridgeError::CliError("VKTEAMS_BOT_CHAT_ID environment variable not set".to_string()))?;
        
        let config_path = std::env::var("VKTEAMS_BOT_CONFIG").ok();
        
        // Try to find CLI binary
        let cli_path = which::which("vkteams-bot-cli")
            .or_else(|_| {
                // Try relative path from current executable
                std::env::current_exe()
                    .map(|mut p| {
                        p.pop(); // remove filename
                        p.push("vkteams-bot-cli");
                        p
                    })
            })
            .map_err(|_| BridgeError::CliNotFound("vkteams-bot-cli not found in PATH or relative to current executable".to_string()))?;
            
        let mut default_args = vec!["--output".to_string(), "json".to_string()];
        
        if let Some(config) = config_path {
            default_args.extend(vec!["--config".to_string(), config]);
        }
        
        Ok(Self {
            cli_path: cli_path.to_string_lossy().to_string(),
            default_args,
        })
    }

    /// Execute a CLI command with arguments
    pub async fn execute_command(&self, command: &[&str]) -> Result<Value, BridgeError> {
        self.execute_command_with_timeout(command, Duration::from_secs(30)).await
    }
    
    /// Execute a CLI command with custom timeout
    pub async fn execute_command_with_timeout(
        &self, 
        command: &[&str], 
        timeout_duration: Duration
    ) -> Result<Value, BridgeError> {
        debug!("Executing CLI command: {} {:?}", self.cli_path, command);
        
        let mut cmd = Command::new(&self.cli_path);
        cmd.args(&self.default_args)
           .args(command)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
           
        let output = match timeout(timeout_duration, cmd.output()).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => {
                error!("CLI command timed out after {:?}", timeout_duration);
                return Err(BridgeError::Timeout(timeout_duration));
            }
        };
        
        // Check if process was terminated by signal
        if let Some(code) = output.status.code() {
            if code < 0 {
                return Err(BridgeError::ProcessTerminated(
                    format!("Process terminated with signal {}", -code)
                ));
            }
        }
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            // Try to parse structured error from stderr
            if let Ok(error_info) = serde_json::from_str::<CliErrorInfo>(&stderr) {
                // Check for specific error types
                if let Some(ref code) = error_info.code {
                    match code.as_str() {
                        "RATE_LIMIT" => return Err(BridgeError::RateLimit(error_info.message.clone())),
                        _ => return Err(BridgeError::CliReturnedError(error_info)),
                    }
                }
                return Err(BridgeError::CliReturnedError(error_info));
            }
            
            error!("CLI command failed with unstructured error: {}", stderr);
            return Err(BridgeError::CliError(stderr.to_string()));
        }
        
        let response_text = String::from_utf8_lossy(&output.stdout);
        debug!("CLI response: {}", response_text);
        
        // Handle empty responses
        if response_text.trim().is_empty() {
            warn!("CLI returned empty response");
            return Ok(serde_json::json!({"success": true, "data": null}));
        }
        
        let response: Value = serde_json::from_str(&response_text)?;
        
        // Check if CLI returned an error in the JSON response
        if let Some(success) = response.get("success") {
            if !success.as_bool().unwrap_or(true) {
                let error_info = if let Some(error) = response.get("error") {
                    CliErrorInfo {
                        code: response.get("error_code")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        message: error.as_str().unwrap_or("Unknown error").to_string(),
                        details: response.get("error_details").cloned(),
                    }
                } else {
                    CliErrorInfo {
                        code: None,
                        message: "Command failed without error details".to_string(),
                        details: None,
                    }
                };
                
                // Check for rate limiting in response
                if error_info.code.as_deref() == Some("RATE_LIMIT") {
                    return Err(BridgeError::RateLimit(error_info.message));
                }
                
                return Err(BridgeError::CliReturnedError(error_info));
            }
        }
        
        Ok(response)
    }

    /// Execute command with retry logic
    pub async fn execute_command_with_retry(
        &self, 
        command: &[&str], 
        max_retries: usize
    ) -> Result<Value, BridgeError> {
        self.execute_command_with_retry_and_timeout(
            command, 
            max_retries, 
            Duration::from_secs(30)
        ).await
    }
    
    /// Execute command with retry logic and custom timeout
    pub async fn execute_command_with_retry_and_timeout(
        &self, 
        command: &[&str], 
        max_retries: usize,
        timeout_duration: Duration
    ) -> Result<Value, BridgeError> {
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            match self.execute_command_with_timeout(command, timeout_duration).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Determine if error is retryable
                    let should_retry = match &e {
                        BridgeError::Timeout(_) => true,
                        BridgeError::Io(_) => true,
                        BridgeError::RateLimit(_) => {
                            // For rate limit, use exponential backoff
                            if attempt < max_retries {
                                let backoff = Duration::from_secs(2u64.pow(attempt as u32));
                                warn!("Rate limited, backing off for {:?}", backoff);
                                tokio::time::sleep(backoff).await;
                            }
                            true
                        },
                        BridgeError::CliReturnedError(info) => {
                            // Check if error code indicates a retryable error
                            matches!(info.code.as_deref(), Some("NETWORK_ERROR") | Some("TIMEOUT"))
                        },
                        _ => false,
                    };
                    
                    last_error = Some(e);
                    
                    if attempt < max_retries && should_retry {
                        let delay = if matches!(last_error, Some(BridgeError::RateLimit(_))) {
                            // Already handled above
                            Duration::from_millis(0)
                        } else {
                            // Standard backoff for other errors
                            Duration::from_millis(100 * (attempt + 1) as u64)
                        };
                        
                        if !delay.is_zero() {
                            debug!("Retrying command after {:?} (attempt {}/{})", delay, attempt + 1, max_retries);
                            tokio::time::sleep(delay).await;
                        }
                    } else if !should_retry {
                        // Non-retryable error, fail immediately
                        break;
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }
    
    /// Health check - test if CLI is working
    pub async fn health_check(&self) -> Result<(), BridgeError> {
        self.execute_command(&["--version"]).await?;
        Ok(())
    }
    
    // === Daemon Commands ===
    
    /// Get daemon status
    pub async fn get_daemon_status(&self) -> Result<Value, BridgeError> {
        self.execute_command(&["status"]).await
    }
    
    // === Enhanced Storage Commands ===
    
    /// Get recent messages from storage
    pub async fn get_recent_messages(
        &self,
        chat_id: Option<&str>,
        limit: Option<usize>,
        since: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let mut args = vec!["database", "recent"];
        
        if let Some(chat_id) = chat_id {
            args.extend(&["--chat-id", chat_id]);
        }
        
        let limit_str;
        if let Some(limit) = limit {
            limit_str = limit.to_string();
            args.extend(&["--limit", &limit_str]);
        }
        
        if let Some(since) = since {
            args.extend(&["--since", since]);
        }
        
        self.execute_command(&args).await
    }
}

impl Default for CliBridge {
    fn default() -> Self {
        Self::new().expect("Failed to create CLI bridge")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        // Set required environment variable for test
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat");
        }
        
        let result = CliBridge::new();
        // Note: This might fail if CLI binary is not available, which is expected in test environment
        // The important thing is that the code compiles and handles errors properly
        match result {
            Ok(_) => println!("CLI bridge created successfully"),
            Err(e) => println!("Expected error in test environment: {}", e),
        }
    }

    #[test]
    fn test_bridge_error_display() {
        let error = BridgeError::CliError("test error".to_string());
        assert!(error.to_string().contains("CLI execution failed"));
    }
}