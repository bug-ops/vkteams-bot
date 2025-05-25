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
