//! # Webhook consumer server
//! This module provides a webhook consumer server for the VK Teams Bot API.
//! This crate provides a client for the [VK Teams Bot API] V1.
//! Asynchronous request is based on [`reqwest`] and [`tokio`].
//! Server webhook consumer is based on [`axum`].
//! JSON Serialization and Deserialization [`serde_json`].
//!
//! ```toml
//! [dependencies]
//! vkteams_bot = { version = "0.6", features = ["webhook"] }
//! log = "0.4"
//! ```
//!
//! [VK Teams Bot API]: https://teams.vk.com/botapi/?lang=en
//! [`axum`]: https://docs.rs/axum
//! # Environment
//! - `RUST_LOG` - log level (default: `info`)
//! - `VKTEAMS_BOT_API_TOKEN` - bot token
//! - `VKTEAMS_BOT_API_URL` - bot api url
//! - `VKTEAMS_PROXY` - proxy url (optional)
//! - `VKTEAMS_BOT_SERVER_PORT` - server port (default: 3333)
#[cfg(feature = "grpc")]
use crate::bot::grpc::GRPCRouter;
use crate::bot::net::shutdown_signal;
use crate::error::{BotError, Result};
use async_trait::async_trait;
use axum::extract::FromRef;
use axum::{
    Router,
    extract::{DefaultBodyLimit, State},
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::post,
};
use serde::de::DeserializeOwned;
use std::time::Duration;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info, trace};

/// Environment variable for the port
const DEFAULT_TCP_PORT: &str = "VKTEAMS_TCP_PORT";
const TIMEOUT_SECS: u64 = 5;

// State for the webhook
#[derive(Default, Debug, Clone)]
pub struct AppState<T>
where
    T: Default + WebhookState + Clone + Send + Sync + 'static,
{
    pub ext: T,
}

/// Trait for webhook state
#[async_trait]
pub trait WebhookState: Clone + Send + Sync + 'static {
    type WebhookType: DeserializeOwned;

    /// Get webhook path
    ///
    /// ## Errors
    /// - `BotError::Config` - configuration error
    fn get_path(&self) -> Result<String>;

    /// Deserialize JSON into webhook type
    ///
    /// ## Errors
    /// - `BotError::Serialization` - deserialization error
    fn deserialize(&self, json: String) -> Result<Self::WebhookType> {
        serde_json::from_str(&json).map_err(BotError::Serialization)
    }

    /// Process webhook message
    ///
    /// ## Errors
    /// - `BotError::Api` - API error when processing message
    /// - `BotError::Network` - network error when sending request
    /// - `BotError::Serialization` - serialization/deserialization error
    async fn handler(&self, msg: Self::WebhookType) -> Result<()>;
}

/// Trait for bot router
pub trait BotRouter<S> {
    fn route_bot(self) -> Self;
}

impl<S> BotRouter<S> for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn route_bot(self) -> Self {
        if cfg!(feature = "grpc") {
            self.route_grpc_probe()
        } else {
            self
        }
    }
}

/// Run the webhook consumer server
///
/// ## Errors
/// - `BotError::Config` - configuration error (invalid port)
/// - `BotError::Network` - network error when starting server
/// - `BotError::System` - error when processing shutdown signals
pub async fn run_app<T>(ext: T) -> Result<()>
where
    T: WebhookState + FromRef<AppState<T>> + Default + 'static,
{
    let tcp_port = std::env::var(DEFAULT_TCP_PORT)
        .map_err(|e| BotError::Config(format!("Failed to get port: {}", e)))
        .unwrap_or_else(|e| panic!("{}", e));

    let listener = tokio::net::TcpListener::bind(format!("[::]:{tcp_port}")).await?;
    info!("Server started on localhost:{tcp_port}{}", ext.get_path()?);

    let app = build_router(ext)?;
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Build router for the webhook
///
/// ## Errors
/// - `BotError::Config` - configuration error (invalid path)
pub fn build_router<T>(ext: T) -> Result<Router>
where
    T: WebhookState + FromRef<AppState<T>> + Default + 'static,
{
    Ok(Router::new()
        .route(
            ext.get_path()?.as_str(),
            post(webhook_handler::<T>).layer((
                DefaultBodyLimit::disable(),
                RequestBodyLimitLayer::new(1024 * 5_000 /* ~5mb */),
            )),
        )
        .route_bot()
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(TIMEOUT_SECS)),
            CorsLayer::new()
                .allow_origin(AllowOrigin::predicate(|_, _| true))
                .allow_methods([Method::POST]),
        ))
        .with_state(AppState { ext }))
}

/// Handler for the webhook
async fn webhook_handler<T>(State(state): State<T>, json: String) -> impl IntoResponse
where
    T: WebhookState + Default + Clone + Send + Sync + 'static,
{
    trace!("Received webhook. Attempting deserialization");
    let msg = match state.deserialize(json) {
        Ok(msg) => msg,
        Err(e) => {
            error!("Error deserializing webhook: {}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    trace!("Webhook deserialized. Processing");
    match state.handler(msg).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            error!("Error processing webhook: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ApiError;
    use crate::error::BotError;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde::{Deserialize, Serialize};
    use tower::ServiceExt; // for .oneshot

    #[derive(Clone, Default, Debug, Serialize, Deserialize)]
    struct DummyWebhookType {
        pub value: String,
    }

    #[derive(Clone, Default)]
    struct DummyState {
        pub fail_path: bool,
        pub fail_deserialize: bool,
        pub fail_handler: bool,
    }

    #[async_trait]
    impl WebhookState for DummyState {
        type WebhookType = DummyWebhookType;

        fn get_path(&self) -> Result<String> {
            if self.fail_path {
                Err(BotError::Config("bad path".to_string()))
            } else {
                Ok("/webhook".to_string())
            }
        }

        fn deserialize(&self, json: String) -> Result<Self::WebhookType> {
            if self.fail_deserialize {
                Err(BotError::Serialization(serde_json::Error::io(
                    std::io::Error::other("fail deserialize"),
                )))
            } else {
                serde_json::from_str(&json).map_err(BotError::Serialization)
            }
        }

        async fn handler(&self, msg: Self::WebhookType) -> Result<()> {
            if self.fail_handler {
                Err(BotError::Api(ApiError {
                    description: String::from("fail handler"),
                }))
            } else if msg.value == "error" {
                Err(BotError::Api(ApiError {
                    description: String::from("error value"),
                }))
            } else {
                Ok(())
            }
        }
    }

    impl FromRef<AppState<DummyState>> for DummyState {
        fn from_ref(state: &AppState<DummyState>) -> DummyState {
            state.ext.clone()
        }
    }

    #[tokio::test]
    async fn test_build_router_success() {
        let state = DummyState::default();
        let router = build_router(state);
        assert!(router.is_ok());
    }

    #[tokio::test]
    async fn test_build_router_fail_path() {
        let state = DummyState {
            fail_path: true,
            ..Default::default()
        };
        let router = build_router(state);
        assert!(router.is_err());
    }

    #[tokio::test]
    async fn test_webhook_handler_success() {
        let state = DummyState::default();
        let router = build_router(state).unwrap();
        let payload = serde_json::json!({"value": "ok"}).to_string();
        let req = Request::builder()
            .method("POST")
            .uri("/webhook")
            .header("content-type", "application/json")
            .body(Body::from(payload))
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_webhook_handler_deserialize_error() {
        let state = DummyState {
            fail_deserialize: true,
            ..Default::default()
        };
        let router = build_router(state).unwrap();
        let payload = serde_json::json!({"value": "ok"}).to_string();
        let req = Request::builder()
            .method("POST")
            .uri("/webhook")
            .header("content-type", "application/json")
            .body(Body::from(payload))
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_webhook_handler_handler_error() {
        let state = DummyState {
            fail_handler: true,
            ..Default::default()
        };
        let router = build_router(state).unwrap();
        let payload = serde_json::json!({"value": "ok"}).to_string();
        let req = Request::builder()
            .method("POST")
            .uri("/webhook")
            .header("content-type", "application/json")
            .body(Body::from(payload))
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_webhook_handler_error_value() {
        let state = DummyState::default();
        let router = build_router(state).unwrap();
        let payload = serde_json::json!({"value": "error"}).to_string();
        let req = Request::builder()
            .method("POST")
            .uri("/webhook")
            .header("content-type", "application/json")
            .body(Body::from(payload))
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_webhook_handler_invalid_json() {
        let state = DummyState::default();
        let router = build_router(state).unwrap();
        let payload = "{invalid json";
        let req = Request::builder()
            .method("POST")
            .uri("/webhook")
            .header("content-type", "application/json")
            .body(Body::from(payload))
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_dummy_webhook_type_serialization() {
        let webhook = DummyWebhookType {
            value: "test_value".to_string(),
        };

        let serialized = serde_json::to_string(&webhook).unwrap();
        assert!(serialized.contains("test_value"));

        let deserialized: DummyWebhookType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.value, "test_value");
    }

    #[test]
    fn test_dummy_state_creation() {
        let state = DummyState::default();
        assert!(!state.fail_path);
        assert!(!state.fail_deserialize);
        assert!(!state.fail_handler);

        let state_with_failures = DummyState {
            fail_path: true,
            fail_deserialize: true,
            fail_handler: true,
        };
        assert!(state_with_failures.fail_path);
        assert!(state_with_failures.fail_deserialize);
        assert!(state_with_failures.fail_handler);
    }

    #[test]
    fn test_app_state_creation() {
        let dummy_state = DummyState::default();
        let app_state = AppState {
            ext: dummy_state.clone(),
        };

        // Test that AppState wraps the state correctly
        assert_eq!(app_state.ext.fail_path, dummy_state.fail_path);
    }

    #[test]
    fn test_webhook_state_deserialize_default() {
        let state = DummyState::default();
        let json = r#"{"value": "test"}"#;

        let result = state.deserialize(json.to_string());
        assert!(result.is_ok());

        let webhook = result.unwrap();
        assert_eq!(webhook.value, "test");
    }

    #[test]
    fn test_webhook_state_deserialize_invalid() {
        let state = DummyState::default();
        let json = r#"{"invalid": "json"}"#; // Missing "value" field

        let result = state.deserialize(json.to_string());
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_webhook_state_handler_success() {
        let state = DummyState::default();
        let webhook = DummyWebhookType {
            value: "success".to_string(),
        };

        let result = state.handler(webhook).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_webhook_state_handler_error_value() {
        let state = DummyState::default();
        let webhook = DummyWebhookType {
            value: "error".to_string(),
        };

        let result = state.handler(webhook).await;
        assert!(result.is_err());

        if let Err(BotError::Api(api_error)) = result {
            assert_eq!(api_error.description, "error value");
        } else {
            panic!("Expected Api error");
        }
    }

    #[test]
    fn test_from_ref_implementation() {
        let dummy_state = DummyState {
            fail_path: true,
            fail_deserialize: false,
            fail_handler: true,
        };
        let app_state = AppState {
            ext: dummy_state.clone(),
        };

        let extracted_state = DummyState::from_ref(&app_state);
        assert_eq!(extracted_state.fail_path, dummy_state.fail_path);
        assert_eq!(
            extracted_state.fail_deserialize,
            dummy_state.fail_deserialize
        );
        assert_eq!(extracted_state.fail_handler, dummy_state.fail_handler);
    }

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_TCP_PORT, "VKTEAMS_TCP_PORT");
        assert_eq!(TIMEOUT_SECS, 5);
    }

    #[tokio::test]
    async fn test_webhook_handler_different_paths() {
        // Test with custom webhook path
        let state = DummyState::default();
        let router = build_router(state).unwrap();

        // Test valid path
        let payload = serde_json::json!({"value": "test"}).to_string();
        let req = Request::builder()
            .method("POST")
            .uri("/webhook")
            .header("content-type", "application/json")
            .body(Body::from(payload))
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_webhook_handler_wrong_method() {
        let state = DummyState::default();
        let router = build_router(state).unwrap();

        // Test GET method (should fail as only POST is allowed)
        let req = Request::builder()
            .method("GET")
            .uri("/webhook")
            .body(Body::empty())
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[test]
    fn test_bot_router_trait() {
        // Test that BotRouter trait can be used with a concrete state type
        let router: Router<AppState<DummyState>> = Router::new();
        let _routed = router.route_bot();

        // If we get here without panicking, the trait works
        // We can't easily test the internal routing without more complex setup
    }
}
