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
        let chat_id = std::env::var("VKTEAMS_BOT_CHAT_ID").expect("VKTEAMS_BOT_CHAT_ID is not set");
        Self {
            bot: Arc::new(Bot::default()),
            chat_id,
        }
    }
}
