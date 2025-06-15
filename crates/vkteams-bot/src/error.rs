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
    /// Environment Error
    Environment(std::env::VarError),
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
            BotError::Environment(e) => write!(f, "Environment Error: {}", e),
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
            BotError::Environment(e) => Some(e),
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
        BotError::Environment(err)
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
        let boxed: OtlpError = Box::<dyn std::error::Error>::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "err",
        ))
        .into();
        assert!(boxed.message.contains("err"));
    }

    #[test]
    fn test_bot_error_display_and_from() {
        let api = ApiError {
            description: "api".to_string(),
        };
        let err = BotError::Api(api.clone());
        assert!(format!("{}", err).contains("API Error"));
        let ser = BotError::Serialization(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "ser",
        )));
        assert!(format!("{}", ser).contains("Serialization Error"));
        let url = BotError::Url(ParseError::EmptyHost);
        assert!(format!("{}", url).contains("URL Error"));
        let ioerr = BotError::Io(std::io::Error::new(std::io::ErrorKind::Other, "io"));
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
            serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::Other, "ser")).into();
        let _b: BotError = url::ParseError::EmptyHost.into();
        let _b: BotError = std::io::Error::new(std::io::ErrorKind::Other, "io").into();
        let _b: BotError = serde_url_params::Error::unsupported("params").into();
        let _b: BotError = toml::from_str::<i32>("not toml").unwrap_err().into();
        let _b: BotError = std::env::VarError::NotPresent.into();
        let _b: BotError = ApiError {
            description: "api".to_string(),
        }
        .into();
    }

    #[test]
    fn test_api_error_serialization() {
        let error = ApiError {
            description: "Test error message".to_string(),
        };

        // Test serialization
        let serialized = serde_json::to_string(&error).unwrap();
        assert!(serialized.contains("Test error message"));

        // Test deserialization
        let deserialized: ApiError = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.description, "Test error message");
    }

    #[test]
    fn test_api_error_clone() {
        let error = ApiError {
            description: "Cloneable error".to_string(),
        };
        let cloned = error.clone();
        assert_eq!(error.description, cloned.description);
    }

    #[test]
    fn test_api_error_as_std_error() {
        let error = ApiError {
            description: "Standard error test".to_string(),
        };

        // Test that it implements std::error::Error
        let std_err: &dyn std::error::Error = &error;
        assert!(std_err.source().is_none());
    }

    #[test]
    fn test_otlp_error_from_box_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let boxed: Box<dyn std::error::Error> = Box::new(io_error);
        let otlp_error: OtlpError = boxed.into();

        assert!(otlp_error.message.contains("Access denied"));
    }

    #[test]
    fn test_otlp_error_as_std_error() {
        let error = OtlpError {
            message: "OTLP connection failed".to_string(),
        };

        // Test that it implements std::error::Error
        let std_err: &dyn std::error::Error = &error;
        assert!(std_err.source().is_none());
    }

    #[tokio::test]
    async fn test_bot_error_source_chain() {
        use std::error::Error;

        // Test API error source
        let api_err = BotError::Api(ApiError {
            description: "API failed".to_string(),
        });
        assert!(api_err.source().is_some());

        // Test Network error source
        let network_err = BotError::Network(
            reqwest::ClientBuilder::new()
                .build()
                .unwrap()
                .get("http://invalid-url")
                .send()
                .await
                .unwrap_err(),
        );
        assert!(network_err.source().is_some());

        // Test Serialization error source
        let ser_err = BotError::Serialization(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "serialization",
        )));
        assert!(ser_err.source().is_some());

        // Test URL error source
        let url_err = BotError::Url(url::ParseError::EmptyHost);
        assert!(url_err.source().is_some());

        // Test IO error source
        let io_err = BotError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert!(io_err.source().is_some());

        // Test UrlParams error source
        let params_err = BotError::UrlParams(serde_url_params::Error::unsupported("test"));
        assert!(params_err.source().is_some());

        // Test OTLP error source
        let otlp_err = BotError::Otlp(OtlpError {
            message: "OTLP failed".to_string(),
        });
        assert!(otlp_err.source().is_some());

        // Test errors with no source
        let config_err = BotError::Config("Config error".to_string());
        assert!(config_err.source().is_none());

        let validation_err = BotError::Validation("Validation error".to_string());
        assert!(validation_err.source().is_none());

        let system_err = BotError::System("System error".to_string());
        assert!(system_err.source().is_none());
    }

    #[test]
    fn test_bot_error_debug_format() {
        let errors = vec![
            BotError::Api(ApiError {
                description: "Debug API error".to_string(),
            }),
            BotError::Config("Debug config error".to_string()),
            BotError::Validation("Debug validation error".to_string()),
            BotError::System("Debug system error".to_string()),
            BotError::Otlp(OtlpError {
                message: "Debug OTLP error".to_string(),
            }),
        ];

        for error in errors {
            let debug_str = error.to_string();
            assert!(!debug_str.is_empty());
            assert!(debug_str.contains("Error"));
        }
    }

    #[test]
    fn test_error_conversion_chain() {
        // Test VarError conversion chain
        let var_error = std::env::VarError::NotPresent;
        let bot_error: BotError = var_error.into();
        match bot_error {
            BotError::Environment(msg) => assert!(msg.to_string().contains("not found")),
            _ => panic!("Expected Config error"),
        }

        // Test TOML error conversion chain
        let toml_error = toml::from_str::<i32>("invalid toml").unwrap_err();
        let bot_error: BotError = toml_error.into();
        match bot_error {
            BotError::Config(_) => {} // Expected
            _ => panic!("Expected Config error"),
        }
    }

    #[tokio::test]
    async fn test_all_error_types_display() {
        let test_cases = vec![
            (
                BotError::Api(ApiError {
                    description: "API test".to_string(),
                }),
                "API Error",
            ),
            (
                BotError::Network(
                    reqwest::ClientBuilder::new()
                        .build()
                        .unwrap()
                        .get("http://test")
                        .send()
                        .await
                        .unwrap_err(),
                ),
                "Network Error",
            ),
            (
                BotError::Serialization(serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "test",
                ))),
                "Serialization Error",
            ),
            (BotError::Url(url::ParseError::EmptyHost), "URL Error"),
            (
                BotError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test")),
                "IO Error",
            ),
            (BotError::Config("test".to_string()), "Config Error"),
            (BotError::Validation("test".to_string()), "Validation Error"),
            (
                BotError::UrlParams(serde_url_params::Error::unsupported("test")),
                "URL Parameters Error",
            ),
            (BotError::System("test".to_string()), "System Error"),
            (
                BotError::Otlp(OtlpError {
                    message: "test".to_string(),
                }),
                "Otlp Error",
            ),
        ];

        for (error, expected_prefix) in test_cases {
            let display_str = format!("{}", error);
            assert!(
                display_str.contains(expected_prefix),
                "Error '{}' should contain '{}'",
                display_str,
                expected_prefix
            );
        }
    }

    #[cfg(feature = "templates")]
    #[test]
    fn test_template_error_conversion() {
        use tera::Tera;

        let tera = Tera::new("templates/*").unwrap_or_default();
        let template_error = tera
            .render("nonexistent", &tera::Context::new())
            .unwrap_err();
        let bot_error: BotError = template_error.into();

        match bot_error {
            BotError::Template(_) => {} // Expected
            _ => panic!("Expected Template error"),
        }

        let display_str = format!("{}", bot_error);
        assert!(display_str.contains("Template Error"));
    }

    #[test]
    fn test_result_type_alias() {
        // Test that our Result type alias works correctly
        fn test_function() -> Result<String> {
            Ok("success".to_string())
        }

        fn test_error_function() -> Result<String> {
            Err(BotError::Validation("test error".to_string()))
        }

        assert!(test_function().is_ok());
        assert_eq!(test_function().unwrap(), "success");

        assert!(test_error_function().is_err());
        match test_error_function().unwrap_err() {
            BotError::Validation(msg) => assert_eq!(msg, "test error"),
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_error_message_content() {
        // Test that error messages contain expected content
        let api_error = ApiError {
            description: "Specific API failure".to_string(),
        };
        assert!(format!("{}", api_error).contains("Specific API failure"));

        let otlp_error = OtlpError {
            message: "OTLP connection timeout".to_string(),
        };
        assert!(format!("{}", otlp_error).contains("OTLP connection timeout"));

        let config_error = BotError::Config("Missing required field".to_string());
        assert!(format!("{}", config_error).contains("Missing required field"));
    }
}

#[test]
fn test_api_error_serialization() {
    let error = ApiError {
        description: "Test error message".to_string(),
    };

    // Test serialization
    let serialized = serde_json::to_string(&error).unwrap();
    assert!(serialized.contains("Test error message"));

    // Test deserialization
    let deserialized: ApiError = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.description, "Test error message");
}

#[test]
fn test_api_error_clone() {
    let error = ApiError {
        description: "Cloneable error".to_string(),
    };
    let cloned = error.clone();
    assert_eq!(error.description, cloned.description);
}

#[test]
fn test_api_error_as_std_error() {
    let error = ApiError {
        description: "Standard error test".to_string(),
    };

    // Test that it implements std::error::Error
    let std_err: &dyn std::error::Error = &error;
    assert!(std_err.source().is_none());
}

#[test]
fn test_otlp_error_from_box_error() {
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let boxed: Box<dyn std::error::Error> = Box::new(io_error);
    let otlp_error: OtlpError = boxed.into();

    assert!(otlp_error.message.contains("Access denied"));
}

#[test]
fn test_otlp_error_as_std_error() {
    let error = OtlpError {
        message: "OTLP connection failed".to_string(),
    };

    // Test that it implements std::error::Error
    let std_err: &dyn std::error::Error = &error;
    assert!(std_err.source().is_none());
}

#[tokio::test]
async fn test_bot_error_source_chain() {
    use std::error::Error;
    // Test API error source
    let api_err = BotError::Api(ApiError {
        description: "API failed".to_string(),
    });
    assert!(api_err.source().is_some());

    // Test Network error source
    let network_err = BotError::Network(
        reqwest::ClientBuilder::new()
            .build()
            .unwrap()
            .get("http://invalid-url")
            .send()
            .await
            .unwrap_err(),
    );
    assert!(network_err.source().is_some());

    // Test Serialization error source
    let ser_err = BotError::Serialization(serde_json::Error::io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "serialization",
    )));
    assert!(ser_err.source().is_some());

    // Test URL error source
    let url_err = BotError::Url(url::ParseError::EmptyHost);
    assert!(url_err.source().is_some());

    // Test IO error source
    let io_err = BotError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "file not found",
    ));
    assert!(io_err.source().is_some());

    // Test UrlParams error source
    let params_err = BotError::UrlParams(serde_url_params::Error::unsupported("test"));
    assert!(params_err.source().is_some());

    // Test OTLP error source
    let otlp_err = BotError::Otlp(OtlpError {
        message: "OTLP failed".to_string(),
    });
    assert!(otlp_err.source().is_some());

    // Test errors with no source
    let config_err = BotError::Config("Config error".to_string());
    assert!(config_err.source().is_none());

    let validation_err = BotError::Validation("Validation error".to_string());
    assert!(validation_err.source().is_none());

    let system_err = BotError::System("System error".to_string());
    assert!(system_err.source().is_none());
}

#[test]
fn test_bot_error_debug_format() {
    let errors = vec![
        BotError::Api(ApiError {
            description: "Debug API error".to_string(),
        }),
        BotError::Config("Debug config error".to_string()),
        BotError::Validation("Debug validation error".to_string()),
        BotError::System("Debug system error".to_string()),
        BotError::Otlp(OtlpError {
            message: "Debug OTLP error".to_string(),
        }),
    ];

    for error in errors {
        let debug_str = error.to_string();
        assert!(!debug_str.is_empty());
        assert!(debug_str.contains("Error"));
    }
}

#[test]
fn test_error_conversion_chain() {
    // Test VarError conversion chain
    let var_error = std::env::VarError::NotPresent;
    let bot_error: BotError = var_error.into();
    match bot_error {
        BotError::Environment(msg) => assert!(msg.to_string().contains("not found")),
        _ => panic!("Expected Config error"),
    }

    // Test TOML error conversion chain
    let toml_error = toml::from_str::<i32>("invalid toml").unwrap_err();
    let bot_error: BotError = toml_error.into();
    match bot_error {
        BotError::Config(_) => {} // Expected
        _ => panic!("Expected Config error"),
    }
}

#[tokio::test]
async fn test_all_error_types_display() {
    let test_cases = vec![
        (
            BotError::Api(ApiError {
                description: "API test".to_string(),
            }),
            "API Error",
        ),
        (
            BotError::Network(
                reqwest::ClientBuilder::new()
                    .build()
                    .unwrap()
                    .get("http://test")
                    .send()
                    .await
                    .unwrap_err(),
            ),
            "Network Error",
        ),
        (
            BotError::Serialization(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "test",
            ))),
            "Serialization Error",
        ),
        (BotError::Url(url::ParseError::EmptyHost), "URL Error"),
        (
            BotError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test")),
            "IO Error",
        ),
        (BotError::Config("test".to_string()), "Config Error"),
        (BotError::Validation("test".to_string()), "Validation Error"),
        (
            BotError::UrlParams(serde_url_params::Error::unsupported("test")),
            "URL Parameters Error",
        ),
        (BotError::System("test".to_string()), "System Error"),
        (
            BotError::Otlp(OtlpError {
                message: "test".to_string(),
            }),
            "Otlp Error",
        ),
    ];

    for (error, expected_prefix) in test_cases {
        let display_str = format!("{}", error);
        assert!(
            display_str.contains(expected_prefix),
            "Error '{}' should contain '{}'",
            display_str,
            expected_prefix
        );
    }
}

#[cfg(feature = "templates")]
#[test]
fn test_template_error_conversion() {
    use tera::Tera;

    let tera = Tera::new("templates/*").unwrap_or_default();
    let template_error = tera
        .render("nonexistent", &tera::Context::new())
        .unwrap_err();
    let bot_error: BotError = template_error.into();

    match bot_error {
        BotError::Template(_) => {} // Expected
        _ => panic!("Expected Template error"),
    }

    let display_str = format!("{}", bot_error);
    assert!(display_str.contains("Template Error"));
}

#[test]
fn test_result_type_alias() {
    // Test that our Result type alias works correctly
    fn test_function() -> Result<String> {
        Ok("success".to_string())
    }

    fn test_error_function() -> Result<String> {
        Err(BotError::Validation("test error".to_string()))
    }

    assert!(test_function().is_ok());
    assert_eq!(test_function().unwrap(), "success");

    assert!(test_error_function().is_err());
    match test_error_function().unwrap_err() {
        BotError::Validation(msg) => assert_eq!(msg, "test error"),
        _ => panic!("Expected Validation error"),
    }
}

#[test]
fn test_error_message_content() {
    // Test that error messages contain expected content
    let api_error = ApiError {
        description: "Specific API failure".to_string(),
    };
    assert!(format!("{}", api_error).contains("Specific API failure"));

    let otlp_error = OtlpError {
        message: "OTLP connection timeout".to_string(),
    };
    assert!(format!("{}", otlp_error).contains("OTLP connection timeout"));

    let config_error = BotError::Config("Missing required field".to_string());
    assert!(format!("{}", config_error).contains("Missing required field"));
}
