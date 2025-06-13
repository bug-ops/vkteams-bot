use serde::{Deserialize, Serialize};
use std::env::VarError;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub description: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "API Error: {}", self.description)
    }
}

impl std::error::Error for ApiError {}

#[derive(Debug)]
pub struct OtlpError {
    pub message: String,
}

impl fmt::Display for OtlpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for OtlpError {}

impl From<String> for OtlpError {
    fn from(message: String) -> Self {
        OtlpError { message }
    }
}

impl From<Box<dyn std::error::Error>> for OtlpError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        OtlpError {
            message: err.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum BotError {
    /// API Error
    Api(ApiError),
    /// Network Error
    Network(reqwest::Error),
    /// gRPC Error
    #[cfg(feature = "grpc")]
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
    /// Otlp Error
    Otlp(OtlpError),
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
            BotError::Otlp(e) => write!(f, "Otlp Error: {}", e),
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
            BotError::Otlp(e) => Some(e),
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

impl From<toml::de::Error> for BotError {
    fn from(err: toml::de::Error) -> Self {
        BotError::Config(err.to_string())
    }
}

impl From<VarError> for BotError {
    fn from(err: VarError) -> Self {
        BotError::Config(err.to_string())
    }
}

impl From<ApiError> for BotError {
    fn from(err: ApiError) -> Self {
        BotError::Api(err)
    }
}

pub type Result<T> = std::result::Result<T, BotError>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::io;
    use url::ParseError;

    #[test]
    fn test_api_error_display() {
        let err = ApiError {
            description: "fail".to_string(),
        };
        assert_eq!(format!("{}", err), "API Error: fail");
    }

    #[test]
    fn test_otlp_error_display_and_from() {
        let err = OtlpError {
            message: "otlp fail".to_string(),
        };
        assert_eq!(format!("{}", err), "otlp fail");
        let from_str: OtlpError = "msg".to_string().into();
        assert_eq!(from_str.message, "msg");
        let boxed: OtlpError =
            Box::<dyn std::error::Error>::from(io::Error::new(io::ErrorKind::Other, "err")).into();
        assert!(boxed.message.contains("err"));
    }

    #[test]
    fn test_bot_error_display_and_from() {
        let api = ApiError {
            description: "api".to_string(),
        };
        let err = BotError::Api(api.clone());
        assert!(format!("{}", err).contains("API Error"));
        let ser = BotError::Serialization(serde_json::Error::io(io::Error::new(
            io::ErrorKind::Other,
            "ser",
        )));
        assert!(format!("{}", ser).contains("Serialization Error"));
        let url = BotError::Url(ParseError::EmptyHost);
        assert!(format!("{}", url).contains("URL Error"));
        let ioerr = BotError::Io(io::Error::new(io::ErrorKind::Other, "io"));
        assert!(format!("{}", ioerr).contains("IO Error"));
        let conf = BotError::Config("conf".to_string());
        assert!(format!("{}", conf).contains("Config Error"));
        let val = BotError::Validation("val".to_string());
        assert!(format!("{}", val).contains("Validation Error"));
        let sys = BotError::System("sys".to_string());
        assert!(format!("{}", sys).contains("System Error"));
        let otlp = BotError::Otlp(OtlpError {
            message: "otlp".to_string(),
        });
        assert!(format!("{}", otlp).contains("Otlp Error"));
    }

    #[test]
    fn test_bot_error_from_impls() {
        let _b: BotError =
            serde_json::Error::io(io::Error::new(io::ErrorKind::Other, "ser")).into();
        let _b: BotError = url::ParseError::EmptyHost.into();
        let _b: BotError = io::Error::new(io::ErrorKind::Other, "io").into();
        let _b: BotError = serde_url_params::Error::unsupported("params").into();
        let _b: BotError = toml::from_str::<i32>("not toml").unwrap_err().into();
        let _b: BotError = std::env::VarError::NotPresent.into();
        let _b: BotError = ApiError {
            description: "api".to_string(),
        }
        .into();
    }
}
