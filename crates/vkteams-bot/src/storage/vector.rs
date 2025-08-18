//! Vector storage implementations for similarity search

use crate::storage::config::SslConfig;
use crate::storage::{StorageError, StorageResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{
    PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};
use std::str::FromStr;
use std::time::Instant;

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

/// Performance metrics for pgvector operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMetrics {
    /// Total number of vector documents stored
    pub total_documents: i64,
    /// Total size of vector data in bytes
    pub total_size_bytes: i64,
    /// Size of vector indexes in bytes
    pub index_size_bytes: i64,
    /// Number of vector dimensions
    pub dimensions: usize,
    /// Collection name
    pub collection_name: String,
    /// Last query execution time in milliseconds
    pub last_query_time_ms: f64,
    /// Average query time over recent queries (rolling average)
    pub avg_query_time_ms: f64,
    /// Total number of queries executed
    pub total_queries: i64,
    /// Number of failed queries
    pub failed_queries: i64,
    /// Last vacuum/maintenance timestamp
    pub last_maintenance: Option<DateTime<Utc>>,
    /// Index usage statistics
    pub index_usage: IndexUsageStats,
}

/// Index usage statistics for performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexUsageStats {
    /// Number of index scans performed
    pub index_scans: i64,
    /// Number of index tuples read
    pub index_tuples_read: i64,
    /// Number of index tuples fetched
    pub index_tuples_fetched: i64,
    /// Number of blocks read from index
    pub index_blocks_read: i64,
    /// Number of blocks hit in cache
    pub index_blocks_hit: i64,
    /// Index cache hit ratio (0.0 to 1.0)
    pub cache_hit_ratio: f64,
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

    /// Get performance metrics for the vector store
    async fn get_metrics(&self) -> StorageResult<VectorMetrics>;

    /// Perform maintenance operations (vacuum, analyze, etc.)
    async fn perform_maintenance(&self) -> StorageResult<()>;
}

/// PostgreSQL + pgvector implementation
pub struct PgVectorStore {
    pool: PgPool,
    collection_name: String,
    dimensions: usize,
    ivfflat_lists: u32,
    // Performance tracking
    query_count: std::sync::atomic::AtomicI64,
    failed_query_count: std::sync::atomic::AtomicI64,
    total_query_time_ms: std::sync::atomic::AtomicU64, // Using AtomicU64 for f64 representation
}

impl PgVectorStore {
    pub async fn new(
        database_url: &str,
        collection_name: String,
        dimensions: usize,
        ivfflat_lists: u32,
    ) -> StorageResult<Self> {
        Self::new_with_ssl(
            database_url,
            collection_name,
            dimensions,
            ivfflat_lists,
            &SslConfig::default(),
        )
        .await
    }

    pub async fn new_with_ssl(
        database_url: &str,
        collection_name: String,
        dimensions: usize,
        ivfflat_lists: u32,
        ssl_config: &SslConfig,
    ) -> StorageResult<Self> {
        let pool = if ssl_config.enabled {
            Self::create_pool_with_ssl(database_url, ssl_config).await?
        } else {
            PgPool::connect(database_url)
                .await
                .map_err(|e| StorageError::Query(e.to_string()))?
        };

        let store = Self {
            pool,
            collection_name,
            dimensions,
            ivfflat_lists,
            query_count: std::sync::atomic::AtomicI64::new(0),
            failed_query_count: std::sync::atomic::AtomicI64::new(0),
            total_query_time_ms: std::sync::atomic::AtomicU64::new(0),
        };

        store.initialize().await?;
        Ok(store)
    }

    async fn create_pool_with_ssl(
        database_url: &str,
        ssl_config: &SslConfig,
    ) -> StorageResult<PgPool> {
        let mut options = PgConnectOptions::from_str(database_url)
            .map_err(|e| StorageError::Connection(format!("Invalid database URL: {e}")))?;

        // Set SSL mode
        let ssl_mode = match ssl_config.mode.as_str() {
            "disable" => PgSslMode::Disable,
            "prefer" => PgSslMode::Prefer,
            "require" => PgSslMode::Require,
            "verify-ca" => PgSslMode::VerifyCa,
            "verify-full" => PgSslMode::VerifyFull,
            _ => PgSslMode::Prefer,
        };
        options = options.ssl_mode(ssl_mode);

        // Set SSL certificates if provided
        if let Some(root_cert) = &ssl_config.root_cert {
            options = options.ssl_root_cert(root_cert);
        }

        if let Some(client_cert) = &ssl_config.client_cert
            && let Some(client_key) = &ssl_config.client_key
        {
            options = options
                .ssl_client_cert(client_cert)
                .ssl_client_key(client_key);
        }

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(|e| StorageError::Connection(format!("Failed to connect with SSL: {e}")))?;

        Ok(pool)
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
            "CREATE INDEX IF NOT EXISTS {}_embedding_idx ON {} USING ivfflat (embedding vector_cosine_ops) WITH (lists = {})",
            self.collection_name, self.collection_name, self.ivfflat_lists
        );

        sqlx::query(&index_query)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        Ok(())
    }

    /// Track query execution time and update performance metrics
    fn track_query_performance(&self, duration: std::time::Duration, success: bool) {
        use std::sync::atomic::Ordering;

        let duration_ms = duration.as_secs_f64() * 1000.0;

        if success {
            self.query_count.fetch_add(1, Ordering::Relaxed);
            // Convert f64 to u64 by multiplying by 1000 to preserve precision
            let duration_micros = (duration_ms * 1000.0) as u64;
            self.total_query_time_ms
                .fetch_add(duration_micros, Ordering::Relaxed);
        } else {
            self.failed_query_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Execute a query with performance tracking
    async fn execute_tracked_query<F, T>(&self, operation: F) -> StorageResult<T>
    where
        F: std::future::Future<Output = StorageResult<T>>,
    {
        let start = Instant::now();
        let result = operation.await;
        let duration = start.elapsed();

        match &result {
            Ok(_) => self.track_query_performance(duration, true),
            Err(_) => self.track_query_performance(duration, false),
        }

        result
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
        let mut tx = self
            .pool
            .begin()
            .await
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

        tx.commit()
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        Ok(())
    }

    async fn search_similar(&self, query: SearchQuery) -> StorageResult<Vec<SearchResult>> {
        self.execute_tracked_query(async {
            let mut sql = format!(
                "SELECT id, content, metadata, embedding <=> $1 as distance, created_at FROM {} WHERE 1=1",
                self.collection_name
            );

            let mut bind_count = 1;

            // Add score threshold filter
            // Convert similarity threshold to distance threshold
            // similarity = 1 - distance, so distance = 1 - similarity
            if let Some(_threshold) = query.score_threshold {
                bind_count += 1;
                sql.push_str(&format!(" AND embedding <=> $1 < ${bind_count}"));
            }

            // Add metadata filter (simplified)
            if query.metadata_filter.is_some() {
                bind_count += 1;
                sql.push_str(&format!(" AND metadata @> ${bind_count}"));
            }

            sql.push_str(&format!(" ORDER BY embedding <=> $1 LIMIT {}", query.limit));

            let mut sqlx_query = sqlx::query(&sql).bind(&query.embedding);

            if let Some(threshold) = query.score_threshold {
                // Convert similarity threshold to distance threshold
                let distance_threshold = 1.0 - threshold;
                sqlx_query = sqlx_query.bind(distance_threshold);
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
        }).await
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

    async fn get_metrics(&self) -> StorageResult<VectorMetrics> {
        use std::sync::atomic::Ordering;

        // Get basic collection statistics
        let stats_query = format!(
            r#"
            SELECT 
                COUNT(*) as total_documents,
                pg_total_relation_size('{}') as total_size_bytes,
                pg_size_pretty(pg_total_relation_size('{}')) as total_size_human
            FROM {}
            "#,
            self.collection_name, self.collection_name, self.collection_name
        );

        let stats_row = sqlx::query(&stats_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        // Get index size
        let index_query = format!(
            r#"
            SELECT 
                pg_total_relation_size(indexrelid) as index_size_bytes
            FROM pg_stat_user_indexes 
            WHERE relname = '{}' AND indexrelname = '{}_embedding_idx'
            "#,
            self.collection_name, self.collection_name
        );

        let index_row = sqlx::query(&index_query)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        let index_size_bytes = index_row
            .map(|row| row.get::<i64, _>("index_size_bytes"))
            .unwrap_or(0);

        // Get index usage statistics
        let index_usage_query = format!(
            r#"
            SELECT 
                idx_scan as index_scans,
                idx_tup_read as index_tuples_read,
                idx_tup_fetch as index_tuples_fetched,
                idx_blks_read as index_blocks_read,
                idx_blks_hit as index_blocks_hit,
                CASE 
                    WHEN (idx_blks_read + idx_blks_hit) > 0 
                    THEN idx_blks_hit::float / (idx_blks_read + idx_blks_hit)::float
                    ELSE 0.0
                END as cache_hit_ratio
            FROM pg_stat_user_indexes 
            WHERE relname = '{}' AND indexrelname = '{}_embedding_idx'
            "#,
            self.collection_name, self.collection_name
        );

        let usage_row = sqlx::query(&index_usage_query)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        let index_usage = if let Some(row) = usage_row {
            IndexUsageStats {
                index_scans: row.get::<i64, _>("index_scans"),
                index_tuples_read: row.get::<i64, _>("index_tuples_read"),
                index_tuples_fetched: row.get::<i64, _>("index_tuples_fetched"),
                index_blocks_read: row.get::<i64, _>("index_blocks_read"),
                index_blocks_hit: row.get::<i64, _>("index_blocks_hit"),
                cache_hit_ratio: row.get::<f64, _>("cache_hit_ratio"),
            }
        } else {
            IndexUsageStats {
                index_scans: 0,
                index_tuples_read: 0,
                index_tuples_fetched: 0,
                index_blocks_read: 0,
                index_blocks_hit: 0,
                cache_hit_ratio: 0.0,
            }
        };

        // Get performance metrics from atomic counters
        let total_queries = self.query_count.load(Ordering::Relaxed);
        let failed_queries = self.failed_query_count.load(Ordering::Relaxed);
        let total_time_micros = self.total_query_time_ms.load(Ordering::Relaxed);

        // Calculate average query time
        let avg_query_time_ms = if total_queries > 0 {
            (total_time_micros as f64) / 1000.0 / (total_queries as f64)
        } else {
            0.0
        };

        // Get last query time (approximation - in real implementation you might want to store this)
        let last_query_time_ms = avg_query_time_ms;

        // Check last maintenance time
        let maintenance_query = format!(
            r#"
            SELECT last_vacuum, last_analyze 
            FROM pg_stat_user_tables 
            WHERE relname = '{}'
            "#,
            self.collection_name
        );

        let maintenance_row = sqlx::query(&maintenance_query)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        let last_maintenance = maintenance_row.and_then(|row| {
            let last_vacuum: Option<DateTime<Utc>> = row.get("last_vacuum");
            let last_analyze: Option<DateTime<Utc>> = row.get("last_analyze");

            match (last_vacuum, last_analyze) {
                (Some(vacuum), Some(analyze)) => Some(vacuum.max(analyze)),
                (Some(vacuum), None) => Some(vacuum),
                (None, Some(analyze)) => Some(analyze),
                (None, None) => None,
            }
        });

        Ok(VectorMetrics {
            total_documents: stats_row.get::<i64, _>("total_documents"),
            total_size_bytes: stats_row.get::<i64, _>("total_size_bytes"),
            index_size_bytes,
            dimensions: self.dimensions,
            collection_name: self.collection_name.clone(),
            last_query_time_ms,
            avg_query_time_ms,
            total_queries,
            failed_queries,
            last_maintenance,
            index_usage,
        })
    }

    async fn perform_maintenance(&self) -> StorageResult<()> {
        // Vacuum and analyze the vector table for optimal performance
        let vacuum_query = format!("VACUUM ANALYZE {}", self.collection_name);

        sqlx::query(&vacuum_query)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        // Reindex the vector index if needed (optional, can be resource-intensive)
        let reindex_query = format!("REINDEX INDEX {}_embedding_idx", self.collection_name);

        sqlx::query(&reindex_query)
            .execute(&self.pool)
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
    dimensions: usize,
    ivfflat_lists: u32,
) -> StorageResult<Box<dyn VectorStore>> {
    create_vector_store_with_ssl(
        provider,
        connection_url,
        collection_name,
        dimensions,
        ivfflat_lists,
        &SslConfig::default(),
    )
    .await
}

/// Create vector store instance with SSL configuration
pub async fn create_vector_store_with_ssl(
    provider: &str,
    connection_url: &str,
    collection_name: Option<String>,
    dimensions: usize,
    ivfflat_lists: u32,
    ssl_config: &SslConfig,
) -> StorageResult<Box<dyn VectorStore>> {
    match provider {
        "pgvector" => {
            let collection = collection_name.unwrap_or_else(|| "vector_documents".to_string());
            let store = PgVectorStore::new_with_ssl(
                connection_url,
                collection,
                dimensions,
                ivfflat_lists,
                ssl_config,
            )
            .await?;
            Ok(Box::new(store))
        }
        _ => Err(StorageError::Configuration(format!(
            "Unknown vector store provider: {provider}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_metrics_creation() {
        let index_usage = IndexUsageStats {
            index_scans: 100,
            index_tuples_read: 1000,
            index_tuples_fetched: 950,
            index_blocks_read: 50,
            index_blocks_hit: 450,
            cache_hit_ratio: 0.9,
        };

        let metrics = VectorMetrics {
            total_documents: 1000,
            total_size_bytes: 1024 * 1024 * 10, // 10MB
            index_size_bytes: 1024 * 1024 * 2,  // 2MB
            dimensions: 1536,
            collection_name: "test_collection".to_string(),
            last_query_time_ms: 15.5,
            avg_query_time_ms: 12.3,
            total_queries: 500,
            failed_queries: 5,
            last_maintenance: Some(Utc::now()),
            index_usage,
        };

        assert_eq!(metrics.total_documents, 1000);
        assert_eq!(metrics.dimensions, 1536);
        assert_eq!(metrics.collection_name, "test_collection");
        assert_eq!(metrics.index_usage.cache_hit_ratio, 0.9);
    }

    #[test]
    fn test_index_usage_stats() {
        let stats = IndexUsageStats {
            index_scans: 0,
            index_tuples_read: 0,
            index_tuples_fetched: 0,
            index_blocks_read: 0,
            index_blocks_hit: 0,
            cache_hit_ratio: 0.0,
        };

        // Test default/empty stats
        assert_eq!(stats.index_scans, 0);
        assert_eq!(stats.cache_hit_ratio, 0.0);
    }

    #[test]
    fn test_performance_calculations() {
        // Test query success rate calculation
        let total_queries = 100;
        let failed_queries = 5;
        let success_rate = if total_queries > 0 {
            1.0 - (failed_queries as f64 / total_queries as f64)
        } else {
            0.0
        };

        assert_eq!(success_rate, 0.95); // 95% success rate

        // Test size conversions
        let size_bytes = 10 * 1024 * 1024; // 10MB
        let size_mb = size_bytes as f64 / 1024.0 / 1024.0;
        assert_eq!(size_mb, 10.0);
    }
}
