use std::sync::Arc;
use vkteams_bot::prelude::*;
use crate::storage::DatabaseManager;
use crate::event_processor::EventProcessor;
use crate::config::Config;

#[derive(Debug, Clone)]
pub struct Server {
    pub bot: Arc<Bot>,
    pub chat_id: String,
    pub event_processor: Arc<EventProcessor>,
}

impl Server {
    pub async fn new() -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let config = Config::from_env()?;
        
        // Инициализируем базу данных
        let db = Arc::new(DatabaseManager::new(&config.database.url).await?);
        
        // Создаем процессор событий
        let event_processor = Arc::new(EventProcessor::new(db));
        
        Ok(Self {
            bot: Arc::new(Bot::default()),
            chat_id: config.bot.chat_id,
            event_processor,
        })
    }

    pub fn client(&self) -> Arc<Bot> {
        Arc::clone(&self.bot)
    }
}

impl Default for Server {
    fn default() -> Self {
        // Для обратной совместимости, но лучше использовать new()
        let chat_id = std::env::var("VKTEAMS_BOT_CHAT_ID").expect("VKTEAMS_BOT_CHAT_ID is not set");
        
        // Создаем упрощенную версию без базы данных для тестов
        let db = std::sync::Arc::new(
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    DatabaseManager::new("sqlite::memory:").await.expect("Failed to create in-memory database")
                })
            })
        );
        
        let event_processor = Arc::new(EventProcessor::new(db));
        
        Self {
            bot: Arc::new(Bot::default()),
            chat_id,
            event_processor,
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
