use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub ok: bool,
    pub description: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "API Error: {}", self.description)
    }
}

impl std::error::Error for ApiError {}

#[derive(Debug)]
pub enum BotError {
    /// API Error
    Api(ApiError),
    /// Network Error
    Network(reqwest::Error),
    /// gRPC Error
    /// #[cfg(feature = "grpc")]
    Grpc(tonic::transport::Error),
    /// Serialization/Deserialization Error
    Serialization(serde_json::Error),
    /// URL Error
    Url(url::ParseError),
    /// File System Error
    Io(std::io::Error),
    /// Template Error
    #[cfg(feature = "templates")]
    Template(tera::Error),
    /// Configuration Error
    Config(String),
    /// Validation Error
    Validation(String),
    /// URL Parameters Error
    UrlParams(serde_url_params::Error),
    /// System Error
    System(String),
}

impl fmt::Display for BotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BotError::Api(e) => write!(f, "API Error: {}", e),
            BotError::Network(e) => write!(f, "Network Error: {}", e),
            #[cfg(feature = "grpc")]
            BotError::Grpc(e) => write!(f, "gRPC Error: {}", e),
            BotError::Serialization(e) => write!(f, "Serialization Error: {}", e),
            BotError::Url(e) => write!(f, "URL Error: {}", e),
            BotError::Io(e) => write!(f, "IO Error: {}", e),
            #[cfg(feature = "templates")]
            BotError::Template(e) => write!(f, "Template Error: {}", e),
            BotError::Config(e) => write!(f, "Config Error: {}", e),
            BotError::Validation(e) => write!(f, "Validation Error: {}", e),
            BotError::UrlParams(e) => write!(f, "URL Parameters Error: {}", e),
            BotError::System(e) => write!(f, "System Error: {}", e),
        }
    }
}

impl std::error::Error for BotError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BotError::Api(e) => Some(e),
            BotError::Network(e) => Some(e),
            #[cfg(feature = "grpc")]
            BotError::Grpc(e) => Some(e),
            BotError::Serialization(e) => Some(e),
            BotError::Url(e) => Some(e),
            BotError::Io(e) => Some(e),
            #[cfg(feature = "templates")]
            BotError::Template(e) => Some(e),
            BotError::Config(_) => None,
            BotError::Validation(_) => None,
            BotError::UrlParams(e) => Some(e),
            BotError::System(_) => None,
        }
    }
}

impl From<reqwest::Error> for BotError {
    fn from(err: reqwest::Error) -> Self {
        BotError::Network(err)
    }
}

impl From<serde_json::Error> for BotError {
    fn from(err: serde_json::Error) -> Self {
        BotError::Serialization(err)
    }
}

impl From<url::ParseError> for BotError {
    fn from(err: url::ParseError) -> Self {
        BotError::Url(err)
    }
}

impl From<std::io::Error> for BotError {
    fn from(err: std::io::Error) -> Self {
        BotError::Io(err)
    }
}

#[cfg(feature = "templates")]
impl From<tera::Error> for BotError {
    fn from(err: tera::Error) -> Self {
        BotError::Template(err)
    }
}

impl From<serde_url_params::Error> for BotError {
    fn from(err: serde_url_params::Error) -> Self {
        BotError::UrlParams(err)
    }
}
#[cfg(feature = "grpc")]
impl From<tonic::transport::Error> for BotError {
    fn from(err: tonic::transport::Error) -> Self {
        BotError::Grpc(err)
    }
}

pub type Result<T> = std::result::Result<T, BotError>;
