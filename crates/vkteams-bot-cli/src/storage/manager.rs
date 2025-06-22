//! Storage manager that coordinates relational and vector storage

use crate::config::Config;
use crate::storage::{StorageError, StorageResult};
use chrono::{DateTime, Utc};
use serde_json::Value;
use vkteams_bot::prelude::EventMessage;

#[cfg(feature = "database")]
use crate::storage::relational::RelationalStore;
#[cfg(feature = "database")]
use crate::storage::models::*;

#[cfg(feature = "vector-search")]
use crate::storage::vector::{VectorStore, VectorDocument, SearchQuery, SearchResult, create_vector_store};

#[cfg(feature = "ai-embeddings")]
use crate::storage::embedding::EmbeddingClient;

/// Main storage manager coordinating all storage backends
pub struct StorageManager {
    #[cfg(feature = "database")]
    relational: RelationalStore,
    
    #[cfg(feature = "vector-search")]
    vector: Box<dyn VectorStore>,
    
    #[cfg(feature = "ai-embeddings")]
    embedding: EmbeddingClient,
}

impl StorageManager {
    /// Create new storage manager from configuration
    pub async fn new(config: &Config) -> StorageResult<Self> {
        #[cfg(feature = "database")]
        let relational = {
            let db_config = config.database.as_ref()
                .ok_or_else(|| StorageError::Configuration("Database configuration missing".to_string()))?;
            RelationalStore::new(&db_config.url).await?
        };

        #[cfg(feature = "vector-search")]
        let vector = {
            let vector_config = config.vector.as_ref()
                .ok_or_else(|| StorageError::Configuration("Vector configuration missing".to_string()))?;
            create_vector_store(
                &vector_config.provider,
                &vector_config.connection_url,
                vector_config.collection_name.clone(),
            ).await?
        };

        #[cfg(feature = "ai-embeddings")]
        let embedding = {
            let embedding_config = config.embedding.as_ref()
                .ok_or_else(|| StorageError::Configuration("Embedding configuration missing".to_string()))?;
            EmbeddingClient::new(embedding_config).await?
        };

        Ok(Self {
            #[cfg(feature = "database")]
            relational,
            #[cfg(feature = "vector-search")]
            vector,
            #[cfg(feature = "ai-embeddings")]
            embedding,
        })
    }

    /// Initialize database and run migrations
    pub async fn initialize(&self) -> StorageResult<()> {
        #[cfg(feature = "database")]
        {
            self.relational.migrate().await?;
        }
        Ok(())
    }

    /// Process a VK Teams event and store it
    #[cfg(feature = "database")]
    pub async fn process_event(&self, event: &EventMessage) -> StorageResult<i64> {
        use vkteams_bot::api::types::EventType;
        
        let new_event = NewEvent {
            event_id: event.event_id.to_string(),
            event_type: self.event_type_to_string(&event.event_type),
            chat_id: self.extract_chat_id(event).unwrap_or_default(),
            user_id: self.extract_user_id(event),
            timestamp: self.extract_timestamp(event).unwrap_or_else(|| Utc::now()),
            raw_payload: serde_json::to_value(event)
                .map_err(|e| StorageError::Serialization(e))?,
            processed_data: None,
        };

        let event_id = self.relational.insert_event(new_event).await?;

        // If it's a message event, extract and store message details
        if matches!(event.event_type, EventType::NewMessage(_)) {
            if let Some(message_data) = self.extract_message_data(event, event_id)? {
                self.relational.insert_message(message_data).await?;
            }
        }

        Ok(event_id)
    }

    /// Process event with vector embedding generation
    #[cfg(all(feature = "database", feature = "vector-search", feature = "ai-embeddings"))]
    pub async fn process_event_with_embeddings(&self, event: &EventMessage) -> StorageResult<i64> {
        // Store in relational database first
        let event_id = self.process_event(event).await?;

        // Extract text content for embedding
        if let Some(text_content) = self.extract_text_content(event) {
            // Generate embedding
            let embedding = self.embedding.generate_embedding(&text_content).await
                .map_err(|e| StorageError::Embedding(e.to_string()))?;

            // Store in vector database
            let vector_doc = VectorDocument {
                id: format!("event_{}", event_id),
                content: text_content,
                metadata: serde_json::json!({
                    "event_id": event_id,
                    "chat_id": event.payload.chat.chat_id,
                    "event_type": event.event_type,
                    "timestamp": Utc::now()
                }),
                embedding: pgvector::Vector::from(embedding),
                created_at: Utc::now(),
            };

            self.vector.store_document(vector_doc).await?;
        }

        Ok(event_id)
    }

    /// Search for similar events using vector similarity
    #[cfg(all(feature = "vector-search", feature = "ai-embeddings"))]
    pub async fn search_similar_events(
        &self,
        query_text: &str,
        chat_id: Option<&str>,
        limit: usize,
    ) -> StorageResult<Vec<SearchResult>> {
        // Generate embedding for query
        let query_embedding = self.embedding.generate_embedding(query_text).await
            .map_err(|e| StorageError::Embedding(e.to_string()))?;

        let mut metadata_filter = serde_json::json!({});
        if let Some(chat_id) = chat_id {
            metadata_filter["chat_id"] = Value::String(chat_id.to_string());
        }

        let search_query = SearchQuery {
            embedding: pgvector::Vector::from(query_embedding),
            limit,
            score_threshold: Some(0.7), // Minimum similarity threshold
            metadata_filter: Some(metadata_filter),
            include_content: true,
        };

        self.vector.search_similar(search_query).await
    }

    /// Get recent events from relational database
    #[cfg(feature = "database")]
    pub async fn get_recent_events(
        &self,
        chat_id: Option<&str>,
        since: Option<DateTime<Utc>>,
        limit: i64,
    ) -> StorageResult<Vec<Event>> {
        self.relational.get_recent_events(chat_id, since, limit).await
    }

    /// Search messages using full-text search
    #[cfg(feature = "database")]
    pub async fn search_messages(
        &self,
        query: &str,
        chat_id: Option<&str>,
        limit: i64,
    ) -> StorageResult<Vec<Message>> {
        self.relational.search_messages(query, chat_id, limit).await
    }

    /// Get storage statistics
    #[cfg(feature = "database")]
    pub async fn get_stats(&self, chat_id: Option<&str>) -> StorageResult<EventStats> {
        self.relational.get_stats(chat_id).await
    }

    /// Clean up old data
    #[cfg(feature = "database")]
    pub async fn cleanup_old_data(&self, older_than_days: u32) -> StorageResult<u64> {
        let deleted_events = self.relational.cleanup_old_events(older_than_days).await?;

        #[cfg(feature = "vector-search")]
        {
            let older_than = Utc::now() - chrono::Duration::days(older_than_days as i64);
            let _deleted_vectors = self.vector.cleanup_old_documents(older_than).await?;
        }

        Ok(deleted_events)
    }

    /// Health check for all storage backends
    pub async fn health_check(&self) -> StorageResult<()> {
        #[cfg(feature = "database")]
        self.relational.health_check().await?;

        #[cfg(feature = "vector-search")]
        self.vector.health_check().await?;

        Ok(())
    }

    /// Extract message data from event
    #[cfg(feature = "database")]
    fn extract_message_data(&self, event: &EventMessage, event_id: i64) -> StorageResult<Option<NewMessage>> {
        use vkteams_bot::api::types::EventType;
        
        // Extract message data based on event type
        match &event.event_type {
            EventType::NewMessage(payload) => {
                let text = payload.text.clone();
                let user_id = payload.from.user_id.clone();
                let message_id = payload.msg_id.clone();
                let chat_id = payload.chat.chat_id.clone();
                
                // Extract formatted text from parts if available
                let formatted_text = if !payload.parts.is_empty() {
                    Some(self.extract_formatted_text(&payload.parts))
                } else if payload.format.is_some() {
                    Some(text.clone()) // Use original text if format is specified
                } else {
                    None
                };
                
                // Extract file attachments
                let file_attachments = self.extract_file_attachments(&payload.parts);
                
                // Detect mentions
                let (has_mentions, mentions) = self.extract_mentions(&text, &payload.parts);
                
                // Parse timestamp from event
                let timestamp = chrono::DateTime::from_timestamp(payload.timestamp.0 as i64, 0)
                    .unwrap_or_else(|| Utc::now());

                Ok(Some(NewMessage {
                    event_id,
                    message_id: message_id.0.to_string(),
                    chat_id: chat_id.0.to_string(),
                    user_id: user_id.0.to_string(),
                    text,
                    formatted_text,
                    reply_to_message_id: None, // TODO: extract reply info from parts
                    forward_from_chat_id: None, // TODO: extract forward info
                    forward_from_message_id: None, // TODO: extract forward info
                    file_attachments,
                    has_mentions,
                    mentions,
                    timestamp,
                }))
            }
            _ => Ok(None), // Other event types don't contain message data
        }
    }

    /// Extract text content for embedding generation
    fn extract_text_content(&self, event: &EventMessage) -> Option<String> {
        use vkteams_bot::api::types::EventType;
        
        match &event.event_type {
            EventType::NewMessage(payload) => {
                if payload.text.is_empty() {
                    None
                } else {
                    Some(payload.text.clone())
                }
            }
            EventType::EditedMessage(payload) => {
                if payload.text.is_empty() {
                    None
                } else {
                    Some(payload.text.clone())
                }
            }
            _ => None, // Other event types don't have text content
        }
    }
    
    /// Extract formatted text from message parts
    fn extract_formatted_text(&self, parts: &[vkteams_bot::api::types::MessageParts]) -> String {
        use vkteams_bot::api::types::MessageParts;
        
        parts.iter().map(|part| {
            match part {
                MessageParts::Text { text } => text.clone(),
                MessageParts::Mention { text, .. } => text.clone(),
                MessageParts::Link { text, .. } => text.clone(),
                MessageParts::Bold { text } => format!("**{}**", text),
                MessageParts::Italic { text } => format!("*{}*", text),
                MessageParts::Underline { text } => format!("__{}", text),
                MessageParts::Strikethrough { text } => format!("~~{}~~", text),
                MessageParts::Code { text } => format!("`{}`", text),
                MessageParts::Pre { text, .. } => format!("```\n{}\n```", text),
                MessageParts::OrderedList { text } => text.clone(),
                MessageParts::UnorderedList { text } => text.clone(),
                MessageParts::Quote { text } => format!("> {}", text),
                _ => String::new(), // Handle other types as needed
            }
        }).collect::<Vec<_>>().join("")
    }
    
    /// Extract file attachments from message parts
    fn extract_file_attachments(&self, parts: &[vkteams_bot::api::types::MessageParts]) -> Option<Value> {
        use vkteams_bot::api::types::MessageParts;
        
        let files: Vec<Value> = parts.iter().filter_map(|part| {
            match part {
                MessageParts::File { file_id, filename, .. } => {
                    Some(serde_json::json!({
                        "file_id": file_id,
                        "filename": filename,
                        "type": "file"
                    }))
                }
                MessageParts::Image { file_id, .. } => {
                    Some(serde_json::json!({
                        "file_id": file_id,
                        "type": "image"
                    }))
                }
                MessageParts::Video { file_id, .. } => {
                    Some(serde_json::json!({
                        "file_id": file_id,
                        "type": "video"
                    }))
                }
                MessageParts::Audio { file_id, .. } => {
                    Some(serde_json::json!({
                        "file_id": file_id,
                        "type": "audio"
                    }))
                }
                MessageParts::Voice { file_id, .. } => {
                    Some(serde_json::json!({
                        "file_id": file_id,
                        "type": "voice"
                    }))
                }
                MessageParts::Sticker { file_id, .. } => {
                    Some(serde_json::json!({
                        "file_id": file_id,
                        "type": "sticker"
                    }))
                }
                _ => None,
            }
        }).collect();
        
        if files.is_empty() {
            None
        } else {
            Some(Value::Array(files))
        }
    }
    
    /// Extract mentions from text and message parts
    fn extract_mentions(&self, text: &str, parts: &[vkteams_bot::api::types::MessageParts]) -> (bool, Option<Value>) {
        use vkteams_bot::api::types::MessageParts;
        
        let mentions: Vec<Value> = parts.iter().filter_map(|part| {
            match part {
                MessageParts::Mention { user_id, text } => {
                    Some(serde_json::json!({
                        "user_id": user_id,
                        "text": text
                    }))
                }
                _ => None,
            }
        }).collect();
        
        // Also check for @mentions in text (simple regex check)
        let text_mentions = text.contains('@');
        let has_mentions = !mentions.is_empty() || text_mentions;
        
        let mentions_data = if mentions.is_empty() {
            None
        } else {
            Some(Value::Array(mentions))
        };
        
        (has_mentions, mentions_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_text_content() {
        let manager = StorageManager {
            #[cfg(feature = "database")]
            relational: unimplemented!(),
            #[cfg(feature = "vector-search")]
            vector: unimplemented!(),
            #[cfg(feature = "ai-embeddings")]
            embedding: unimplemented!(),
        };

        // Mock event message
        let mut event = EventMessage {
            event_id: "test_123".to_string(),
            event_type: "newMessage".to_string(),
            payload: Default::default(),
        };
        event.payload.text = Some("Hello, world!".to_string());

        let text = manager.extract_text_content(&event);
        assert_eq!(text, Some("Hello, world!".to_string()));
    }
}