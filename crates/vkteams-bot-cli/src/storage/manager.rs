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
        let new_event = NewEvent {
            event_id: event.event_id.clone(),
            event_type: event.event_type.clone(),
            chat_id: event.payload.chat.chat_id.clone(),
            user_id: event.payload.from.as_ref().map(|u| u.user_id.clone()),
            timestamp: Utc::now(), // TODO: parse from event if available
            raw_payload: serde_json::to_value(event)
                .map_err(|e| StorageError::Serialization(e))?,
            processed_data: None,
        };

        let event_id = self.relational.insert_event(new_event).await?;

        // If it's a message event, extract and store message details
        if event.event_type == "newMessage" {
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
        if event.event_type != "newMessage" {
            return Ok(None);
        }

        // TODO: Parse actual message data from event payload
        // This is a simplified version - you'll need to implement proper parsing
        let text = event.payload.text.clone();
        let user_id = event.payload.from.as_ref()
            .map(|u| u.user_id.clone())
            .ok_or_else(|| StorageError::InvalidInput("Missing user_id in message event".to_string()))?;

        Ok(Some(NewMessage {
            event_id,
            message_id: format!("msg_{}", event.event_id), // TODO: extract real message ID
            chat_id: event.payload.chat.chat_id.clone(),
            user_id,
            text,
            formatted_text: None, // TODO: extract formatted text if available
            reply_to_message_id: None, // TODO: extract reply info
            forward_from_chat_id: None,
            forward_from_message_id: None,
            file_attachments: None, // TODO: extract file attachments
            has_mentions: false, // TODO: detect mentions
            mentions: None,
            timestamp: Utc::now(), // TODO: parse from event
        }))
    }

    /// Extract text content for embedding generation
    fn extract_text_content(&self, event: &EventMessage) -> Option<String> {
        // Extract text from different event types
        match event.event_type.as_str() {
            "newMessage" => {
                event.payload.text.clone()
            }
            _ => None, // Add other event types as needed
        }
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