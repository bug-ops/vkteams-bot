#[macro_use]
extern crate log;

use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::*;
use vkteams_bot::bot::webhook::*;
use vkteams_bot::prelude::*;
// Environment variable for the chat id
const CHAT_ID: &str = "VKTEAMS_CHAT_ID";
// State for the webhook
#[derive(Debug, Clone)]
struct ApiState {
    bot: Bot,
    chat_id: ChatId,
    path: String,
}
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrometheusMessage {
    pub alerts: Vec<Alert>,
    pub common_annotations: Option<HashMap<String, String>>,
    pub common_labels: Option<HashMap<String, String>>,
    pub external_url: Option<String>,
    pub group_key: Option<String>,
    pub group_labels: Option<HashMap<String, String>>,
    pub truncated_alerts: Option<u32>,
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

impl WebhookState for ApiState {
    type WebhookType = PrometheusMessage;

    fn new(bot: Bot) -> Self {
        let chat_id = ChatId(
            std::env::var(CHAT_ID)
                .expect("Unable to find VKTEAMS_CHAT_ID in .env file")
                .to_string(),
        );
        Self {
            bot,
            chat_id: chat_id.to_owned(),
            path: format!("/alert/{chat_id}"),
        }
    }
    fn get_path(&self) -> String {
        self.path.clone()
    }
    fn handler(&self, json: Self::WebhookType) -> Result<()> {
        let message = format!("Prometheus Alert: {:?}", json);
        let parser = MessageTextParser::default().add(MessageTextFormat::Plain(message));
        let req = RequestMessagesSendText::new(self.chat_id.to_owned()).set_text(parser);
        move || async { self.bot.send_api_request(req).await };
        Ok(())
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    // Load .env file
    dotenvy::dotenv().expect("unable to load .env file");
    // Initialize logger
    pretty_env_logger::init();
    info!("Starting...");
    // Make bot
    let state = ApiState::new(Bot::default());
    webhook(state).await
}
