//! Storage manager that coordinates all storage backends

use crate::api::types::EventMessage;
use crate::storage::{StorageConfig, StorageError, StorageResult};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::sync::Arc;

#[cfg(feature = "storage")]
use crate::storage::models::*;
#[cfg(feature = "storage")]
use crate::storage::simple::SimpleRelationalStore;

#[cfg(feature = "vector-search")]
use crate::storage::vector::{
    SearchQuery, SearchResult, VectorDocument, VectorStore, create_vector_store_with_ssl,
};

#[cfg(feature = "ai-embeddings")]
use crate::storage::embedding::EmbeddingClient;

/// Main storage manager coordinating all storage backends
pub struct StorageManager {
    #[cfg(feature = "storage")]
    relational: Arc<SimpleRelationalStore>,

    #[cfg(feature = "vector-search")]
    vector: Option<Arc<Box<dyn VectorStore>>>,

    #[cfg(feature = "ai-embeddings")]
    embedding: Option<Arc<Box<dyn EmbeddingClient>>>,

    config: Arc<StorageConfig>,
}

impl StorageManager {
    /// Initialize storage (run migrations, etc.)
    #[cfg(feature = "storage")]
    pub async fn initialize(&self) -> StorageResult<()> {
        self.relational.initialize().await
    }

    /// Get embedding dimensions from configuration
    #[cfg(feature = "vector-search")]
    pub fn get_embedding_dimensions(&self) -> usize {
        if let Some(ref vector_config) = self.config.vector {
            vector_config.dimensions
        } else {
            1536 // default dimensions
        }
    }
    /// Create new storage manager from configuration
    pub async fn new(config: &StorageConfig) -> StorageResult<Self> {
        #[cfg(feature = "storage")]
        let relational = {
            let store = SimpleRelationalStore::new(&config.database.url).await?;
            if config.database.auto_migrate {
                store.migrate().await?;
            }
            Arc::new(store)
        };

        #[cfg(feature = "vector-search")]
        let vector = {
            if let Some(vector_config) = &config.vector {
                let store = create_vector_store_with_ssl(
                    &vector_config.provider,
                    &vector_config.connection_url,
                    Some(vector_config.collection_name.clone()),
                    vector_config.dimensions,
                    vector_config.ivfflat_lists,
                    &config.database.ssl,
                )
                .await?;
                Some(Arc::new(store))
            } else {
                None
            }
        };

        #[cfg(feature = "ai-embeddings")]
        let embedding = {
            if let Some(embedding_config) = &config.embedding {
                use crate::storage::embedding::EmbeddingProviderConfig;

                let provider_config = match embedding_config.provider.as_str() {
                    "openai" => {
                        let api_key =
                            std::env::var(&embedding_config.api_key_env).map_err(|_| {
                                StorageError::Configuration(format!(
                                    "Environment variable {} not found",
                                    embedding_config.api_key_env
                                ))
                            })?;
                        EmbeddingProviderConfig::OpenAI {
                            api_key,
                            model: embedding_config.model.clone(),
                        }
                    }
                    "ollama" => EmbeddingProviderConfig::Ollama {
                        host: embedding_config.ollama_host.clone(),
                        port: embedding_config.ollama_port,
                        model: embedding_config.model.clone(),
                        dimensions: embedding_config.custom_dimensions,
                    },
                    _ => {
                        return Err(StorageError::Configuration(format!(
                            "Unknown embedding provider: {}",
                            embedding_config.provider
                        )));
                    }
                };

                let client =
                    crate::storage::embedding::create_embedding_client(provider_config).await?;
                Some(Arc::new(client))
            } else {
                None
            }
        };

        Ok(Self {
            #[cfg(feature = "storage")]
            relational,
            #[cfg(feature = "vector-search")]
            vector,
            #[cfg(feature = "ai-embeddings")]
            embedding,
            config: Arc::new(config.clone()),
        })
    }

    /// Process a VK Teams event and store it
    #[cfg(feature = "storage")]
    pub async fn process_event(&self, event: &EventMessage) -> StorageResult<i64> {
        // Create event record
        let new_event = NewEvent {
            event_id: event.event_id.to_string(),
            event_type: self.event_type_to_string(&event.event_type),
            chat_id: self.extract_chat_id(event).unwrap_or_default(),
            user_id: self.extract_user_id(event),
            timestamp: self.extract_timestamp(event).unwrap_or_else(Utc::now),
            raw_payload: serde_json::to_value(event).map_err(StorageError::Serialization)?,
            processed_data: None,
        };

        // Store event in relational database
        let event_id = self.relational.store_event(new_event).await?;

        // Store message data if this is a message event
        if let Some(message_data) = self.extract_message_data(event, event_id)? {
            self.relational.store_message(message_data).await?;
        }

        Ok(event_id)
    }

    /// Process event with vector embedding generation
    #[cfg(all(
        feature = "storage",
        feature = "vector-search",
        feature = "ai-embeddings"
    ))]
    pub async fn process_event_with_embeddings(&self, event: &EventMessage) -> StorageResult<i64> {
        // Store in relational database first
        let event_id = self.process_event(event).await?;

        // Extract text content for embedding
        if let Some(text_content) = self.extract_text_content(event) {
            // Generate embedding if embedding client is available
            if let Some(embedding_client) = &self.embedding {
                let embedding = embedding_client
                    .generate_embedding(&text_content)
                    .await
                    .map_err(|e| StorageError::Embedding(e.to_string()))?;

                // Store in vector database if vector store is available
                if let Some(vector_store) = &self.vector {
                    let event_timestamp = self.extract_timestamp(event).unwrap_or_else(Utc::now);
                    let vector_doc = VectorDocument {
                        id: format!("event_{event_id}"),
                        content: text_content,
                        metadata: serde_json::json!({
                            "event_id": event_id,
                            "event_type": format!("{:?}", event.event_type),
                            "timestamp": event_timestamp
                        }),
                        embedding: pgvector::Vector::from(embedding),
                        created_at: event_timestamp,
                    };

                    vector_store.store_document(vector_doc).await?;
                }
            }
        }

        Ok(event_id)
    }

    /// Batch process multiple events
    #[cfg(feature = "storage")]
    pub async fn process_events_batch(&self, events: &[EventMessage]) -> StorageResult<Vec<i64>> {
        let mut event_ids = Vec::new();

        for event in events {
            let event_id = self.process_event(event).await?;
            event_ids.push(event_id);
        }

        // Generate embeddings in batch if enabled
        #[cfg(all(feature = "vector-search", feature = "ai-embeddings"))]
        {
            if let (Some(embedding_client), Some(vector_store)) = (&self.embedding, &self.vector) {
                let texts: Vec<String> = events
                    .iter()
                    .filter_map(|e| self.extract_text_content(e))
                    .collect();

                if !texts.is_empty() {
                    let embeddings = embedding_client
                        .generate_embeddings(&texts)
                        .await
                        .map_err(|e| StorageError::Embedding(e.to_string()))?;

                    let vector_docs: Vec<VectorDocument> = event_ids
                        .iter()
                        .zip(events.iter())
                        .zip(embeddings.into_iter())
                        .filter_map(|((event_id, event), embedding)| {
                            self.extract_text_content(event).map(|text| {
                                let event_timestamp =
                                    self.extract_timestamp(event).unwrap_or_else(Utc::now);
                                VectorDocument {
                                    id: format!("event_{event_id}"),
                                    content: text,
                                    metadata: serde_json::json!({
                                        "event_id": event_id,
                                        "event_type": format!("{:?}", event.event_type),
                                        "timestamp": event_timestamp
                                    }),
                                    embedding: pgvector::Vector::from(embedding),
                                    created_at: event_timestamp,
                                }
                            })
                        })
                        .collect();

                    vector_store.store_documents(vector_docs).await?;
                }
            }
        }

        Ok(event_ids)
    }

    /// Search for similar events using vector similarity
    #[cfg(all(feature = "vector-search", feature = "ai-embeddings"))]
    pub async fn search_similar_events(
        &self,
        query_text: &str,
        chat_id: Option<&str>,
        limit: usize,
    ) -> StorageResult<Vec<SearchResult>> {
        match (&self.embedding, &self.vector) {
            (Some(embedding_client), Some(vector_store)) => {
                // Generate embedding for query
                let query_embedding = embedding_client
                    .generate_embedding(query_text)
                    .await
                    .map_err(|e| StorageError::Embedding(e.to_string()))?;

                let mut metadata_filter = serde_json::json!({});
                if let Some(chat_id) = chat_id {
                    metadata_filter["chat_id"] = Value::String(chat_id.to_string());
                }

                let search_query = SearchQuery {
                    embedding: pgvector::Vector::from(query_embedding),
                    limit,
                    score_threshold: Some(
                        self.config
                            .vector
                            .as_ref()
                            .map(|v| v.similarity_threshold)
                            .unwrap_or(0.7),
                    ),
                    metadata_filter: Some(metadata_filter),
                    include_content: true,
                };

                vector_store.search_similar(search_query).await
            }
            _ => Err(StorageError::Configuration(
                "Vector search or embedding client not available".to_string(),
            )),
        }
    }

    /// Get recent events from relational database
    #[cfg(feature = "storage")]
    pub async fn get_recent_events(
        &self,
        chat_id: Option<&str>,
        _since: Option<DateTime<Utc>>,
        limit: i64,
    ) -> StorageResult<Vec<Event>> {
        self.relational
            .get_recent_events(chat_id, None, limit)
            .await
    }

    /// Search messages using full-text search
    #[cfg(feature = "storage")]
    pub async fn search_messages(
        &self,
        query: &str,
        chat_id: Option<&str>,
        limit: i64,
    ) -> StorageResult<Vec<Message>> {
        self.relational.search_messages(query, chat_id, limit).await
    }

    /// Advanced search with multiple filters
    #[cfg(feature = "storage")]
    pub async fn search_events_advanced(
        &self,
        user_id: Option<&str>,
        event_type: Option<&str>,
        since: Option<chrono::DateTime<chrono::Utc>>,
        until: Option<chrono::DateTime<chrono::Utc>>,
        limit: i64,
    ) -> StorageResult<Vec<Event>> {
        self.relational
            .search_events_advanced(user_id, event_type, since, until, limit)
            .await
    }

    /// Store a vector document (for contexts, summaries, etc.)
    #[cfg(feature = "vector-search")]
    pub async fn store_vector_document(&self, document: &VectorDocument) -> StorageResult<()> {
        if let Some(vector_store) = &self.vector {
            vector_store.store_document(document.clone()).await?;
            Ok(())
        } else {
            Err(StorageError::Vector(
                "Vector store not configured".to_string(),
            ))
        }
    }

    /// Get storage statistics
    #[cfg(feature = "storage")]
    pub async fn get_stats(&self, chat_id: Option<&str>) -> StorageResult<EventStats> {
        self.relational.get_stats(chat_id).await
    }

    /// Clean up old data across all storage backends
    #[cfg(feature = "storage")]
    pub async fn cleanup_old_data(&self, older_than_days: u32) -> StorageResult<u64> {
        let deleted_events = self.relational.cleanup_old_data(older_than_days).await?;

        #[cfg(feature = "vector-search")]
        {
            if let Some(vector_store) = &self.vector {
                let older_than = Utc::now() - chrono::Duration::days(older_than_days as i64);
                let _deleted_vectors = vector_store.cleanup_old_documents(older_than).await?;
            }
        }

        Ok(deleted_events as u64)
    }

    /// Health check for all storage backends
    pub async fn health_check(&self) -> StorageResult<()> {
        #[cfg(feature = "storage")]
        self.relational.health_check().await?;

        #[cfg(feature = "vector-search")]
        {
            if let Some(vector_store) = &self.vector {
                vector_store.health_check().await?;
            }
        }

        #[cfg(feature = "ai-embeddings")]
        {
            if let Some(embedding_client) = &self.embedding {
                embedding_client.health_check().await?;
            }
        }

        Ok(())
    }

    /// Get vector store performance metrics
    #[cfg(feature = "vector-search")]
    pub async fn get_vector_metrics(
        &self,
    ) -> StorageResult<Option<crate::storage::vector::VectorMetrics>> {
        if let Some(vector_store) = &self.vector {
            Ok(Some(vector_store.get_metrics().await?))
        } else {
            Ok(None)
        }
    }

    /// Perform vector store maintenance
    #[cfg(feature = "vector-search")]
    pub async fn perform_vector_maintenance(&self) -> StorageResult<()> {
        if let Some(vector_store) = &self.vector {
            vector_store.perform_maintenance().await?;
        }
        Ok(())
    }

    /// Get configuration
    pub fn config(&self) -> &StorageConfig {
        &self.config
    }

    /// Extract message data from event (private helper)
    fn extract_message_data(
        &self,
        event: &EventMessage,
        event_id: i64,
    ) -> StorageResult<Option<NewMessage>> {
        match &event.event_type {
            crate::api::types::EventType::NewMessage(payload) => {
                let user_id = payload.from.user_id.clone();
                let chat_id = payload.chat.chat_id.clone();

                // Detect mentions in text
                let (has_mentions, mentions) = self.detect_mentions(&payload.text);

                // Extract file attachments from parts
                let file_attachments = self.extract_file_attachments(&payload.parts);

                Ok(Some(NewMessage {
                    event_id,
                    message_id: payload.msg_id.0.clone(),
                    chat_id: chat_id.0.to_string(),
                    user_id: user_id.0.to_string(),
                    text: Some(payload.text.clone()).filter(|t| !t.is_empty()),
                    formatted_text: self.extract_formatted_text(payload),
                    reply_to_message_id: None, // VK Teams API doesn't provide this in standard payload
                    forward_from_chat_id: None,
                    forward_from_message_id: None,
                    file_attachments,
                    has_mentions,
                    mentions,
                    timestamp: DateTime::from_timestamp(payload.timestamp.0 as i64, 0)
                        .unwrap_or_else(Utc::now),
                }))
            }
            crate::api::types::EventType::EditedMessage(payload) => {
                // For edited messages, we can update the existing message
                let user_id = payload.from.user_id.clone();
                let chat_id = payload.chat.chat_id.clone();

                let (has_mentions, mentions) = self.detect_mentions(&payload.text);
                // Note: EventPayloadEditedMessage doesn't have parts field
                let file_attachments = None;

                Ok(Some(NewMessage {
                    event_id,
                    message_id: payload.msg_id.0.clone(),
                    chat_id: chat_id.0.to_string(),
                    user_id: user_id.0.to_string(),
                    text: Some(payload.text.clone()).filter(|t| !t.is_empty()),
                    formatted_text: self.extract_formatted_text_edited(payload),
                    reply_to_message_id: None,
                    forward_from_chat_id: None,
                    forward_from_message_id: None,
                    file_attachments,
                    has_mentions,
                    mentions,
                    timestamp: DateTime::from_timestamp(payload.timestamp.0 as i64, 0)
                        .unwrap_or_else(Utc::now),
                }))
            }
            _ => Ok(None), // Other event types don't contain message data
        }
    }

    /// Extract text content for embedding generation (private helper)
    fn extract_text_content(&self, event: &EventMessage) -> Option<String> {
        match &event.event_type {
            crate::api::types::EventType::NewMessage(payload) => {
                if payload.text.is_empty() {
                    None
                } else {
                    Some(payload.text.clone())
                }
            }
            crate::api::types::EventType::EditedMessage(payload) => {
                if payload.text.is_empty() {
                    None
                } else {
                    Some(payload.text.clone())
                }
            }
            _ => None,
        }
    }

    /// Convert EventType to string representation
    fn event_type_to_string(&self, event_type: &crate::api::types::EventType) -> String {
        match event_type {
            crate::api::types::EventType::NewMessage(_) => "newMessage".to_string(),
            crate::api::types::EventType::EditedMessage(_) => "editedMessage".to_string(),
            crate::api::types::EventType::DeleteMessage(_) => "deleteMessage".to_string(),
            crate::api::types::EventType::PinnedMessage(_) => "pinnedMessage".to_string(),
            crate::api::types::EventType::UnpinnedMessage(_) => "unpinnedMessage".to_string(),
            crate::api::types::EventType::NewChatMembers(_) => "newChatMembers".to_string(),
            crate::api::types::EventType::LeftChatMembers(_) => "leftChatMembers".to_string(),
            crate::api::types::EventType::CallbackQuery(_) => "callbackQuery".to_string(),
            crate::api::types::EventType::None => "none".to_string(),
        }
    }

    /// Extract chat ID from event
    fn extract_chat_id(&self, event: &EventMessage) -> Option<String> {
        match &event.event_type {
            crate::api::types::EventType::NewMessage(payload) => {
                Some(payload.chat.chat_id.0.to_string())
            }
            crate::api::types::EventType::EditedMessage(payload) => {
                Some(payload.chat.chat_id.0.to_string())
            }
            crate::api::types::EventType::DeleteMessage(payload) => {
                Some(payload.chat.chat_id.0.to_string())
            }
            crate::api::types::EventType::PinnedMessage(payload) => {
                Some(payload.chat.chat_id.0.to_string())
            }
            crate::api::types::EventType::UnpinnedMessage(payload) => {
                Some(payload.chat.chat_id.0.to_string())
            }
            crate::api::types::EventType::NewChatMembers(payload) => {
                Some(payload.chat.chat_id.0.to_string())
            }
            crate::api::types::EventType::LeftChatMembers(payload) => {
                Some(payload.chat.chat_id.0.to_string())
            }
            crate::api::types::EventType::CallbackQuery(payload) => {
                Some(payload.message.chat.chat_id.0.to_string())
            }
            _ => None,
        }
    }

    /// Extract user ID from event
    fn extract_user_id(&self, event: &EventMessage) -> Option<String> {
        match &event.event_type {
            crate::api::types::EventType::NewMessage(payload) => {
                Some(payload.from.user_id.0.to_string())
            }
            crate::api::types::EventType::EditedMessage(payload) => {
                Some(payload.from.user_id.0.to_string())
            }
            crate::api::types::EventType::CallbackQuery(payload) => {
                Some(payload.from.user_id.0.to_string())
            }
            _ => None,
        }
    }

    /// Extract timestamp from event
    fn extract_timestamp(&self, event: &EventMessage) -> Option<DateTime<Utc>> {
        match &event.event_type {
            crate::api::types::EventType::NewMessage(payload) => {
                DateTime::from_timestamp(payload.timestamp.0 as i64, 0)
            }
            crate::api::types::EventType::EditedMessage(payload) => {
                DateTime::from_timestamp(payload.timestamp.0 as i64, 0)
            }
            crate::api::types::EventType::DeleteMessage(payload) => {
                DateTime::from_timestamp(payload.timestamp.0 as i64, 0)
            }
            crate::api::types::EventType::PinnedMessage(payload) => {
                DateTime::from_timestamp(payload.timestamp.0 as i64, 0)
            }
            crate::api::types::EventType::UnpinnedMessage(payload) => {
                DateTime::from_timestamp(payload.timestamp.0 as i64, 0)
            }
            // NewChatMembers and LeftChatMembers don't have timestamp field,
            // use fallback to current time
            crate::api::types::EventType::NewChatMembers(_) => None,
            crate::api::types::EventType::LeftChatMembers(_) => None,
            crate::api::types::EventType::CallbackQuery(payload) => {
                // CallbackQuery uses message timestamp
                DateTime::from_timestamp(payload.message.timestamp.0 as i64, 0)
            }
            crate::api::types::EventType::None => None,
        }
    }

    /// Detect mentions in text
    fn detect_mentions(&self, text: &str) -> (bool, Option<Value>) {
        // Simple regex-based mention detection
        let mention_regex = regex::Regex::new(r"@(\w+)").unwrap();
        let mentions: Vec<String> = mention_regex
            .captures_iter(text)
            .map(|cap| cap[1].to_string())
            .collect();

        let has_mentions = !mentions.is_empty();
        let mentions_json = if has_mentions {
            Some(serde_json::to_value(mentions).unwrap_or(Value::Null))
        } else {
            None
        };

        (has_mentions, mentions_json)
    }

    /// Extract file attachments from message parts
    fn extract_file_attachments(&self, parts: &[crate::api::types::MessageParts]) -> Option<Value> {
        let mut attachments = Vec::new();

        for part in parts {
            match &part.part_type {
                crate::api::types::MessagePartsType::File(file_payload) => {
                    attachments.push(serde_json::json!({
                        "type": "file",
                        "file_id": file_payload.file_id,
                        "filename": file_payload.caption
                    }));
                }
                crate::api::types::MessagePartsType::Sticker(sticker_payload) => {
                    attachments.push(serde_json::json!({
                        "type": "sticker",
                        "file_id": sticker_payload.file_id
                    }));
                }
                crate::api::types::MessagePartsType::Voice(voice_payload) => {
                    attachments.push(serde_json::json!({
                        "type": "voice",
                        "file_id": voice_payload.file_id
                    }));
                }
                _ => {} // Other part types don't contain file attachments
            }
        }

        if attachments.is_empty() {
            None
        } else {
            Some(serde_json::to_value(attachments).unwrap_or(Value::Null))
        }
    }

    /// Extract formatted text from message payload
    fn extract_formatted_text(
        &self,
        payload: &crate::api::types::EventPayloadNewMessage,
    ) -> Option<String> {
        if let Some(format) = &payload.format {
            // Convert MessageFormat to formatted text representation
            serde_json::to_string(format).ok()
        } else {
            None
        }
    }

    /// Extract formatted text from edited message payload
    fn extract_formatted_text_edited(
        &self,
        payload: &crate::api::types::EventPayloadEditedMessage,
    ) -> Option<String> {
        if let Some(format) = &payload.format {
            // Convert MessageFormat to formatted text representation
            serde_json::to_string(format).ok()
        } else {
            None
        }
    }
}

// Implement Debug manually to avoid issues with trait objects
impl std::fmt::Debug for StorageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageManager")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::*;

    // Helper struct to test extraction methods without requiring database connection
    struct MockStorageManager;

    impl MockStorageManager {
        fn extract_timestamp(&self, event: &EventMessage) -> Option<DateTime<Utc>> {
            match &event.event_type {
                crate::api::types::EventType::NewMessage(payload) => {
                    DateTime::from_timestamp(payload.timestamp.0 as i64, 0)
                }
                crate::api::types::EventType::EditedMessage(payload) => {
                    DateTime::from_timestamp(payload.timestamp.0 as i64, 0)
                }
                crate::api::types::EventType::DeleteMessage(payload) => {
                    DateTime::from_timestamp(payload.timestamp.0 as i64, 0)
                }
                crate::api::types::EventType::PinnedMessage(payload) => {
                    DateTime::from_timestamp(payload.timestamp.0 as i64, 0)
                }
                crate::api::types::EventType::UnpinnedMessage(payload) => {
                    DateTime::from_timestamp(payload.timestamp.0 as i64, 0)
                }
                // NewChatMembers and LeftChatMembers don't have timestamp field,
                // use fallback to current time
                crate::api::types::EventType::NewChatMembers(_) => None,
                crate::api::types::EventType::LeftChatMembers(_) => None,
                crate::api::types::EventType::CallbackQuery(payload) => {
                    // CallbackQuery uses message timestamp
                    DateTime::from_timestamp(payload.message.timestamp.0 as i64, 0)
                }
                crate::api::types::EventType::None => None,
            }
        }
    }

    #[test]
    fn test_extract_timestamp_from_new_message() {
        let manager = MockStorageManager;

        let payload = EventPayloadNewMessage {
            msg_id: MsgId("test_msg".to_string()),
            text: "Test message".to_string(),
            chat: Chat {
                chat_id: ChatId("test_chat".into()),
                chat_type: "group".to_string(),
                title: Some("Test Chat".to_string()),
            },
            from: From {
                user_id: UserId("test_user".to_string()),
                first_name: "Test".to_string(),
                last_name: None,
            },
            format: None,
            parts: vec![],
            timestamp: Timestamp(1700000000), // Unix timestamp
        };

        let event = EventMessage {
            event_id: 123,
            event_type: EventType::NewMessage(Box::new(payload)),
        };

        let extracted_timestamp = manager.extract_timestamp(&event);
        assert!(extracted_timestamp.is_some());

        let timestamp = extracted_timestamp.unwrap();
        // Verify the timestamp is extracted correctly from the event
        assert_eq!(timestamp.timestamp(), 1700000000);
    }

    #[test]
    fn test_extract_timestamp_from_callback_query() {
        let manager = MockStorageManager;

        let message_payload = EventPayloadNewMessage {
            msg_id: MsgId("test_msg".to_string()),
            text: "Test message".to_string(),
            chat: Chat {
                chat_id: ChatId("test_chat".into()),
                chat_type: "group".to_string(),
                title: Some("Test Chat".to_string()),
            },
            from: From {
                user_id: UserId("test_user".to_string()),
                first_name: "Test".to_string(),
                last_name: None,
            },
            format: None,
            parts: vec![],
            timestamp: Timestamp(1700000000), // Unix timestamp
        };

        let callback_payload = EventPayloadCallbackQuery {
            query_id: QueryId("test_query".to_string()),
            from: From {
                user_id: UserId("test_user".to_string()),
                first_name: "Test".to_string(),
                last_name: None,
            },
            chat: Chat {
                chat_id: ChatId("test_chat".into()),
                chat_type: "group".to_string(),
                title: Some("Test Chat".to_string()),
            },
            message: message_payload,
            callback_data: "test_data".to_string(),
        };

        let event = EventMessage {
            event_id: 123,
            event_type: EventType::CallbackQuery(Box::new(callback_payload)),
        };

        let extracted_timestamp = manager.extract_timestamp(&event);
        assert!(extracted_timestamp.is_some());

        let timestamp = extracted_timestamp.unwrap();
        // Verify the timestamp is extracted from the embedded message
        assert_eq!(timestamp.timestamp(), 1700000000);
    }

    #[test]
    fn test_extract_timestamp_from_events_without_timestamp() {
        let manager = MockStorageManager;

        let new_members_payload = EventPayloadNewChatMembers {
            chat: Chat {
                chat_id: ChatId("test_chat".into()),
                chat_type: "group".to_string(),
                title: Some("Test Chat".to_string()),
            },
            new_members: vec![From {
                user_id: UserId("new_user".to_string()),
                first_name: "New".to_string(),
                last_name: None,
            }],
            added_by: From {
                user_id: UserId("admin_user".to_string()),
                first_name: "Admin".to_string(),
                last_name: None,
            },
        };

        let event = EventMessage {
            event_id: 123,
            event_type: EventType::NewChatMembers(Box::new(new_members_payload)),
        };

        let extracted_timestamp = manager.extract_timestamp(&event);
        // Events without timestamp field should return None
        assert!(extracted_timestamp.is_none());
    }

    #[test]
    fn test_extract_timestamp_accuracy() {
        let manager = MockStorageManager;

        // Test different timestamps to ensure accuracy
        let test_timestamps = vec![
            1700000000, // 2023-11-14 22:13:20 UTC
            1640995200, // 2022-01-01 00:00:00 UTC
            946684800,  // 2000-01-01 00:00:00 UTC
        ];

        for timestamp_value in test_timestamps {
            let payload = EventPayloadNewMessage {
                msg_id: MsgId("test_msg".to_string()),
                text: "Test message".to_string(),
                chat: Chat {
                    chat_id: ChatId("test_chat".into()),
                    chat_type: "group".to_string(),
                    title: Some("Test Chat".to_string()),
                },
                from: From {
                    user_id: UserId("test_user".to_string()),
                    first_name: "Test".to_string(),
                    last_name: None,
                },
                format: None,
                parts: vec![],
                timestamp: Timestamp(timestamp_value),
            };

            let event = EventMessage {
                event_id: 123,
                event_type: EventType::NewMessage(Box::new(payload)),
            };

            let extracted_timestamp = manager.extract_timestamp(&event);
            assert!(extracted_timestamp.is_some());

            let timestamp = extracted_timestamp.unwrap();
            assert_eq!(timestamp.timestamp(), timestamp_value as i64);
        }
    }

    #[test]
    fn test_event_type_to_string_all_variants() {
        let manager = MockStorageManager;

        // Create test data for all event types
        let test_cases = vec![
            (
                EventType::NewMessage(Box::new(create_test_new_message_payload())),
                "newMessage",
            ),
            (
                EventType::EditedMessage(Box::new(create_test_edited_message_payload())),
                "editedMessage",
            ),
            (
                EventType::DeleteMessage(Box::new(create_test_delete_message_payload())),
                "deleteMessage",
            ),
            (
                EventType::PinnedMessage(Box::new(create_test_pinned_message_payload())),
                "pinnedMessage",
            ),
            (
                EventType::UnpinnedMessage(Box::new(create_test_unpinned_message_payload())),
                "unpinnedMessage",
            ),
            (
                EventType::NewChatMembers(Box::new(create_test_new_chat_members_payload())),
                "newChatMembers",
            ),
            (
                EventType::LeftChatMembers(Box::new(create_test_left_chat_members_payload())),
                "leftChatMembers",
            ),
            (
                EventType::CallbackQuery(Box::new(create_test_callback_query_payload())),
                "callbackQuery",
            ),
            (EventType::None, "unknown"),
        ];

        for (event_type, expected_string) in test_cases {
            let result = manager.event_type_to_string(&event_type);
            assert_eq!(result, expected_string);
        }
    }

    #[test]
    fn test_detect_mentions() {
        let manager = MockStorageManager;

        let test_cases = vec![
            ("Hello @user123", (true, vec!["user123".to_string()])),
            ("No mentions here", (false, vec![])),
            (
                "@first and @second user",
                (true, vec!["first".to_string(), "second".to_string()]),
            ),
            ("Email at domain.com is not a mention", (false, vec![])),
            (
                "@user_with_underscore",
                (true, vec!["user_with_underscore".to_string()]),
            ),
            ("", (false, vec![])),
        ];

        for (input, (expected_has_mentions, expected_mentions)) in test_cases {
            let (has_mentions, mentions) = manager.detect_mentions(input);
            assert_eq!(has_mentions, expected_has_mentions);
            assert_eq!(mentions, expected_mentions);
        }
    }

    #[test]
    fn test_extract_file_attachments() {
        let manager = MockStorageManager;

        // Test with empty parts
        let empty_parts = vec![];
        let result = manager.extract_file_attachments(&empty_parts);
        assert!(result.is_none());

        // Test with simplified parts - create a basic file structure
        let file_parts = vec![MessageParts {
            part_type: MessagePartsType::File(Box::new(MessagePartsPayloadFile {
                file_id: FileId("file123".to_string()),
                file_type: "application/pdf".to_string(),
                caption: "document.pdf".to_string(),
                format: MessageFormat::default(),
            })),
        }];
        let result = manager.extract_file_attachments(&file_parts);
        assert!(result.is_some());
        let attachments = result.unwrap();
        assert_eq!(attachments["file_id"], "file123");
        assert_eq!(attachments["file_name"], "document.pdf");

        // Test with non-file parts
        let text_parts = vec![MessageParts {
            part_type: MessagePartsType::Mention(MessagePartsPayloadMention {
                user_id: From {
                    user_id: UserId("user123".to_string()),
                    first_name: "User".to_string(),
                    last_name: None,
                },
            }),
        }];
        let result = manager.extract_file_attachments(&text_parts);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_formatted_text() {
        let manager = MockStorageManager;

        // Test with format present
        let payload_with_format = EventPayloadNewMessage {
            msg_id: MsgId("test_msg".to_string()),
            text: "Test message".to_string(),
            chat: create_test_chat(),
            from: create_test_from(),
            format: Some(MessageFormat {
                bold: Some(vec![MessageFormatStruct {
                    offset: 0,
                    length: 4,
                }]),
                italic: None,
                underline: None,
                strikethrough: None,
                link: None,
                ordered_list: None,
                quote: None,
                mention: None,
                inline_code: None,
                pre: None,
            }),
            parts: vec![],
            timestamp: Timestamp(1700000000),
        };

        let result = manager.extract_formatted_text(&payload_with_format);
        assert!(result.is_some());
        let formatted = result.unwrap();
        assert!(formatted.contains("bold"));

        // Test with no format
        let payload_without_format = EventPayloadNewMessage {
            msg_id: MsgId("test_msg".to_string()),
            text: "Test message".to_string(),
            chat: create_test_chat(),
            from: create_test_from(),
            format: None,
            parts: vec![],
            timestamp: Timestamp(1700000000),
        };

        let result = manager.extract_formatted_text(&payload_without_format);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_formatted_text_edited() {
        let manager = MockStorageManager;

        // Test with format present
        let payload_with_format = EventPayloadEditedMessage {
            msg_id: MsgId("test_msg".to_string()),
            text: "Edited message".to_string(),
            chat: create_test_chat(),
            from: create_test_from(),
            format: Some(MessageFormat {
                bold: None,
                italic: Some(vec![MessageFormatStruct {
                    offset: 0,
                    length: 6,
                }]),
                underline: None,
                strikethrough: None,
                link: None,
                ordered_list: None,
                quote: None,
                mention: None,
                inline_code: None,
                pre: None,
            }),
            timestamp: Timestamp(1700000000),
            edited_timestamp: Timestamp(1700000001),
        };

        let result = manager.extract_formatted_text_edited(&payload_with_format);
        assert!(result.is_some());
        let formatted = result.unwrap();
        assert!(formatted.contains("italic"));

        // Test with no format
        let payload_without_format = EventPayloadEditedMessage {
            msg_id: MsgId("test_msg".to_string()),
            text: "Edited message".to_string(),
            chat: create_test_chat(),
            from: create_test_from(),
            format: None,
            timestamp: Timestamp(1700000000),
            edited_timestamp: Timestamp(1700000001),
        };

        let result = manager.extract_formatted_text_edited(&payload_without_format);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_text_content() {
        let manager = MockStorageManager;

        // Test NewMessage with text
        let new_msg_event = EventMessage {
            event_id: 123,
            event_type: EventType::NewMessage(Box::new(create_test_new_message_payload())),
        };
        let result = manager.extract_text_content(&new_msg_event);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "Test message");

        // Test EditedMessage with text
        let edited_msg_event = EventMessage {
            event_id: 123,
            event_type: EventType::EditedMessage(Box::new(create_test_edited_message_payload())),
        };
        let result = manager.extract_text_content(&edited_msg_event);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "Edited message");

        // Test NewMessage with empty text
        let mut empty_payload = create_test_new_message_payload();
        empty_payload.text = "".to_string();
        let empty_event = EventMessage {
            event_id: 123,
            event_type: EventType::NewMessage(Box::new(empty_payload)),
        };
        let result = manager.extract_text_content(&empty_event);
        assert!(result.is_none());

        // Test non-message event
        let other_event = EventMessage {
            event_id: 123,
            event_type: EventType::NewChatMembers(Box::new(create_test_new_chat_members_payload())),
        };
        let result = manager.extract_text_content(&other_event);
        assert!(result.is_none());
    }

    #[test]
    fn test_storage_config_default() {
        // Test that StorageConfig Default implementation works
        let config = StorageConfig::default();
        assert!(config.database.auto_migrate);
        assert_eq!(config.database.url, "postgresql://localhost/vkteams_bot");
    }

    // Helper functions for test data creation
    fn create_test_new_message_payload() -> EventPayloadNewMessage {
        EventPayloadNewMessage {
            msg_id: MsgId("test_msg".to_string()),
            text: "Test message".to_string(),
            chat: create_test_chat(),
            from: create_test_from(),
            format: None,
            parts: vec![],
            timestamp: Timestamp(1700000000),
        }
    }

    fn create_test_edited_message_payload() -> EventPayloadEditedMessage {
        EventPayloadEditedMessage {
            msg_id: MsgId("test_msg".to_string()),
            text: "Edited message".to_string(),
            chat: create_test_chat(),
            from: create_test_from(),
            format: None,
            timestamp: Timestamp(1700000000),
            edited_timestamp: Timestamp(1700000001),
        }
    }

    fn create_test_delete_message_payload() -> EventPayloadDeleteMessage {
        EventPayloadDeleteMessage {
            msg_id: MsgId("test_msg".to_string()),
            chat: create_test_chat(),
            timestamp: Timestamp(1700000000),
        }
    }

    fn create_test_pinned_message_payload() -> EventPayloadPinnedMessage {
        EventPayloadPinnedMessage {
            msg_id: MsgId("pinned_msg_123".to_string()),
            chat: create_test_chat(),
            from: create_test_from(),
            text: "Test pinned message".to_string(),
            format: None,
            parts: vec![],
            timestamp: Timestamp(1700000000),
        }
    }

    fn create_test_unpinned_message_payload() -> EventPayloadUnpinnedMessage {
        EventPayloadUnpinnedMessage {
            msg_id: MsgId("test_msg".to_string()),
            chat: create_test_chat(),
            timestamp: Timestamp(1700000000),
        }
    }

    fn create_test_new_chat_members_payload() -> EventPayloadNewChatMembers {
        EventPayloadNewChatMembers {
            chat: create_test_chat(),
            new_members: vec![create_test_from()],
            added_by: create_test_from(),
        }
    }

    fn create_test_left_chat_members_payload() -> EventPayloadLeftChatMembers {
        EventPayloadLeftChatMembers {
            chat: create_test_chat(),
            left_members: vec![create_test_from()],
            removed_by: Some(create_test_from()),
        }
    }

    fn create_test_callback_query_payload() -> EventPayloadCallbackQuery {
        EventPayloadCallbackQuery {
            query_id: QueryId("test_query".to_string()),
            from: create_test_from(),
            chat: create_test_chat(),
            message: create_test_new_message_payload(),
            callback_data: "test_data".to_string(),
        }
    }

    fn create_test_chat() -> Chat {
        Chat {
            chat_id: ChatId("test_chat".into()),
            chat_type: "group".to_string(),
            title: Some("Test Chat".to_string()),
        }
    }

    fn create_test_from() -> From {
        From {
            user_id: UserId("test_user".to_string()),
            first_name: "Test".to_string(),
            last_name: Some("User".to_string()),
        }
    }

    impl MockStorageManager {
        fn event_type_to_string(&self, event_type: &crate::api::types::EventType) -> String {
            match event_type {
                crate::api::types::EventType::NewMessage(_) => "newMessage".to_string(),
                crate::api::types::EventType::EditedMessage(_) => "editedMessage".to_string(),
                crate::api::types::EventType::DeleteMessage(_) => "deleteMessage".to_string(),
                crate::api::types::EventType::PinnedMessage(_) => "pinnedMessage".to_string(),
                crate::api::types::EventType::UnpinnedMessage(_) => "unpinnedMessage".to_string(),
                crate::api::types::EventType::NewChatMembers(_) => "newChatMembers".to_string(),
                crate::api::types::EventType::LeftChatMembers(_) => "leftChatMembers".to_string(),
                crate::api::types::EventType::CallbackQuery(_) => "callbackQuery".to_string(),
                crate::api::types::EventType::None => "unknown".to_string(),
            }
        }

        fn detect_mentions(&self, text: &str) -> (bool, Vec<String>) {
            use regex::Regex;
            let mention_regex = Regex::new(r"@(\w+)").unwrap();
            let mentions: Vec<String> = mention_regex
                .captures_iter(text)
                .map(|cap| cap[1].to_string())
                .collect();

            (!mentions.is_empty(), mentions)
        }

        fn extract_file_attachments(&self, parts: &[MessageParts]) -> Option<serde_json::Value> {
            for part in parts {
                if let MessagePartsType::File(file_payload) = &part.part_type {
                    return Some(serde_json::json!({
                        "file_id": file_payload.file_id.0,
                        "file_name": &file_payload.caption,
                        "file_type": &file_payload.file_type
                    }));
                }
            }
            None
        }

        fn extract_formatted_text(&self, payload: &EventPayloadNewMessage) -> Option<String> {
            if let Some(format) = &payload.format {
                serde_json::to_string(format).ok()
            } else {
                None
            }
        }

        fn extract_formatted_text_edited(
            &self,
            payload: &EventPayloadEditedMessage,
        ) -> Option<String> {
            if let Some(format) = &payload.format {
                serde_json::to_string(format).ok()
            } else {
                None
            }
        }

        fn extract_text_content(&self, event: &EventMessage) -> Option<String> {
            match &event.event_type {
                crate::api::types::EventType::NewMessage(payload) => {
                    if payload.text.is_empty() {
                        None
                    } else {
                        Some(payload.text.clone())
                    }
                }
                crate::api::types::EventType::EditedMessage(payload) => {
                    if payload.text.is_empty() {
                        None
                    } else {
                        Some(payload.text.clone())
                    }
                }
                _ => None,
            }
        }
    }
}
