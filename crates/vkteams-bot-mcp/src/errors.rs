use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use vkteams_bot::error::BotError;

/// Structured error response from CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliErrorInfo {
    pub code: Option<String>,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Errors that can occur when executing CLI commands  
#[derive(Debug, Error, Clone)]
pub enum BridgeError {
    #[error("CLI execution failed: {0}")]
    CliError(String),

    #[error("CLI not found at path: {0}")]
    CliNotFound(String),

    #[error("Invalid JSON response from CLI: {0}")]
    InvalidResponse(String),

    #[error("CLI returned error: {}", .0.message)]
    CliReturnedError(CliErrorInfo),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Command timed out after {0:?}")]
    Timeout(Duration),

    #[error("CLI process terminated with signal: {0}")]
    ProcessTerminated(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
}

impl BridgeError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            BridgeError::Timeout(_) => true,
            BridgeError::Io(_) => true,
            BridgeError::RateLimit(_) => true,
            BridgeError::CliReturnedError(info) => {
                matches!(
                    info.code.as_deref(),
                    Some("NETWORK_ERROR") | Some("TIMEOUT") | Some("TEMPORARY_ERROR")
                )
            }
            _ => false,
        }
    }

    /// Get suggested retry delay based on error type
    pub fn retry_delay(&self) -> Duration {
        match self {
            BridgeError::RateLimit(_) => Duration::from_secs(60),
            BridgeError::Timeout(_) => Duration::from_secs(10),
            BridgeError::Io(_) => Duration::from_secs(5),
            _ => Duration::from_secs(2),
        }
    }
}

#[derive(Debug, Error)]
pub enum McpError {
    #[error("VKTeams Bot error: {0}")]
    Bot(#[from] BotError),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("RMCP error: {0}")]
    Rmcp(#[from] rmcp::ErrorData),
    #[error("Bridge error: {0}")]
    Bridge(#[from] BridgeError),
    #[error("Other: {0}")]
    Other(String),
}

impl From<McpError> for rmcp::ErrorData {
    fn from(e: McpError) -> Self {
        match e {
            McpError::Bot(e) => rmcp::ErrorData::internal_error(e.to_string(), None),
            McpError::Serde(e) => rmcp::ErrorData::parse_error(e.to_string(), None),
            McpError::Rmcp(e) => rmcp::ErrorData::internal_error(e.to_string(), None),
            McpError::Bridge(e) => rmcp::ErrorData::internal_error(e.to_string(), None),
            McpError::Other(e) => rmcp::ErrorData::internal_error(e, None),
        }
    }
}

// From implementations for BridgeError
impl From<serde_json::Error> for BridgeError {
    fn from(err: serde_json::Error) -> Self {
        BridgeError::InvalidResponse(err.to_string())
    }
}

impl From<std::io::Error> for BridgeError {
    fn from(err: std::io::Error) -> Self {
        BridgeError::Io(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::ErrorData as RmcpError;
    use serde_json;

    #[test]
    fn test_mcp_error_bot() {
        let bot_err = BotError::Config("test bot error".to_string());
        let err = McpError::Bot(bot_err);
        let rmcp_err: RmcpError = err.into();
        let msg = format!("{rmcp_err}");
        assert!(msg.contains("test bot error"));
    }

    #[test]
    fn test_mcp_error_serde() {
        let serde_err = serde_json::from_str::<u32>("not_a_number").unwrap_err();
        let err = McpError::Serde(serde_err);
        let rmcp_err: RmcpError = err.into();
        let msg = format!("{rmcp_err}");
        assert!(msg.contains("expected ident") || msg.contains("expected"));
    }

    #[test]
    fn test_mcp_error_rmcp() {
        let rmcp_err = RmcpError::parse_error("rmcp parse error", None);
        let err = McpError::Rmcp(rmcp_err.clone());
        let rmcp_err2: RmcpError = err.into();
        let msg = format!("{rmcp_err2}");
        assert!(msg.contains("rmcp parse error"));
    }

    #[test]
    fn test_mcp_error_other() {
        let err = McpError::Other("other error".to_string());
        let rmcp_err: RmcpError = err.into();
        let msg = format!("{rmcp_err}");
        assert!(msg.contains("other error"));
    }

    #[test]
    fn test_mcp_error_bridge() {
        let bridge_err = BridgeError::RateLimit("rate limit exceeded".to_string());
        let err = McpError::Bridge(bridge_err);
        let rmcp_err: RmcpError = err.into();
        let msg = format!("{rmcp_err}");
        assert!(msg.contains("rate limit"));
    }
}
