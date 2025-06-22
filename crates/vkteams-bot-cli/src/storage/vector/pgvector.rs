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