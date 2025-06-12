use rmcp::Error;
use thiserror::Error;
use vkteams_bot::error::BotError;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("VKTeams Bot error: {0}")]
    Bot(#[from] BotError),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("RMCP error: {0}")]
    Rmcp(#[from] rmcp::Error),
    #[error("Other: {0}")]
    Other(String),
}

impl From<McpError> for Error {
    fn from(e: McpError) -> Self {
        match e {
            McpError::Bot(e) => Error::internal_error(e.to_string(), None),
            McpError::Serde(e) => Error::parse_error(e.to_string(), None),
            McpError::Rmcp(e) => Error::internal_error(e.to_string(), None),
            McpError::Other(e) => Error::internal_error(e, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::Error as RmcpError;
    use serde_json;

    #[test]
    fn test_mcp_error_bot() {
        let bot_err = BotError::Config("test bot error".to_string());
        let err = McpError::Bot(bot_err);
        let rmcp_err: Error = err.into();
        let msg = format!("{}", rmcp_err);
        assert!(msg.contains("test bot error"));
    }

    #[test]
    fn test_mcp_error_serde() {
        let serde_err = serde_json::from_str::<u32>("not_a_number").unwrap_err();
        let err = McpError::Serde(serde_err);
        let rmcp_err: Error = err.into();
        let msg = format!("{}", rmcp_err);
        assert!(msg.contains("expected ident") || msg.contains("expected"));
    }

    #[test]
    fn test_mcp_error_rmcp() {
        let rmcp_err = RmcpError::parse_error("rmcp parse error", None);
        let err = McpError::Rmcp(rmcp_err.clone());
        let rmcp_err2: Error = err.into();
        let msg = format!("{}", rmcp_err2);
        assert!(msg.contains("rmcp parse error"));
    }

    #[test]
    fn test_mcp_error_other() {
        let err = McpError::Other("other error".to_string());
        let rmcp_err: Error = err.into();
        let msg = format!("{}", rmcp_err);
        assert!(msg.contains("other error"));
    }
}
