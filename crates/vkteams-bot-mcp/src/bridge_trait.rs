//! Bridge trait for testable CLI interaction
//!
//! This module defines traits for CLI bridge to enable mocking and testing

use crate::errors::BridgeError;
use async_trait::async_trait;
use serde_json::Value;

/// Trait for CLI bridge operations - enables mocking for tests
#[async_trait]
pub trait CliBridgeTrait: Send + Sync + std::fmt::Debug {
    /// Execute a CLI command with arguments
    async fn execute_command(&self, args: &[&str]) -> Result<Value, BridgeError>;
    
    /// Get daemon status
    async fn get_daemon_status(&self) -> Result<Value, BridgeError>;
    
    /// Health check
    async fn health_check(&self) -> Result<(), BridgeError>;
}

/// Mock CLI bridge for testing
#[cfg(test)]
#[derive(Debug)]
pub struct MockCliBridge {
    pub responses: std::collections::HashMap<String, Result<Value, BridgeError>>,
}

#[cfg(test)]
impl Default for MockCliBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl MockCliBridge {
    pub fn new() -> Self {
        Self {
            responses: std::collections::HashMap::new(),
        }
    }
    
    pub fn add_response(&mut self, command: String, response: Result<Value, BridgeError>) {
        self.responses.insert(command, response);
    }
    
    pub fn add_success_response(&mut self, command: String, data: Value) {
        let command_clone = command.clone();
        self.responses.insert(command, Ok(serde_json::json!({
            "success": true,
            "data": data,
            "timestamp": "2024-01-01T00:00:00Z",
            "command": command_clone
        })));
    }
    
    pub fn add_error_response(&mut self, command: String, error: String) {
        self.responses.insert(command, Err(BridgeError::CliReturnedError(
            crate::errors::CliErrorInfo {
                code: Some("ERROR".to_string()),
                message: error,
                details: None,
            }
        )));
    }
}

#[cfg(test)]
#[async_trait]
impl CliBridgeTrait for MockCliBridge {
    async fn execute_command(&self, args: &[&str]) -> Result<Value, BridgeError> {
        let command_key = args.join(" ");
        
        // Try exact match first
        if let Some(response) = self.responses.get(&command_key) {
            return match response {
                Ok(value) => Ok(value.clone()),
                Err(err) => Err(err.clone()),
            };
        }
        
        // Try partial matches for flexibility
        for (key, response) in &self.responses {
            if command_key.contains(key) || key.contains(&command_key) {
                return match response {
                    Ok(value) => Ok(value.clone()),
                    Err(err) => Err(err.clone()),
                };
            }
        }
        
        // Default success response for unknown commands
        Ok(serde_json::json!({
            "success": true,
            "data": {},
            "timestamp": "2024-01-01T00:00:00Z",
            "command": command_key
        }))
    }
    
    async fn get_daemon_status(&self) -> Result<Value, BridgeError> {
        self.execute_command(&["daemon", "status"]).await
    }
    
    async fn health_check(&self) -> Result<(), BridgeError> {
        self.execute_command(&["--version"]).await.map(|_| ())
    }
}

/// Helper function to create a command key from arguments
pub fn make_command_key(args: &[&str]) -> String {
    args.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_bridge_success() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "send-text Hello".to_string(),
            serde_json::json!({"message_id": "123"})
        );
        
        let result = mock.execute_command(&["send-text", "Hello"]).await.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["data"]["message_id"], "123");
    }
    
    #[tokio::test]
    async fn test_mock_bridge_error() {
        let mut mock = MockCliBridge::new();
        mock.add_error_response(
            "send-text".to_string(),
            "Invalid message".to_string()
        );
        
        let result = mock.execute_command(&["send-text", "Invalid"]).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            BridgeError::CliReturnedError(info) => {
                assert_eq!(info.message, "Invalid message");
            }
            _ => panic!("Expected CliReturnedError"),
        }
    }
    
    #[tokio::test]
    async fn test_mock_bridge_partial_match() {
        let mut mock = MockCliBridge::new();
        mock.add_success_response(
            "send-text".to_string(),
            serde_json::json!({"result": "sent"})
        );
        
        let result = mock.execute_command(&["send-text", "Hello", "World"]).await.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["data"]["result"], "sent");
    }
    
    #[tokio::test]
    async fn test_mock_bridge_default_response() {
        let mock = MockCliBridge::new();
        
        let result = mock.execute_command(&["unknown", "command"]).await.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["command"], "unknown command");
    }
}