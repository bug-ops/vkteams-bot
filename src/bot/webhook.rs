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
