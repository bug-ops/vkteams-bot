#[macro_use]
extern crate log;
use anyhow::Result;
use async_trait::async_trait;
use axum::extract::FromRef;
use serde::Deserialize;
use std::collections::HashMap;
use vkteams_bot::bot::webhook::*;
use vkteams_bot::prelude::*;
// Environment variable for the chat id
const CHAT_ID: &str = "VKTEAMS_CHAT_ID";

#[derive(Debug, Clone)]
pub struct ExtendState {
    bot: Bot,
    chat_id: ChatId,
    path: String,
}
// Must implement FromRef trait to extract the substate
impl FromRef<AppState<ExtendState>> for ExtendState {
    fn from_ref(state: &AppState<ExtendState>) -> Self {
        state.ext.to_owned()
    }
}

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

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrometheusMessage {
    pub alerts: Vec<Alert>,
    pub common_annotations: Option<HashMap<String, String>>,
    pub common_labels: Option<HashMap<String, String>>,
    pub external_url: Option<String>,
    pub group_key: Option<f32>,
    pub group_labels: Option<HashMap<String, String>>,
    pub truncated_alerts: Option<f32>,
    pub receiver: String,
    pub status: AlertStatus,
    pub version: String,
}
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub annotations: HashMap<String, String>,
    pub ends_at: Option<String>,
    pub fingerprint: Option<String>,
    pub starts_at: Option<String>,
    pub status: Option<AlertStatus>,
    pub labels: HashMap<String, String>,
}
#[derive(Deserialize, Debug, Clone)]
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
        let message = format!("Prometheus Alert: {:?}", msg);

        let parser = MessageTextParser::default().add(MessageTextFormat::Plain(message));
        let req = RequestMessagesSendText::new(self.chat_id.to_owned()).set_text(parser);
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
    // Run the app
    run_app_webhook(ExtendState::default()).await
}
