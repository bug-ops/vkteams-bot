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
//! - `VKTEAMS_BOT_SERVER_PORT` - server port (default: 3000)

use crate::bot::net::shutdown_signal;
use anyhow::Result;
use async_trait::async_trait;
use axum::extract::FromRef;
use axum::{
    extract::State,
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use serde::de::DeserializeOwned;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
/// Environment variable for the port
const DEFAULT_TCP_PORT: &str = "VKTEAMS_BOT_HTTP_PORT";
const TIMEOUT_SECS: u64 = 10;
// State for the webhook
#[derive(Default, Debug, Clone)]
pub struct AppState<T>
where
    T: Default + WebhookState + Clone + Send + Sync + 'static,
{
    pub ext: T,
}
/// Trait for the webhook state
#[async_trait]
pub trait WebhookState: Clone + Send + Sync + 'static {
    type WebhookType: DeserializeOwned;
    fn get_path(&self) -> String;
    fn deserialize(&self, json: String) -> Result<Self::WebhookType> {
        serde_json::from_str(&json).map_err(|e| e.into())
    }
    async fn handler(&self, msg: Self::WebhookType) -> Result<()>;
}
/// Run the webhook consumer server
pub async fn run_app<T>(ext: T) -> Result<()>
where
    T: WebhookState + FromRef<AppState<T>> + Default + 'static,
{
    // Get the port from the environment variable or use the default port 3000
    let tcp_port = std::env::var(DEFAULT_TCP_PORT).unwrap_or_else(|_| "3333".to_string());
    // Bind the server to the port
    let listener = TcpListener::bind(format!("[::]:{tcp_port}")).await?;
    // build our application with a single route
    trace!("Listening localhost:{tcp_port}{}", ext.get_path());
    let app = Router::new()
        .route(ext.get_path().as_str(), post(webhook_handler::<T>))
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(TIMEOUT_SECS)),
            CorsLayer::new()
                .allow_origin(AllowOrigin::predicate(|_, _| true))
                .allow_methods([Method::POST]),
        ))
        .with_state(AppState { ext });
    // Start the server
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
/// Handler for the webhook
async fn webhook_handler<T>(State(state): State<T>, json: String) -> impl IntoResponse
where
    T: WebhookState + Default + Clone + Send + Sync + 'static,
{
    trace!("Received webhook. Trying to deserialize");
    let msg = match state.deserialize(json) {
        Ok(msg) => msg,
        Err(e) => {
            error!("Failed to deserialize webhook: {}", e);
            return StatusCode::BAD_REQUEST;
        }
    };
    trace!("Deserialized webhook. Handling");
    state
        .handler(msg)
        .await
        .map(|_| StatusCode::OK)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}
