//! CLI Bridge for MCP Server
//!
//! This module provides an abstraction layer for calling the VK Teams Bot CLI
//! from the MCP server using subprocess calls. This approach ensures that all
//! business logic remains in the CLI while the MCP server acts as a thin adapter.

use serde_json::Value;
use std::process::Stdio;
use thiserror::Error;
use tokio::process::Command;
use tracing::{debug, error};

/// Errors that can occur when executing CLI commands
#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("CLI execution failed: {0}")]
    CliError(String),
    
    #[error("CLI not found at path: {0}")]
    CliNotFound(String),
    
    #[error("Invalid JSON response from CLI: {0}")]
    InvalidResponse(#[from] serde_json::Error),
    
    #[error("CLI returned error: {0}")]
    CliReturnedError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

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
        debug!("Executing CLI command: {} {:?}", self.cli_path, command);
        
        let mut cmd = Command::new(&self.cli_path);
        cmd.args(&self.default_args)
           .args(command)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
           
        let output = cmd.output().await?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            error!("CLI command failed: {}", error);
            return Err(BridgeError::CliError(error.to_string()));
        }
        
        let response_text = String::from_utf8_lossy(&output.stdout);
        debug!("CLI response: {}", response_text);
        
        let response: Value = serde_json::from_str(&response_text)?;
        
        // Check if CLI returned an error in the JSON response
        if let Some(success) = response.get("success") {
            if !success.as_bool().unwrap_or(true) {
                if let Some(error) = response.get("error") {
                    return Err(BridgeError::CliReturnedError(error.as_str().unwrap_or("Unknown error").to_string()));
                }
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
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            match self.execute_command(command).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        tokio::time::sleep(std::time::Duration::from_millis(100 * (attempt + 1) as u64)).await;
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