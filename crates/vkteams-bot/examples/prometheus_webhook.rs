use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use tracing::{error, info};
use vkteams_bot::prelude::*;
// Environment variable for the chat id
const CHAT_ID: &str = "VKTEAMS_BOT_CHAT_ID";
const TMPL_NAME: &str = "alert";
// define the Tera template
pub static TEMPLATES: LazyLock<tera::Tera> =
    LazyLock::new(|| match tera::Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            error!("Error parsing templates: {}", e);
            std::process::exit(1);
        }
    });
// }
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

    fn get_path(&self) -> Result<String> {
        Ok(self.path.clone())
    }

    async fn handler(&self, msg: Self::WebhookType) -> Result<()> {
        // Parse the webhook message and render inti template
        let parser = MessageTextParser::from_tmpl(TEMPLATES.to_owned()).set_ctx(msg, TMPL_NAME);
        // Make request for bot API
        let req = RequestMessagesSendText::new(self.chat_id.to_owned()).set_text(parser)?;
        // Send request to the bot API
        self.bot.send_api_request(req).await?;
        Ok(())
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    // Load .env file
    dotenvy::dotenv().expect("unable to load .env file");
    // Initialize logger
    let _guard = otlp::init().map_err(|e| BotError::Otlp(e.into()))?;
    info!("Starting...");
    // Run the web app
    vkteams_bot::bot::webhook::run_app(ExtendState::default()).await
}
