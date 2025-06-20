use crate::storage::{DatabaseManager, models::{StoredEvent, StoredMessage, StoredFile}};
use vkteams_bot::api::types::{EventMessage, EventType, EventPayloadNewMessage, EventPayloadEditedMessage, EventPayloadDeleteMessage};
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

        // Обрабатываем специфичные типы событий
        match &event.event_type {
            EventType::NewMessage(payload) => {
                self.process_new_message_event(&stored_event, &**payload, chat_id).await?;
            }
            EventType::EditedMessage(payload) => {
                self.process_edited_message_event(&**payload).await?;
            }
            EventType::DeleteMessage(payload) => {
                self.process_deleted_message_event(&**payload).await?;
            }
            EventType::NewChatMembers(_) => {
                log::info!("New chat members event processed for chat {}", chat_id);
            }
            EventType::LeftChatMembers(_) => {
                log::info!("Left chat members event processed for chat {}", chat_id);
            }
            _ => {
                log::debug!("Unhandled event type: {} for chat {}", event_type_str, chat_id);
            }
        }

        // Отмечаем событие как обработанное
        self.db.mark_event_as_processed(&stored_event.id).await?;

        Ok(())
    }

    async fn process_new_message_event(
        &self,
        stored_event: &StoredEvent,
        event: &Event,
        chat_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(payload) = &event.payload {
            // Извлекаем данные сообщения из payload
            let message_id = payload
                .get("msgId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let user_id = payload
                .get("from")
                .and_then(|from| from.get("userId"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let content = payload
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let message_type = payload
                .get("msgType")
                .and_then(|v| v.as_str())
                .unwrap_or("text")
                .to_string();

            // Создаем запись сообщения
            let mut stored_message = StoredMessage::new(
                stored_event.id.clone(),
                message_id.clone(),
                chat_id.to_string(),
                user_id,
                content,
                message_type.clone(),
            );

            // Проверяем на ответ на другое сообщение
            if let Some(reply_msg_id) = payload
                .get("replyMsgId")
                .and_then(|v| v.as_str())
            {
                stored_message.reply_to = Some(reply_msg_id.to_string());
            }

            // Проверяем на пересланное сообщение
            if let Some(forward_data) = payload.get("forwardedMessage") {
                if let Some(from_id) = forward_data
                    .get("from")
                    .and_then(|from| from.get("userId"))
                    .and_then(|v| v.as_str())
                {
                    stored_message.forwarded_from = Some(from_id.to_string());
                }
            }

            // Сохраняем сообщение
            self.db.store_message(&stored_message).await?;

            // Обрабатываем файлы, если есть
            if let Some(parts) = payload.get("parts").and_then(|v| v.as_array()) {
                for part in parts {
                    if let Some(part_type) = part.get("type").and_then(|v| v.as_str()) {
                        match part_type {
                            "file" | "image" | "video" | "voice" => {
                                self.process_file_attachment(part, &stored_message.id).await?;
                            }
                            _ => {
                                // Другие типы частей сообщения (sticker, mention, etc.)
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn process_edited_message_event(
        &self,
        event: &Event,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(payload) = &event.payload {
            if let Some(message_id) = payload
                .get("msgId")
                .and_then(|v| v.as_str())
            {
                self.db.update_message_as_edited(message_id).await?;
                log::info!("Message {} marked as edited", message_id);
            }
        }
        Ok(())
    }

    async fn process_deleted_message_event(
        &self,
        event: &Event,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(payload) = &event.payload {
            if let Some(message_id) = payload
                .get("msgId")
                .and_then(|v| v.as_str())
            {
                self.db.update_message_as_deleted(message_id).await?;
                log::info!("Message {} marked as deleted", message_id);
            }
        }
        Ok(())
    }

    async fn process_file_attachment(
        &self,
        file_part: &serde_json::Value,
        message_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let file_id = file_part
            .get("fileId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let filename = file_part
            .get("caption")
            .and_then(|v| v.as_str())
            .or_else(|| file_part.get("name").and_then(|v| v.as_str()))
            .unwrap_or("unknown")
            .to_string();

        let file_type = file_part
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("file")
            .to_string();

        let size = file_part
            .get("size")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let url = file_part
            .get("url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let stored_file = StoredFile {
            id: uuid::Uuid::new_v4().to_string(),
            message_id: message_id.to_string(),
            file_id,
            filename,
            file_type,
            size,
            url,
            created_at: chrono::Utc::now(),
        };

        self.db.store_file(&stored_file).await?;
        log::info!("File attachment {} processed for message {}", stored_file.file_id, message_id);

        Ok(())
    }

    pub async fn get_latest_event_id(&self, chat_id: &str) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.db.get_latest_event_id(chat_id).await?.unwrap_or(0))
    }
}