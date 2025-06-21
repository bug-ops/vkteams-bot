//! Vector storage implementations for similarity search

use crate::storage::{StorageError, StorageResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};

/// Vector document for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    pub id: String,
    pub content: String,
    pub metadata: Value,
    pub embedding: pgvector::Vector,
    pub created_at: DateTime<Utc>,
}

/// Search query for vector similarity
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub embedding: pgvector::Vector,
    pub limit: usize,
    pub score_threshold: Option<f32>,
    pub metadata_filter: Option<Value>,
    pub include_content: bool,
}

/// Search result from vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub content: Option<String>,
    pub metadata: Value,
    pub score: f32,
    pub created_at: DateTime<Utc>,
}

/// Trait for vector storage backends
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Store a single document
    async fn store_document(&self, document: VectorDocument) -> StorageResult<()>;

    /// Store multiple documents
    async fn store_documents(&self, documents: Vec<VectorDocument>) -> StorageResult<()>;

    /// Search for similar documents
    async fn search_similar(&self, query: SearchQuery) -> StorageResult<Vec<SearchResult>>;

    /// Get document by ID
    async fn get_document(&self, id: &str) -> StorageResult<Option<VectorDocument>>;

    /// Delete document by ID
    async fn delete_document(&self, id: &str) -> StorageResult<bool>;

    /// Clean up old documents
    async fn cleanup_old_documents(&self, older_than: DateTime<Utc>) -> StorageResult<u64>;

    /// Health check
    async fn health_check(&self) -> StorageResult<()>;
}

/// PostgreSQL + pgvector implementation
pub struct PgVectorStore {
    pool: PgPool,
    collection_name: String,
    dimensions: usize,
}

impl PgVectorStore {
    pub async fn new(
        database_url: &str,
        collection_name: String,
        dimensions: usize,
    ) -> StorageResult<Self> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        let store = Self {
            pool,
            collection_name,
            dimensions,
        };

        store.initialize().await?;
        Ok(store)
    }

    async fn initialize(&self) -> StorageResult<()> {
        // Create pgvector extension
        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        // Create vector documents table
        let query = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                metadata JSONB NOT NULL DEFAULT '{{}}',
                embedding vector({}) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
            self.collection_name, self.dimensions
        );

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        // Create index for vector similarity search
        let index_query = format!(
            "CREATE INDEX IF NOT EXISTS {}_embedding_idx ON {} USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100)",
            self.collection_name, self.collection_name
        );

        sqlx::query(&index_query)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl VectorStore for PgVectorStore {
    async fn store_document(&self, document: VectorDocument) -> StorageResult<()> {
        let query = format!(
            "INSERT INTO {} (id, content, metadata, embedding, created_at) VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (id) DO UPDATE SET content = $2, metadata = $3, embedding = $4",
            self.collection_name
        );

        sqlx::query(&query)
            .bind(&document.id)
            .bind(&document.content)
            .bind(&document.metadata)
            .bind(&document.embedding)
            .bind(document.created_at)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        Ok(())
    }

    async fn store_documents(&self, documents: Vec<VectorDocument>) -> StorageResult<()> {
        let mut tx = self.pool.begin().await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        for document in documents {
            let query = format!(
                "INSERT INTO {} (id, content, metadata, embedding, created_at) VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT (id) DO UPDATE SET content = $2, metadata = $3, embedding = $4",
                self.collection_name
            );

            sqlx::query(&query)
                .bind(&document.id)
                .bind(&document.content)
                .bind(&document.metadata)
                .bind(&document.embedding)
                .bind(document.created_at)
                .execute(&mut *tx)
                .await
                .map_err(|e| StorageError::Query(e.to_string()))?;
        }

        tx.commit().await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        Ok(())
    }

    async fn search_similar(&self, query: SearchQuery) -> StorageResult<Vec<SearchResult>> {
        let mut sql = format!(
            "SELECT id, content, metadata, embedding <=> $1 as distance, created_at FROM {} WHERE 1=1",
            self.collection_name
        );
        
        let mut bind_count = 1;
        
        // Add score threshold filter
        if let Some(_threshold) = query.score_threshold {
            bind_count += 1;
            sql.push_str(&format!(" AND embedding <=> $1 < ${}", bind_count));
        }

        // Add metadata filter (simplified)
        if query.metadata_filter.is_some() {
            bind_count += 1;
            sql.push_str(&format!(" AND metadata @> ${}", bind_count));
        }

        sql.push_str(&format!(" ORDER BY embedding <=> $1 LIMIT {}", query.limit));

        let mut sqlx_query = sqlx::query(&sql).bind(&query.embedding);

        if let Some(threshold) = query.score_threshold {
            sqlx_query = sqlx_query.bind(threshold);
        }

        if let Some(metadata) = query.metadata_filter {
            sqlx_query = sqlx_query.bind(metadata);
        }

        let rows = sqlx_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let content = if query.include_content {
                Some(row.get::<String, _>("content"))
            } else {
                None
            };

            results.push(SearchResult {
                id: row.get("id"),
                content,
                metadata: row.get("metadata"),
                score: 1.0 - row.get::<f64, _>("distance") as f32, // Convert distance to similarity
                created_at: row.get("created_at"),
            });
        }

        Ok(results)
    }

    async fn get_document(&self, id: &str) -> StorageResult<Option<VectorDocument>> {
        let query = format!(
            "SELECT id, content, metadata, embedding, created_at FROM {} WHERE id = $1",
            self.collection_name
        );

        let row = sqlx::query(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        if let Some(row) = row {
            Ok(Some(VectorDocument {
                id: row.get("id"),
                content: row.get("content"),
                metadata: row.get("metadata"),
                embedding: row.get("embedding"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn delete_document(&self, id: &str) -> StorageResult<bool> {
        let query = format!("DELETE FROM {} WHERE id = $1", self.collection_name);

        let result = sqlx::query(&query)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    async fn cleanup_old_documents(&self, older_than: DateTime<Utc>) -> StorageResult<u64> {
        let query = format!("DELETE FROM {} WHERE created_at < $1", self.collection_name);

        let result = sqlx::query(&query)
            .bind(older_than)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        Ok(result.rows_affected())
    }

    async fn health_check(&self) -> StorageResult<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        Ok(())
    }
}

/// Create vector store instance
pub async fn create_vector_store(
    provider: &str,
    connection_url: &str,
    collection_name: Option<String>,
) -> StorageResult<Box<dyn VectorStore>> {
    match provider {
        "pgvector" => {
            let collection = collection_name.unwrap_or_else(|| "vector_documents".to_string());
            let store = PgVectorStore::new(connection_url, collection, 1536).await?;
            Ok(Box::new(store))
        }
        _ => Err(StorageError::Configuration(
            format!("Unknown vector store provider: {}", provider)
        )),
    }
}