use std::sync::Arc;
use vkteams_bot::prelude::*;

#[derive(Debug, Clone)]
pub struct Server {
    pub bot: Arc<Bot>,
    pub chat_id: String,
}

impl Server {
    pub fn client(&self) -> Arc<Bot> {
        Arc::clone(&self.bot)
    }
}

impl Default for Server {
    fn default() -> Self {
        let api_token =
            std::env::var("VKTEAMS_BOT_API_TOKEN").expect("VKTEAMS_BOT_API_TOKEN is not set");
        let api_url = std::env::var("VKTEAMS_BOT_API_URL").expect("VKTEAMS_BOT_API_URL is not set");
        let chat_id = std::env::var("VKTEAMS_BOT_CHAT_ID").expect("VKTEAMS_BOT_CHAT_ID is not set");
        let bot = Bot::with_default_version(api_token, api_url).expect("Failed to create bot");
        Self {
            bot: Arc::new(bot),
            chat_id,
        }
    }
}
