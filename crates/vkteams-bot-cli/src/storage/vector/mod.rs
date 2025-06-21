//! Vector storage abstraction and implementations

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::storage::{StorageError, StorageResult};

#[cfg(feature = "vector-search")]
pub mod pgvector;

#[cfg(feature = "vector-search")]
pub use pgvector::PgVectorStore;

/// Vector document for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    pub id: String,
    pub content: String,
    pub metadata: serde_json::Value,
    #[cfg(feature = "vector-search")]
    pub embedding: pgvector::Vector,
    #[cfg(not(feature = "vector-search"))]
    pub embedding: Vec<f32>,
    pub created_at: DateTime<Utc>,
}

/// Search result from vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub score: f32, // similarity score
    pub distance: f32, // vector distance
}

/// Search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    #[cfg(feature = "vector-search")]
    pub embedding: pgvector::Vector,
    #[cfg(not(feature = "vector-search"))]
    pub embedding: Vec<f32>,
    pub limit: usize,
    pub score_threshold: Option<f32>,
    pub metadata_filter: Option<serde_json::Value>,
    pub include_content: bool,
}

/// Vector store statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct VectorStoreStats {
    pub total_documents: u64,
    pub storage_size_bytes: u64,
    pub index_size_bytes: Option<u64>,
    pub avg_query_time_ms: f64,
    pub provider: String,
}

/// Vector store trait for different implementations
#[async_trait]
pub trait VectorStore: Send + Sync + Clone {
    /// Store a document with vector representation
    async fn store_document(&self, document: VectorDocument) -> StorageResult<String>;
    
    /// Store multiple documents in batch
    async fn store_documents(&self, documents: Vec<VectorDocument>) -> StorageResult<Vec<String>>;
    
    /// Search for similar vectors
    async fn search_similar(&self, query: SearchQuery) -> StorageResult<Vec<SearchResult>>;
    
    /// Get document by ID
    async fn get_document(&self, id: &str) -> StorageResult<Option<VectorDocument>>;
    
    /// Delete document
    async fn delete_document(&self, id: &str) -> StorageResult<bool>;
    
    /// Update document metadata
    async fn update_metadata(&self, id: &str, metadata: serde_json::Value) -> StorageResult<()>;
    
    /// Clean up old documents
    async fn cleanup_old_documents(&self, older_than: DateTime<Utc>) -> StorageResult<u64>;
    
    /// Get store statistics
    async fn get_stats(&self) -> StorageResult<VectorStoreStats>;
    
    /// Health check
    async fn health_check(&self) -> StorageResult<()>;
}

/// Create vector store based on configuration
#[cfg(feature = "vector-search")]
pub async fn create_vector_store(
    provider: &str,
    connection_url: &str,
    collection_name: Option<String>,
) -> StorageResult<Box<dyn VectorStore>> {
    match provider {
        "pgvector" => {
            let store = PgVectorStore::new(
                connection_url,
                collection_name.unwrap_or_else(|| "vkteams_embeddings".to_string())
            ).await?;
            Ok(Box::new(store))
        },
        _ => Err(StorageError::Configuration(
            format!("Unknown vector store provider: {}", provider)
        ))
    }
}