#[macro_use]
extern crate log;
use anyhow::Result;
use axum::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use vkteams_bot::bot::webhook::{AppState, WebhookState};
use vkteams_bot::prelude::*;
// Environment variable for the gRPC port
const DEFAULT_TCP_PORT: &str = "VKTEAMS_BOT_GRPC_PORT";
// Environment variable for the chat id
const CHAT_ID: &str = "VKTEAMS_CHAT_ID";
const TMPL_NAME: &str = "alert";
// define the Tera template
lazy_static::lazy_static! {
    static ref TEMPLATES:tera::Tera = {
        let mut tera = tera::Tera::default();
        tera.add_template_file(
            format!("examples/templates/{TMPL_NAME}.tmpl"),
            Some(TMPL_NAME),
        )
        .unwrap();
        tera
    };
}
#[derive(Debug, Clone)]
pub struct ExtendState {
    bot: Bot,
    chat_id: ChatId,
    path: String,
}
// Must implement FromRef trait to extract the substate
impl axum::extract::FromRef<AppState<ExtendState>> for ExtendState {
    fn from_ref(state: &AppState<ExtendState>) -> Self {
        state.ext.to_owned()
    }
}
// Default implementation for the ExtendState
impl Default for ExtendState {
    fn default() -> Self {
        let bot = Bot::default();
        let chat_id = ChatId(
            std::env::var(CHAT_ID)
                .expect("Unable to find VKTEAMS_CHAT_ID in .env file")
                .to_string(),
        );
        let path = format!("/alert/{chat_id}");
        Self { bot, chat_id, path }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrometheusMessage {
    pub alerts: Vec<Alert>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub common_annotations: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub common_labels: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_key: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_labels: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated_alerts: Option<f32>,
    pub receiver: String,
    pub status: AlertStatus,
    pub version: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub annotations: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AlertStatus>,
    pub labels: HashMap<String, String>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum AlertStatus {
    Resolved,
    Firing,
}
// Must implement WebhookState trait to handle the webhook
#[async_trait]
impl WebhookState for ExtendState {
    type WebhookType = PrometheusMessage;

    fn get_path(&self) -> String {
        self.path.clone()
    }

    async fn handler(&self, msg: Self::WebhookType) -> Result<()> {
        // Parse the webhook message and render inti template
        let parser = MessageTextParser::from_tmpl(TEMPLATES.to_owned()).set_ctx(msg, TMPL_NAME);
        // Make request for bot API
        let req = RequestMessagesSendText::new(self.chat_id.to_owned()).set_text(parser);
        // Send request to the bot API
        match self.bot.send_api_request(req).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    // Load .env file
    dotenvy::dotenv().expect("unable to load .env file");
    // Initialize logger
    pretty_env_logger::init();
    info!("Starting...");
    // Run the web app
    tokio::spawn(async move {
        vkteams_bot::bot::webhook::run_app(ExtendState::default())
            .await
            .unwrap();
    });
    // Run the gRPC health reporter
    run_probe_app().await?;

    Ok(())
}
// Make health reporter for gRPC
// https://github.com/grpc/grpc/blob/master/doc/health-checking.md
//
// supported in Kubernetes by default
// https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/#define-a-grpc-liveness-probe
pub async fn run_probe_app() -> Result<()> {
    use vkteams_bot::bot::net::shutdown_signal;
    // Create gRPC health reporter and service
    let (_, health_service) = tonic_health::server::health_reporter();
    // Get the port from the environment variable or use the default port 50555
    let tcp_port = std::env::var(DEFAULT_TCP_PORT).unwrap_or_else(|_| "50555".to_string());
    // Start gRPC server
    tonic::transport::Server::builder()
        .add_service(health_service)
        .serve_with_shutdown(
            format!("[::]:{tcp_port}").parse().unwrap(),
            shutdown_signal(),
        )
        .await?;
    Ok(())
}
