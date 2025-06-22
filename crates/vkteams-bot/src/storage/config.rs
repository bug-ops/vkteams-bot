//! Storage configuration types

use serde::{Deserialize, Serialize};

/// Complete storage configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageConfig {
    /// Database configuration
    pub database: DatabaseConfig,

    /// Vector storage configuration (optional)
    #[cfg(feature = "vector-search")]
    pub vector: Option<VectorConfig>,

    /// Embedding generation configuration (optional)
    #[cfg(feature = "ai-embeddings")]
    pub embedding: Option<EmbeddingConfig>,

    /// General storage settings
    pub settings: StorageSettings,
}

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL
    pub url: String,

    /// Maximum number of connections in pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,

    /// Auto-run migrations on startup
    #[serde(default = "default_auto_migrate")]
    pub auto_migrate: bool,
}

/// Vector storage configuration
#[cfg(feature = "vector-search")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorConfig {
    /// Vector store provider ("pgvector", "qdrant")
    #[serde(default = "default_vector_provider")]
    pub provider: String,

    /// Connection URL for vector store
    pub connection_url: String,

    /// Collection/table name for vectors
    #[serde(default = "default_collection_name")]
    pub collection_name: String,

    /// Embedding dimensions
    #[serde(default = "default_dimensions")]
    pub dimensions: usize,

    /// Minimum similarity threshold for search
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f32,

    /// IVFFlat index lists (pgvector specific)
    #[serde(default = "default_ivfflat_lists")]
    pub ivfflat_lists: u32,
}

/// Embedding generation configuration
#[cfg(feature = "ai-embeddings")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddingConfig {
    /// Embedding provider ("openai", "ollama")
    #[serde(default = "default_embedding_provider")]
    pub provider: String,

    /// Model name for embeddings
    #[serde(default = "default_embedding_model")]
    pub model: String,

    /// Environment variable name for API key (OpenAI only)
    #[serde(default = "default_api_key_env")]
    pub api_key_env: String,

    /// Ollama host (for ollama provider)
    #[serde(default = "default_ollama_host")]
    pub ollama_host: String,

    /// Ollama port (for ollama provider)
    #[serde(default = "default_ollama_port")]
    pub ollama_port: u16,

    /// Custom embedding dimensions (override model defaults)
    pub custom_dimensions: Option<usize>,

    /// Batch size for embedding generation
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// Auto-generate embeddings for new events
    #[serde(default = "default_auto_generate")]
    pub auto_generate: bool,
}

/// General storage settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageSettings {
    /// Event retention period in days
    #[serde(default = "default_retention_days")]
    pub event_retention_days: u32,

    /// Cleanup interval in hours
    #[serde(default = "default_cleanup_interval")]
    pub cleanup_interval_hours: u32,

    /// Batch size for event processing
    #[serde(default = "default_processing_batch_size")]
    pub batch_size: usize,

    /// Maximum events to keep in memory
    #[serde(default = "default_max_memory_events")]
    pub max_memory_events: usize,
}

// Default value functions
fn default_max_connections() -> u32 {
    20
}
fn default_connection_timeout() -> u64 {
    30
}
fn default_auto_migrate() -> bool {
    true
}

#[cfg(feature = "vector-search")]
fn default_vector_provider() -> String {
    "pgvector".to_string()
}
#[cfg(feature = "vector-search")]
fn default_collection_name() -> String {
    "embeddings".to_string()
}
#[cfg(feature = "vector-search")]
fn default_dimensions() -> usize {
    1536
}
#[cfg(feature = "vector-search")]
fn default_similarity_threshold() -> f32 {
    0.8
}
#[cfg(feature = "vector-search")]
fn default_ivfflat_lists() -> u32 {
    100
}

#[cfg(feature = "ai-embeddings")]
fn default_embedding_provider() -> String {
    "openai".to_string()
}
#[cfg(feature = "ai-embeddings")]
fn default_embedding_model() -> String {
    "text-embedding-ada-002".to_string()
}
#[cfg(feature = "ai-embeddings")]
fn default_api_key_env() -> String {
    "OPENAI_API_KEY".to_string()
}
#[cfg(feature = "ai-embeddings")]
fn default_ollama_host() -> String {
    "localhost".to_string()
}
#[cfg(feature = "ai-embeddings")]
fn default_ollama_port() -> u16 {
    11434
}
#[cfg(feature = "ai-embeddings")]
fn default_batch_size() -> usize {
    50
}
#[cfg(feature = "ai-embeddings")]
fn default_auto_generate() -> bool {
    true
}

fn default_retention_days() -> u32 {
    365
}
fn default_cleanup_interval() -> u32 {
    24
}
fn default_processing_batch_size() -> usize {
    100
}
fn default_max_memory_events() -> usize {
    10000
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: "postgresql://localhost/vkteams_bot".to_string(),
                max_connections: default_max_connections(),
                connection_timeout: default_connection_timeout(),
                auto_migrate: default_auto_migrate(),
            },
            #[cfg(feature = "vector-search")]
            vector: Some(VectorConfig {
                provider: default_vector_provider(),
                connection_url: "postgresql://localhost/vkteams_bot".to_string(),
                collection_name: default_collection_name(),
                dimensions: default_dimensions(),
                similarity_threshold: default_similarity_threshold(),
                ivfflat_lists: default_ivfflat_lists(),
            }),
            #[cfg(feature = "ai-embeddings")]
            embedding: Some(EmbeddingConfig {
                provider: default_embedding_provider(),
                model: default_embedding_model(),
                api_key_env: default_api_key_env(),
                ollama_host: default_ollama_host(),
                ollama_port: default_ollama_port(),
                custom_dimensions: None,
                batch_size: default_batch_size(),
                auto_generate: default_auto_generate(),
            }),
            settings: StorageSettings {
                event_retention_days: default_retention_days(),
                cleanup_interval_hours: default_cleanup_interval(),
                batch_size: default_processing_batch_size(),
                max_memory_events: default_max_memory_events(),
            },
        }
    }
}
