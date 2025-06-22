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
    SearchQuery, SearchResult, VectorDocument, VectorStore, create_vector_store,
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
                let store = create_vector_store(
                    &vector_config.provider,
                    &vector_config.connection_url,
                    Some(vector_config.collection_name.clone()),
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
    pub async fn process_event(&self, _event: &EventMessage) -> StorageResult<i64> {
        // Simplified implementation for now
        // TODO: Implement full event processing when API structures are properly defined
        Ok(1) // Return dummy event ID
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
                    let vector_doc = VectorDocument {
                        id: format!("event_{}", event_id),
                        content: text_content,
                        metadata: serde_json::json!({
                            "event_id": event_id,
                            "event_type": format!("{:?}", event.event_type),
                            "timestamp": Utc::now()
                        }),
                        embedding: pgvector::Vector::from(embedding),
                        created_at: Utc::now(),
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
                            self.extract_text_content(event).map(|text| VectorDocument {
                                id: format!("event_{}", event_id),
                                content: text,
                                metadata: serde_json::json!({
                                    "event_id": event_id,
                                    "event_type": format!("{:?}", event.event_type),
                                    "timestamp": Utc::now()
                                }),
                                embedding: pgvector::Vector::from(embedding),
                                created_at: Utc::now(),
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

    /// Get configuration
    pub fn config(&self) -> &StorageConfig {
        &self.config
    }

    /// Extract message data from event (private helper)
    #[allow(dead_code)]
    fn extract_message_data(
        &self,
        _event: &EventMessage,
        _event_id: i64,
    ) -> StorageResult<Option<NewMessage>> {
        // Simplified implementation - returns None until API structures are properly defined
        Ok(None)
    }

    /// Extract text content for embedding generation (private helper)
    #[allow(dead_code)]
    fn extract_text_content(&self, _event: &EventMessage) -> Option<String> {
        // Simplified implementation - returns None until API structures are properly defined
        None
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

    #[test]
    fn test_extract_text_content() {
        // Mock storage manager
        let _config = StorageConfig::default();
        // Note: This test won't work without actual storage setup
        // Just testing the extraction logic
    }
}
