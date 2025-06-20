use crate::storage::{DatabaseManager, models::{StoredEvent, StoredMessage}};
use vkteams_bot::prelude::{EventMessage, EventType, EventPayloadNewMessage};
use std::sync::Arc;

#[derive(Debug)]
pub struct EventProcessor {
    pub db: Arc<DatabaseManager>,
}

impl EventProcessor {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self { db }
    }

    pub async fn process_event(&self, event: &EventMessage, chat_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event_type_str = match &event.event_type {
            EventType::NewMessage(_) => "newMessage",
            EventType::EditedMessage(_) => "editedMessage", 
            EventType::DeleteMessage(_) => "deletedMessage",
            EventType::PinnedMessage(_) => "pinnedMessage",
            EventType::UnpinnedMessage(_) => "unpinnedMessage",
            EventType::NewChatMembers(_) => "newChatMembers",
            EventType::LeftChatMembers(_) => "leftChatMembers",
            EventType::CallbackQuery(_) => "callbackQuery",
            EventType::None => "none",
        };

        // Создаем запись события
        let stored_event = StoredEvent::new(
            event.event_id,
            chat_id.to_string(),
            event_type_str.to_string(),
            serde_json::to_string(event)?,
        );

        // Сохраняем событие в БД
        self.db.store_event(&stored_event).await?;

        // Обрабатываем только новые сообщения пока что
        if let EventType::NewMessage(payload) = &event.event_type {
            self.process_new_message_event(&stored_event, payload, chat_id).await?;
        }

        // Отмечаем событие как обработанное
        self.db.mark_event_as_processed(&stored_event.id).await?;

        log::info!("Processed event {} of type {} for chat {}", event.event_id, event_type_str, chat_id);
        Ok(())
    }

    async fn process_new_message_event(
        &self,
        stored_event: &StoredEvent,
        payload: &EventPayloadNewMessage,
        chat_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Создаем запись сообщения
        let stored_message = StoredMessage::new(
            stored_event.id.clone(),
            payload.msg_id.0.clone(),
            chat_id.to_string(),
            payload.from.user_id.0.clone(),
            payload.text.clone(),
            "text".to_string(),
        );

        // Сохраняем сообщение
        self.db.store_message(&stored_message).await?;

        log::info!("Processed message {} from user {} in chat {}", 
                   payload.msg_id.0, payload.from.user_id.0, chat_id);

        Ok(())
    }

    pub async fn get_latest_event_id(&self, chat_id: &str) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.db.get_latest_event_id(chat_id).await?.unwrap_or(0))
    }
}