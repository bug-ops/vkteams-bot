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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_default_and_client() {
        // Set required env vars for default
        unsafe {
            std::env::set_var("VKTEAMS_BOT_CHAT_ID", "test_chat_id");
            std::env::set_var("VKTEAMS_BOT_API_TOKEN", "dummy_token");
            std::env::set_var("VKTEAMS_BOT_API_URL", "https://dummy.api");
        }
        let server = Server::default();
        assert_eq!(server.chat_id, "test_chat_id");
        let bot = server.client();
        assert!(Arc::strong_count(&bot) >= 1);
    }
}
