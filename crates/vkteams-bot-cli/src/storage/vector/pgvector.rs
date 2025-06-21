//! pgvector implementation for PostgreSQL

use super::{VectorStore, VectorDocument, SearchQuery, SearchResult, VectorStoreStats};
use crate::storage::{StorageError, StorageResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use pgvector::Vector;
use sqlx::{PgPool, Row};

/// PostgreSQL + pgvector store implementation
#[derive(Debug, Clone)]
pub struct PgVectorStore {
    pool: PgPool,
    collection_name: String,
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
        
        Ok(Self { pool, collection_name })
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
        let row = sqlx::query(&format!(
            r#"
            SELECT 
                COUNT(*) as total_documents,
                pg_total_relation_size('{}') as storage_size_bytes
            FROM {}
            "#,
            self.collection_name, self.collection_name
        ))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| StorageError::Vector(e.to_string()))?;
        
        Ok(VectorStoreStats {
            total_documents: row.get::<i64, _>("total_documents") as u64,
            storage_size_bytes: row.get::<i64, _>("storage_size_bytes") as u64,
            index_size_bytes: None, // TODO: calculate index size
            avg_query_time_ms: 0.0, // TODO: collect metrics
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