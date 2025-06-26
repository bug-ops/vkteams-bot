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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    fn create_test_vector_document() -> VectorDocument {
        VectorDocument {
            id: "test_doc_1".to_string(),
            content: "This is a test document for vector search".to_string(),
            metadata: json!({"category": "test", "priority": "high"}),
            #[cfg(feature = "vector-search")]
            embedding: pgvector::Vector::from(vec![0.1, 0.2, 0.3, 0.4, 0.5]),
            #[cfg(not(feature = "vector-search"))]
            embedding: vec![0.1, 0.2, 0.3, 0.4, 0.5],
            created_at: Utc::now(),
        }
    }

    fn create_test_search_query() -> SearchQuery {
        SearchQuery {
            #[cfg(feature = "vector-search")]
            embedding: pgvector::Vector::from(vec![0.1, 0.2, 0.3, 0.4, 0.5]),
            #[cfg(not(feature = "vector-search"))]
            embedding: vec![0.1, 0.2, 0.3, 0.4, 0.5],
            limit: 10,
            score_threshold: Some(0.7),
            metadata_filter: Some(json!({"category": "test"})),
            include_content: true,
        }
    }

    #[test]
    fn test_vector_document_creation() {
        let doc = create_test_vector_document();
        assert_eq!(doc.id, "test_doc_1");
        assert_eq!(doc.content, "This is a test document for vector search");
        assert_eq!(doc.metadata["category"], "test");
        assert_eq!(doc.metadata["priority"], "high");
        assert_eq!(doc.embedding.len(), 5);
        assert!(doc.created_at <= Utc::now());
    }

    #[test]
    fn test_search_query_creation() {
        let query = create_test_search_query();
        assert_eq!(query.limit, 10);
        assert_eq!(query.score_threshold, Some(0.7));
        assert!(query.metadata_filter.is_some());
        assert!(query.include_content);
        assert_eq!(query.embedding.len(), 5);
    }

    #[test]
    fn test_search_result_structure() {
        let result = SearchResult {
            id: "result_1".to_string(),
            content: "Found content".to_string(),
            metadata: json!({"tag": "relevant"}),
            score: 0.85,
            distance: 0.15,
        };

        assert_eq!(result.id, "result_1");
        assert_eq!(result.content, "Found content");
        assert_eq!(result.metadata["tag"], "relevant");
        assert_eq!(result.score, 0.85);
        assert_eq!(result.distance, 0.15);
    }

    #[test]
    fn test_vector_store_stats_structure() {
        let stats = VectorStoreStats {
            total_documents: 1000,
            storage_size_bytes: 1024 * 1024, // 1MB
            index_size_bytes: Some(512 * 1024), // 512KB
            avg_query_time_ms: 25.5,
            provider: "test_provider".to_string(),
        };

        assert_eq!(stats.total_documents, 1000);
        assert_eq!(stats.storage_size_bytes, 1024 * 1024);
        assert_eq!(stats.index_size_bytes, Some(512 * 1024));
        assert_eq!(stats.avg_query_time_ms, 25.5);
        assert_eq!(stats.provider, "test_provider");
    }

    #[test]
    fn test_vector_document_serialization() {
        let doc = create_test_vector_document();
        
        // Test serialization to JSON
        let json_str = serde_json::to_string(&doc).unwrap();
        assert!(json_str.contains("test_doc_1"));
        assert!(json_str.contains("This is a test document"));
        assert!(json_str.contains("category"));
        
        // Test deserialization from JSON
        let deserialized: VectorDocument = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.id, doc.id);
        assert_eq!(deserialized.content, doc.content);
        assert_eq!(deserialized.metadata, doc.metadata);
    }

    #[test]
    fn test_search_query_serialization() {
        let query = create_test_search_query();
        
        // Test serialization to JSON
        let json_str = serde_json::to_string(&query).unwrap();
        assert!(json_str.contains("limit"));
        assert!(json_str.contains("10"));
        assert!(json_str.contains("score_threshold"));
        
        // Test deserialization from JSON
        let deserialized: SearchQuery = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.limit, query.limit);
        assert_eq!(deserialized.score_threshold, query.score_threshold);
        assert_eq!(deserialized.include_content, query.include_content);
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            id: "result_1".to_string(),
            content: "Found content".to_string(),
            metadata: json!({"tag": "relevant"}),
            score: 0.85,
            distance: 0.15,
        };
        
        // Test serialization to JSON
        let json_str = serde_json::to_string(&result).unwrap();
        assert!(json_str.contains("result_1"));
        assert!(json_str.contains("Found content"));
        assert!(json_str.contains("0.85"));
        
        // Test deserialization from JSON
        let deserialized: SearchResult = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.id, result.id);
        assert_eq!(deserialized.content, result.content);
        assert_eq!(deserialized.score, result.score);
        assert_eq!(deserialized.distance, result.distance);
    }

    #[test]
    fn test_vector_store_stats_serialization() {
        let stats = VectorStoreStats {
            total_documents: 1000,
            storage_size_bytes: 1024 * 1024,
            index_size_bytes: Some(512 * 1024),
            avg_query_time_ms: 25.5,
            provider: "test_provider".to_string(),
        };
        
        // Test serialization to JSON
        let json_str = serde_json::to_string(&stats).unwrap();
        assert!(json_str.contains("total_documents"));
        assert!(json_str.contains("1000"));
        assert!(json_str.contains("test_provider"));
        
        // Test deserialization from JSON
        let deserialized: VectorStoreStats = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.total_documents, stats.total_documents);
        assert_eq!(deserialized.storage_size_bytes, stats.storage_size_bytes);
        assert_eq!(deserialized.provider, stats.provider);
    }

    #[test]
    fn test_search_query_default_values() {
        let query = SearchQuery {
            #[cfg(feature = "vector-search")]
            embedding: pgvector::Vector::from(vec![1.0]),
            #[cfg(not(feature = "vector-search"))]
            embedding: vec![1.0],
            limit: 5,
            score_threshold: None,
            metadata_filter: None,
            include_content: false,
        };

        assert_eq!(query.limit, 5);
        assert!(query.score_threshold.is_none());
        assert!(query.metadata_filter.is_none());
        assert!(!query.include_content);
    }

    #[test]
    fn test_vector_document_metadata_variants() {
        let doc1 = VectorDocument {
            id: "doc1".to_string(),
            content: "Content 1".to_string(),
            metadata: json!({}),
            #[cfg(feature = "vector-search")]
            embedding: pgvector::Vector::from(vec![1.0]),
            #[cfg(not(feature = "vector-search"))]
            embedding: vec![1.0],
            created_at: Utc::now(),
        };

        let doc2 = VectorDocument {
            id: "doc2".to_string(),
            content: "Content 2".to_string(),
            metadata: json!({"complex": {"nested": {"value": 42}}}),
            #[cfg(feature = "vector-search")]
            embedding: pgvector::Vector::from(vec![2.0]),
            #[cfg(not(feature = "vector-search"))]
            embedding: vec![2.0],
            created_at: Utc::now(),
        };

        assert_eq!(doc1.metadata, json!({}));
        assert_eq!(doc2.metadata["complex"]["nested"]["value"], 42);
    }

    #[test]
    #[cfg(feature = "vector-search")]
    fn test_create_vector_store_pgvector() {
        let future = create_vector_store(
            "pgvector",
            "postgresql://test:test@localhost/test",
            Some("test_collection".to_string()),
        );
        
        // We can't test the actual creation without a database connection,
        // but we can test that the function signature is correct
        assert!(std::future::Future::into_future(future).is_pending());
    }

    #[test]
    #[cfg(feature = "vector-search")]
    fn test_create_vector_store_unknown_provider() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let result = create_vector_store(
                "unknown_provider",
                "test://connection",
                None,
            ).await;
            
            assert!(result.is_err());
            if let Err(StorageError::Configuration(msg)) = result {
                assert!(msg.contains("Unknown vector store provider"));
                assert!(msg.contains("unknown_provider"));
            } else {
                panic!("Expected Configuration error");
            }
        });
    }

    #[test]
    fn test_debug_traits() {
        let doc = create_test_vector_document();
        let query = create_test_search_query();
        let stats = VectorStoreStats {
            total_documents: 1000,
            storage_size_bytes: 1024,
            index_size_bytes: None,
            avg_query_time_ms: 10.0,
            provider: "test".to_string(),
        };

        // Test Debug trait implementations
        let doc_debug = format!("{:?}", doc);
        assert!(doc_debug.contains("VectorDocument"));
        assert!(doc_debug.contains("test_doc_1"));

        let query_debug = format!("{:?}", query);
        assert!(query_debug.contains("SearchQuery"));
        assert!(query_debug.contains("limit"));

        let stats_debug = format!("{:?}", stats);
        assert!(stats_debug.contains("VectorStoreStats"));
        assert!(stats_debug.contains("1000"));
    }

    #[test]
    fn test_clone_traits() {
        let doc = create_test_vector_document();
        let query = create_test_search_query();

        // Test Clone trait implementations
        let doc_clone = doc.clone();
        assert_eq!(doc_clone.id, doc.id);
        assert_eq!(doc_clone.content, doc.content);

        let query_clone = query.clone();
        assert_eq!(query_clone.limit, query.limit);
        assert_eq!(query_clone.include_content, query.include_content);
    }
}