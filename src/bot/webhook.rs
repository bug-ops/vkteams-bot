use crate::prelude::*;
use anyhow::Result;
use axum::{
    extract::{Json, State},
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
/// Environment variable for the port
const PORT: &str = "VKTEAMS_BOT_SERVER_PORT";
/// Trait for the webhook state
pub trait WebhookState {
    type WebhookType: Clone + Send + Sync + 'static;
    fn new(bot: Bot) -> Self;
    fn get_path(&self) -> String;
    async fn handler(&self, json: Self::WebhookType) -> Result<()>;
}
// impl WebhookHandler for Bot {
pub async fn webhook<S>(state: S) -> Result<(), anyhow::Error>
where
    S: WebhookState + Clone + Send + Sync + 'static,
{
    // Get the port from the environment variable or use the default port 3000
    let tcp_port = std::env::var(PORT).unwrap_or_else(|_| "3000".to_string());
    // build our application with a single route
    let app = Router::new()
        .route(state.get_path().as_str(), post(webhook_handler::<S>))
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(10)),
            CorsLayer::new()
                .allow_origin(AllowOrigin::predicate(|_, _| true))
                .allow_methods([Method::POST]),
        ));
    // .with_state(state);
    // Bind the server to the port
    let listener = TcpListener::bind(format!("0.0.0.0:{tcp_port}")).await?;
    // Start the server
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
struct PrometheusMessage {}
/// Handler for the webhook
async fn webhook_handler<S>(
    State(state): State<S>,
    Json(json): Json<<S as WebhookState>::WebhookType>,
) -> impl IntoResponse
where
    S: WebhookState + Clone + Send + Sync + 'static,
{
    match state.handler(json).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
/// Graceful shutdown signal
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
