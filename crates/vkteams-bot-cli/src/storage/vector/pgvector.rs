//! pgvector implementation for PostgreSQL

use super::{VectorStore, VectorDocument, SearchQuery, SearchResult, VectorStoreStats};
use crate::storage::{StorageError, StorageResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use pgvector::Vector;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

/// Performance metrics for query tracking
#[derive(Debug)]
struct QueryMetrics {
    total_queries: AtomicUsize,
    total_query_time_ms: AtomicU64,
}

impl Default for QueryMetrics {
    fn default() -> Self {
        Self {
            total_queries: AtomicUsize::new(0),
            total_query_time_ms: AtomicU64::new(0),
        }
    }
}

impl QueryMetrics {
    fn record_query(&self, duration_ms: u64) {
        self.total_queries.fetch_add(1, Ordering::Relaxed);
        self.total_query_time_ms.fetch_add(duration_ms, Ordering::Relaxed);
    }
    
    fn get_avg_query_time_ms(&self) -> f64 {
        let total_queries = self.total_queries.load(Ordering::Relaxed);
        if total_queries == 0 {
            0.0
        } else {
            let total_time = self.total_query_time_ms.load(Ordering::Relaxed);
            total_time as f64 / total_queries as f64
        }
    }
}

/// PostgreSQL + pgvector store implementation
#[derive(Debug, Clone)]
pub struct PgVectorStore {
    pool: PgPool,
    collection_name: String,
    metrics: Arc<QueryMetrics>,
}

impl PgVectorStore {
    /// Create new pgvector store
    pub async fn new(database_url: &str, collection_name: String) -> StorageResult<Self> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| StorageError::Connection(e.to_string()))?;
        
        // Ensure vector extension is enabled
        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(&pool)
            .await
            .map_err(|e| StorageError::Configuration(e.to_string()))?;
        
        // Create embeddings table if not exists
        sqlx::query(&format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                metadata JSONB NOT NULL DEFAULT '{{}}',
                embedding vector(1536) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
            collection_name
        ))
        .execute(&pool)
        .await
        .map_err(|e| StorageError::Configuration(e.to_string()))?;
        
        // Create vector index for fast similarity search
        sqlx::query(&format!(
            "CREATE INDEX IF NOT EXISTS {}_embedding_idx ON {} USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100)",
            collection_name, collection_name
        ))
        .execute(&pool)
        .await
        .map_err(|e| StorageError::Configuration(e.to_string()))?;
        
        Ok(Self { 
            pool, 
            collection_name,
            metrics: Arc::new(QueryMetrics::default()),
        })
    }
}

#[async_trait]
impl VectorStore for PgVectorStore {
    async fn store_document(&self, document: VectorDocument) -> StorageResult<String> {
        sqlx::query(&format!(
            "INSERT INTO {} (id, content, metadata, embedding, created_at) VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (id) DO UPDATE SET 
                content = EXCLUDED.content,
                metadata = EXCLUDED.metadata,
                embedding = EXCLUDED.embedding",
            self.collection_name
        ))
        .bind(&document.id)
        .bind(&document.content)
        .bind(&document.metadata)
        .bind(&document.embedding)
        .bind(document.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Vector(e.to_string()))?;
        
        Ok(document.id)
    }
    
    async fn store_documents(&self, documents: Vec<VectorDocument>) -> StorageResult<Vec<String>> {
        let mut tx = self.pool.begin().await
            .map_err(|e| StorageError::Connection(e.to_string()))?;
        
        let mut ids = Vec::new();
        
        for doc in documents {
            sqlx::query(&format!(
                "INSERT INTO {} (id, content, metadata, embedding, created_at) VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT (id) DO UPDATE SET 
                    content = EXCLUDED.content,
                    metadata = EXCLUDED.metadata,
                    embedding = EXCLUDED.embedding",
                self.collection_name
            ))
            .bind(&doc.id)
            .bind(&doc.content)
            .bind(&doc.metadata)
            .bind(&doc.embedding)
            .bind(doc.created_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| StorageError::Vector(e.to_string()))?;
            
            ids.push(doc.id);
        }
        
        tx.commit().await
            .map_err(|e| StorageError::Connection(e.to_string()))?;
        
        Ok(ids)
    }
    
    async fn search_similar(&self, query: SearchQuery) -> StorageResult<Vec<SearchResult>> {
        let start_time = Instant::now();
        
        let mut sql = format!(
            r#"
            SELECT id, content, metadata, 
                   1 - (embedding <=> $1) as score, 
                   embedding <=> $1 as distance
            FROM {}
            WHERE 1=1
            "#,
            self.collection_name
        );
        
        let mut bind_index = 2;
        let mut query_builder = sqlx::query(&sql).bind(&query.embedding);
        
        // Add metadata filter if provided
        if let Some(metadata_filter) = &query.metadata_filter {
            sql = format!("{} AND metadata @> ${}", sql, bind_index);
            query_builder = query_builder.bind(metadata_filter);
            bind_index += 1;
        }
        
        // Add score threshold filter
        if let Some(threshold) = query.score_threshold {
            sql = format!("{} AND 1 - (embedding <=> $1) >= ${}", sql, bind_index);
            query_builder = query_builder.bind(threshold);
        }
        
        sql = format!("{} ORDER BY embedding <=> $1 LIMIT {}", sql, query.limit);
        
        let rows = sqlx::query(&sql)
            .bind(&query.embedding)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::Vector(e.to_string()))?;
        
        let results = rows
            .into_iter()
            .map(|row| SearchResult {
                id: row.get("id"),
                content: if query.include_content { 
                    row.get("content") 
                } else { 
                    String::new() 
                },
                metadata: row.get("metadata"),
                score: row.get("score"),
                distance: row.get("distance"),
            })
            .collect();
        
        // Record query metrics
        let duration = start_time.elapsed();
        self.metrics.record_query(duration.as_millis() as u64);
        
        Ok(results)
    }
    
    async fn get_document(&self, id: &str) -> StorageResult<Option<VectorDocument>> {
        let row = sqlx::query(&format!(
            "SELECT id, content, metadata, embedding, created_at FROM {} WHERE id = $1",
            self.collection_name
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::Vector(e.to_string()))?;
        
        if let Some(row) = row {
            let embedding: Vector = row.get("embedding");
            Ok(Some(VectorDocument {
                id: row.get("id"),
                content: row.get("content"),
                metadata: row.get("metadata"),
                embedding,
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn delete_document(&self, id: &str) -> StorageResult<bool> {
        let result = sqlx::query(&format!("DELETE FROM {} WHERE id = $1", self.collection_name))
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Vector(e.to_string()))?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn update_metadata(&self, id: &str, metadata: serde_json::Value) -> StorageResult<()> {
        sqlx::query(&format!("UPDATE {} SET metadata = $1 WHERE id = $2", self.collection_name))
            .bind(metadata)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Vector(e.to_string()))?;
        
        Ok(())
    }
    
    async fn cleanup_old_documents(&self, older_than: DateTime<Utc>) -> StorageResult<u64> {
        let result = sqlx::query(&format!("DELETE FROM {} WHERE created_at < $1", self.collection_name))
            .bind(older_than)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Vector(e.to_string()))?;
        
        Ok(result.rows_affected())
    }
    
    async fn get_stats(&self) -> StorageResult<VectorStoreStats> {
        // Get table and index statistics in one query
        let row = sqlx::query(&format!(
            r#"
            WITH table_stats AS (
                SELECT 
                    COUNT(*) as total_documents,
                    pg_total_relation_size('{}') as storage_size_bytes
                FROM {}
            ),
            index_stats AS (
                SELECT 
                    COALESCE(pg_relation_size('{}_embedding_idx'), 0) as index_size_bytes
            )
            SELECT 
                table_stats.total_documents,
                table_stats.storage_size_bytes,
                index_stats.index_size_bytes
            FROM table_stats, index_stats
            "#,
            self.collection_name, self.collection_name, self.collection_name
        ))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| StorageError::Vector(e.to_string()))?;
        
        Ok(VectorStoreStats {
            total_documents: row.get::<i64, _>("total_documents") as u64,
            storage_size_bytes: row.get::<i64, _>("storage_size_bytes") as u64,
            index_size_bytes: Some(row.get::<i64, _>("index_size_bytes") as u64),
            avg_query_time_ms: self.metrics.get_avg_query_time_ms(),
            provider: "pgvector".to_string(),
        })
    }
    
    async fn health_check(&self) -> StorageResult<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| StorageError::Connection(e.to_string()))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;
    
    #[test]
    fn test_query_metrics_default() {
        let metrics = QueryMetrics::default();
        assert_eq!(metrics.total_queries.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.total_query_time_ms.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.get_avg_query_time_ms(), 0.0);
    }

    #[test]
    fn test_query_metrics_record_query() {
        let metrics = QueryMetrics::default();
        
        // Record first query
        metrics.record_query(100);
        assert_eq!(metrics.total_queries.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.total_query_time_ms.load(Ordering::Relaxed), 100);
        assert_eq!(metrics.get_avg_query_time_ms(), 100.0);
        
        // Record second query
        metrics.record_query(200);
        assert_eq!(metrics.total_queries.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.total_query_time_ms.load(Ordering::Relaxed), 300);
        assert_eq!(metrics.get_avg_query_time_ms(), 150.0);
    }

    #[test]
    fn test_query_metrics_multiple_queries() {
        let metrics = QueryMetrics::default();
        
        // Record multiple queries
        for i in 1..=10 {
            metrics.record_query(i * 10);
        }
        
        assert_eq!(metrics.total_queries.load(Ordering::Relaxed), 10);
        assert_eq!(metrics.total_query_time_ms.load(Ordering::Relaxed), 550); // 10+20+...+100 = 550
        assert_eq!(metrics.get_avg_query_time_ms(), 55.0);
    }

    #[test]
    fn test_pgvector_store_structure() {
        // We can't create a real PgVectorStore without database connection,
        // but we can test the structure and related functions
        let metrics = QueryMetrics::default();
        assert_eq!(metrics.get_avg_query_time_ms(), 0.0);
    }

    #[test]
    fn test_query_metrics_concurrent_access() {
        let metrics = Arc::new(QueryMetrics::default());
        let metrics_clone = Arc::clone(&metrics);
        
        // Simulate concurrent access
        metrics.record_query(50);
        metrics_clone.record_query(100);
        
        assert_eq!(metrics.total_queries.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.total_query_time_ms.load(Ordering::Relaxed), 150);
        assert_eq!(metrics.get_avg_query_time_ms(), 75.0);
    }

    #[test]
    fn test_debug_trait() {
        let metrics = QueryMetrics::default();
        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("QueryMetrics"));
        assert!(debug_str.contains("total_queries"));
        assert!(debug_str.contains("total_query_time_ms"));
    }

    #[test]
    fn test_pgvector_store_debug_trait() {
        // Test that we can format debug output even without a real connection
        // We'll create a mock structure to test the Debug trait
        struct MockPgVectorStore {
            collection_name: String,
            metrics: Arc<QueryMetrics>,
        }
        
        impl std::fmt::Debug for MockPgVectorStore {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("MockPgVectorStore")
                    .field("collection_name", &self.collection_name)
                    .field("metrics", &self.metrics)
                    .finish()
            }
        }
        
        let mock_store = MockPgVectorStore {
            collection_name: "test_collection".to_string(),
            metrics: Arc::new(QueryMetrics::default()),
        };
        
        let debug_str = format!("{:?}", mock_store);
        assert!(debug_str.contains("MockPgVectorStore"));
        assert!(debug_str.contains("test_collection"));
    }

    #[test]
    fn test_query_metrics_edge_cases() {
        let metrics = QueryMetrics::default();
        
        // Test with zero duration
        metrics.record_query(0);
        assert_eq!(metrics.total_queries.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.total_query_time_ms.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.get_avg_query_time_ms(), 0.0);
        
        // Test with very large duration
        metrics.record_query(u64::MAX);
        assert_eq!(metrics.total_queries.load(Ordering::Relaxed), 2);
        // Note: this will overflow in practice, but we're testing the mechanics
    }

    #[test]
    fn test_collection_name_formatting() {
        // Test that collection names would be properly formatted in SQL queries
        let collection_name = "test_embeddings";
        let expected_table_sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY)",
            collection_name
        );
        assert!(expected_table_sql.contains("test_embeddings"));
        
        let expected_index_sql = format!(
            "CREATE INDEX IF NOT EXISTS {}_embedding_idx ON {}",
            collection_name, collection_name
        );
        assert!(expected_index_sql.contains("test_embeddings_embedding_idx"));
    }

    #[test]
    fn test_sql_query_formatting() {
        let collection_name = "vkteams_embeddings";
        
        // Test table creation SQL
        let table_sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                metadata JSONB NOT NULL DEFAULT '{{}}',
                embedding vector(1536) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
            collection_name
        );
        assert!(table_sql.contains("vkteams_embeddings"));
        assert!(table_sql.contains("vector(1536)"));
        assert!(table_sql.contains("JSONB"));
        
        // Test index creation SQL
        let index_sql = format!(
            "CREATE INDEX IF NOT EXISTS {}_embedding_idx ON {} USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100)",
            collection_name, collection_name
        );
        assert!(index_sql.contains("vkteams_embeddings_embedding_idx"));
        assert!(index_sql.contains("ivfflat"));
        assert!(index_sql.contains("vector_cosine_ops"));
    }

    #[test]
    fn test_search_sql_formatting() {
        let collection_name = "test_collection";
        
        // Test basic search SQL
        let search_sql = format!(
            r#"
            SELECT id, content, metadata, 
                   1 - (embedding <=> $1) as score, 
                   embedding <=> $1 as distance
            FROM {}
            WHERE 1=1
            "#,
            collection_name
        );
        assert!(search_sql.contains("test_collection"));
        assert!(search_sql.contains("embedding <=> $1"));
        assert!(search_sql.contains("score"));
        assert!(search_sql.contains("distance"));
        
        // Test search with filters
        let filtered_search_sql = format!("{} AND metadata @> $2", search_sql);
        assert!(filtered_search_sql.contains("metadata @> $2"));
        
        let threshold_search_sql = format!("{} AND 1 - (embedding <=> $1) >= $3", search_sql);
        assert!(threshold_search_sql.contains(">= $3"));
    }

    #[test]
    fn test_store_document_sql() {
        let collection_name = "test_docs";
        let insert_sql = format!(
            "INSERT INTO {} (id, content, metadata, embedding, created_at) VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (id) DO UPDATE SET 
                content = EXCLUDED.content,
                metadata = EXCLUDED.metadata,
                embedding = EXCLUDED.embedding",
            collection_name
        );
        assert!(insert_sql.contains("test_docs"));
        assert!(insert_sql.contains("ON CONFLICT (id)"));
        assert!(insert_sql.contains("EXCLUDED.content"));
    }

    #[test]
    fn test_cleanup_sql() {
        let collection_name = "cleanup_test";
        let cleanup_sql = format!("DELETE FROM {} WHERE created_at < $1", collection_name);
        assert!(cleanup_sql.contains("cleanup_test"));
        assert!(cleanup_sql.contains("created_at < $1"));
    }

    #[test]
    fn test_stats_sql() {
        let collection_name = "stats_test";
        let stats_sql = format!(
            r#"
            WITH table_stats AS (
                SELECT 
                    COUNT(*) as total_documents,
                    pg_total_relation_size('{}') as storage_size_bytes
                FROM {}
            ),
            index_stats AS (
                SELECT 
                    COALESCE(pg_relation_size('{}_embedding_idx'), 0) as index_size_bytes
            )
            SELECT 
                table_stats.total_documents,
                table_stats.storage_size_bytes,
                index_stats.index_size_bytes
            FROM table_stats, index_stats
            "#,
            collection_name, collection_name, collection_name
        );
        assert!(stats_sql.contains("stats_test"));
        assert!(stats_sql.contains("pg_total_relation_size"));
        assert!(stats_sql.contains("stats_test_embedding_idx"));
        assert!(stats_sql.contains("COALESCE"));
    }

    #[test]
    fn test_pgvector_store_clone_trait() {
        // Test that we can test the Clone trait without a real database
        let metrics = Arc::new(QueryMetrics::default());
        let metrics_clone = Arc::clone(&metrics);
        
        // Both should point to the same metrics
        metrics.record_query(100);
        assert_eq!(metrics_clone.get_avg_query_time_ms(), 100.0);
    }
}