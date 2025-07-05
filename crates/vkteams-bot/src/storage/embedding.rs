//! AI embedding generation for text content

use crate::storage::{StorageError, StorageResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ai-embeddings")]
use rayon::prelude::*;

/// Configuration for embedding providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingProviderConfig {
    OpenAI {
        api_key: String,
        model: String,
    },
    Ollama {
        host: String,
        port: u16,
        model: String,
        dimensions: Option<usize>,
    },
}

/// Trait for embedding clients
#[async_trait]
pub trait EmbeddingClient: Send + Sync {
    /// Generate embedding for a single text
    async fn generate_embedding(
        &self,
        text: &str,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>>;

    /// Generate embeddings for multiple texts
    async fn generate_embeddings(
        &self,
        texts: &[String],
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>>;

    /// Generate embeddings for multiple texts with parallel processing
    async fn generate_embeddings_parallel(
        &self,
        texts: &[String],
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        // Default implementation falls back to sequential processing
        self.generate_embeddings(texts).await
    }

    /// Batch generate embeddings with configurable chunk size for optimal performance
    async fn generate_embeddings_chunked(
        &self,
        texts: &[String],
        chunk_size: usize,
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        // Default implementation processes in chunks sequentially
        let mut all_embeddings = Vec::new();

        for chunk in texts.chunks(chunk_size) {
            let chunk_embeddings = self.generate_embeddings(chunk).await?;
            all_embeddings.extend(chunk_embeddings);
        }

        Ok(all_embeddings)
    }

    /// Health check
    async fn health_check(&self) -> StorageResult<()>;
}

/// OpenAI embedding client
#[cfg(feature = "ai-embeddings")]
#[derive(Clone)]
pub struct OpenAIEmbeddingClient {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

#[cfg(feature = "ai-embeddings")]
impl OpenAIEmbeddingClient {
    pub fn new(api_key: String, model: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            api_key,
            model,
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }
}

#[cfg(feature = "ai-embeddings")]
#[async_trait]
impl EmbeddingClient for OpenAIEmbeddingClient {
    async fn generate_embedding(
        &self,
        text: &str,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        use serde_json::json;

        let request_body = json!({
            "model": self.model,
            "input": text
        });

        let response = self
            .client
            .post(format!("{}/embeddings", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;

        if let Some(data) = response_json["data"].as_array() {
            if let Some(first_embedding) = data.first() {
                if let Some(embedding_array) = first_embedding["embedding"].as_array() {
                    let embedding: Result<Vec<f32>, _> = embedding_array
                        .iter()
                        .map(|v| {
                            v.as_f64()
                                .map(|f| f as f32)
                                .ok_or("Invalid embedding value")
                        })
                        .collect();
                    return Ok(embedding?);
                }
            }
        }

        Err("No embedding returned from OpenAI".into())
    }

    async fn generate_embeddings(
        &self,
        texts: &[String],
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        use serde_json::json;

        let request_body = json!({
            "model": self.model,
            "input": texts
        });

        let response = self
            .client
            .post(format!("{}/embeddings", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;

        if let Some(data) = response_json["data"].as_array() {
            // Parallel processing of embedding data parsing
            let embeddings: Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> = data
                .par_iter()
                .map(
                    |item| -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
                        if let Some(embedding_array) = item["embedding"].as_array() {
                            let embedding: Result<Vec<f32>, _> = embedding_array
                                .par_iter()
                                .map(|v| {
                                    v.as_f64()
                                        .map(|f| f as f32)
                                        .ok_or("Invalid embedding value")
                                })
                                .collect();
                            Ok(embedding?)
                        } else {
                            Err("Missing embedding array".into())
                        }
                    },
                )
                .collect();

            embeddings
        } else {
            Err("No embeddings returned from OpenAI".into())
        }
    }

    async fn generate_embeddings_parallel(
        &self,
        texts: &[String],
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        // For OpenAI, we can leverage their batch API which already handles parallelization efficiently
        // so we fall back to the standard batch method but with optimized processing
        self.generate_embeddings(texts).await
    }

    async fn generate_embeddings_chunked(
        &self,
        texts: &[String],
        chunk_size: usize,
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        use futures::future::try_join_all;

        // Process chunks in parallel using async tasks
        let chunks: Vec<_> = texts.chunks(chunk_size).collect();

        let chunk_futures = chunks.into_iter().map(|chunk| {
            let chunk_texts: Vec<String> = chunk.to_vec();
            let client = self.clone();

            async move { client.generate_embeddings(&chunk_texts).await }
        });

        let chunk_results = try_join_all(chunk_futures).await?;

        // Flatten results from all chunks
        let mut all_embeddings = Vec::new();
        for chunk_embeddings in chunk_results {
            all_embeddings.extend(chunk_embeddings);
        }

        Ok(all_embeddings)
    }

    async fn health_check(&self) -> StorageResult<()> {
        // Simple test with a minimal text
        self.generate_embedding("test")
            .await
            .map_err(|e| StorageError::Embedding(e.to_string()))?;
        Ok(())
    }
}

/// Ollama embedding client
#[cfg(feature = "ai-embeddings")]
#[derive(Clone)]
pub struct OllamaEmbeddingClient {
    client: ollama_rs::Ollama,
    model: String,
}

#[cfg(feature = "ai-embeddings")]
impl OllamaEmbeddingClient {
    pub fn new(host: String, port: u16, model: String) -> Self {
        let client = ollama_rs::Ollama::new(host, port);
        Self { client, model }
    }
}

#[cfg(feature = "ai-embeddings")]
#[async_trait]
impl EmbeddingClient for OllamaEmbeddingClient {
    async fn generate_embedding(
        &self,
        text: &str,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        let request = ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest::new(
            self.model.clone(),
            ollama_rs::generation::embeddings::request::EmbeddingsInput::Single(text.to_string()),
        );

        let response = self.client.generate_embeddings(request).await?;
        if let Some(embedding) = response.embeddings.first() {
            Ok(embedding.clone())
        } else {
            Err("No embedding returned from Ollama".into())
        }
    }

    async fn generate_embeddings(
        &self,
        texts: &[String],
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        // Use parallel processing for multiple embeddings
        use futures::future::try_join_all;

        let embedding_futures = texts.iter().map(|text| {
            let client = self.clone();
            let text = text.clone();
            async move { client.generate_embedding(&text).await }
        });

        let embeddings = try_join_all(embedding_futures).await?;
        Ok(embeddings)
    }

    async fn generate_embeddings_parallel(
        &self,
        texts: &[String],
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        // For Ollama, we use controlled concurrency to not overwhelm the local server
        use futures::stream::{self, StreamExt};

        const MAX_CONCURRENT: usize = 4; // Limit concurrent requests for local Ollama

        let owned_texts: Vec<String> = texts.to_vec();
        let results: Result<Vec<_>, _> = stream::iter(owned_texts.into_iter())
            .map(|text| {
                let client = self.clone();
                async move { client.generate_embedding(&text).await }
            })
            .buffer_unordered(MAX_CONCURRENT)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect();

        results
    }

    async fn generate_embeddings_chunked(
        &self,
        texts: &[String],
        chunk_size: usize,
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        use futures::future::try_join_all;

        // Process chunks with limited concurrency for Ollama
        let chunks: Vec<_> = texts.chunks(chunk_size).collect();

        let chunk_futures = chunks.into_iter().map(|chunk| {
            let chunk_texts: Vec<String> = chunk.to_vec();
            let client = self.clone();

            async move { client.generate_embeddings_parallel(&chunk_texts).await }
        });

        let chunk_results = try_join_all(chunk_futures).await?;

        // Flatten results from all chunks
        let mut all_embeddings = Vec::new();
        for chunk_embeddings in chunk_results {
            all_embeddings.extend(chunk_embeddings);
        }

        Ok(all_embeddings)
    }

    async fn health_check(&self) -> StorageResult<()> {
        // Simple test with a minimal text
        self.generate_embedding("test")
            .await
            .map_err(|e| StorageError::Embedding(e.to_string()))?;
        Ok(())
    }
}

/// Create embedding client instance
#[cfg(feature = "ai-embeddings")]
pub async fn create_embedding_client(
    config: EmbeddingProviderConfig,
) -> StorageResult<Box<dyn EmbeddingClient>> {
    match config {
        EmbeddingProviderConfig::OpenAI { api_key, model } => {
            let client = OpenAIEmbeddingClient::new(api_key, model);
            Ok(Box::new(client))
        }
        EmbeddingProviderConfig::Ollama {
            host, port, model, ..
        } => {
            let client = OllamaEmbeddingClient::new(host, port, model);
            Ok(Box::new(client))
        }
    }
}

/// Fallback when ai-embeddings feature is disabled
#[cfg(not(feature = "ai-embeddings"))]
pub async fn create_embedding_client(
    _config: EmbeddingProviderConfig,
) -> StorageResult<Box<dyn EmbeddingClient>> {
    Err(StorageError::Configuration(
        "AI embeddings feature is not enabled".to_string(),
    ))
}

/// High-level embedding processing utilities
#[cfg(feature = "ai-embeddings")]
pub mod processing {
    use super::*;

    /// Configuration for parallel embedding processing
    #[derive(Debug, Clone)]
    pub struct ProcessingConfig {
        pub chunk_size: usize,
        pub max_concurrent_chunks: usize,
        pub provider_specific_settings: ProviderSettings,
    }

    #[derive(Debug, Clone)]
    pub enum ProviderSettings {
        OpenAI { max_tokens_per_batch: usize },
        Ollama { max_concurrent_requests: usize },
    }

    impl Default for ProcessingConfig {
        fn default() -> Self {
            Self {
                chunk_size: 50,
                max_concurrent_chunks: 3,
                provider_specific_settings: ProviderSettings::OpenAI {
                    max_tokens_per_batch: 2048,
                },
            }
        }
    }

    /// Process large batches of texts with optimal performance
    pub async fn process_large_batch(
        client: &dyn EmbeddingClient,
        texts: &[String],
        config: &ProcessingConfig,
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        // Filter empty texts to avoid API errors
        let non_empty_texts: Vec<_> = texts
            .par_iter()
            .enumerate()
            .filter(|(_, text)| !text.trim().is_empty())
            .collect();

        if non_empty_texts.is_empty() {
            return Ok(vec![]);
        }

        // Process based on provider type and size
        match (config.provider_specific_settings.clone(), texts.len()) {
            (ProviderSettings::OpenAI { .. }, n) if n > config.chunk_size => {
                // Use chunked processing for large batches
                client
                    .generate_embeddings_chunked(texts, config.chunk_size)
                    .await
            }
            (ProviderSettings::Ollama { .. }, n) if n > 10 => {
                // Use parallel processing for medium batches
                client.generate_embeddings_parallel(texts).await
            }
            _ => {
                // Use standard processing for small batches
                client.generate_embeddings(texts).await
            }
        }
    }

    /// Estimate optimal chunk size based on text lengths and provider
    pub fn estimate_optimal_chunk_size(texts: &[String], provider: &ProviderSettings) -> usize {
        if texts.is_empty() {
            return 50;
        }

        // Calculate average text length
        let avg_length: f64 =
            texts.par_iter().map(|text| text.len()).sum::<usize>() as f64 / texts.len() as f64;

        match provider {
            ProviderSettings::OpenAI {
                max_tokens_per_batch,
            } => {
                // Estimate ~4 characters per token
                let est_tokens_per_text = (avg_length / 4.0).ceil() as usize;
                (max_tokens_per_batch / est_tokens_per_text).clamp(1, 100)
            }
            ProviderSettings::Ollama {
                max_concurrent_requests,
            } => {
                // For Ollama, limit based on concurrent requests
                (*max_concurrent_requests * 5).min(50)
            }
        }
    }

    /// Create adaptive processing configuration based on data characteristics
    pub fn create_adaptive_config(
        texts: &[String],
        provider_config: &EmbeddingProviderConfig,
    ) -> ProcessingConfig {
        let provider_settings = match provider_config {
            EmbeddingProviderConfig::OpenAI { .. } => ProviderSettings::OpenAI {
                max_tokens_per_batch: 8000, // Conservative estimate for ada-002
            },
            EmbeddingProviderConfig::Ollama { .. } => ProviderSettings::Ollama {
                max_concurrent_requests: 4, // Don't overwhelm local server
            },
        };

        let optimal_chunk_size = estimate_optimal_chunk_size(texts, &provider_settings);

        ProcessingConfig {
            chunk_size: optimal_chunk_size,
            max_concurrent_chunks: 3,
            provider_specific_settings: provider_settings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "ai-embeddings")]
    use super::processing::*;

    #[test]
    fn test_embedding_provider_config_debug() {
        let openai_config = EmbeddingProviderConfig::OpenAI {
            api_key: "test_key".to_string(),
            model: "text-embedding-ada-002".to_string(),
        };

        let debug_str = format!("{openai_config:?}");
        assert!(debug_str.contains("OpenAI"));
        assert!(debug_str.contains("test_key"));

        let ollama_config = EmbeddingProviderConfig::Ollama {
            host: "localhost".to_string(),
            port: 11434,
            model: "nomic-embed-text".to_string(),
            dimensions: Some(768),
        };

        let debug_str = format!("{ollama_config:?}");
        assert!(debug_str.contains("Ollama"));
        assert!(debug_str.contains("localhost"));
    }

    #[test]
    fn test_embedding_provider_config_clone() {
        let original = EmbeddingProviderConfig::OpenAI {
            api_key: "original_key".to_string(),
            model: "original_model".to_string(),
        };

        let cloned = original.clone();

        match (original, cloned) {
            (
                EmbeddingProviderConfig::OpenAI {
                    api_key: orig_key, ..
                },
                EmbeddingProviderConfig::OpenAI {
                    api_key: clone_key, ..
                },
            ) => {
                assert_eq!(orig_key, clone_key);
            }
            _ => panic!("Clone should maintain the same variant"),
        }
    }

    #[test]
    fn test_embedding_provider_config_serialization() {
        let config = EmbeddingProviderConfig::OpenAI {
            api_key: "test_key_123".to_string(),
            model: "text-embedding-ada-002".to_string(),
        };

        // Test serialization
        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("OpenAI"));
        assert!(serialized.contains("test_key_123"));

        // Test deserialization
        let deserialized: EmbeddingProviderConfig = serde_json::from_str(&serialized).unwrap();
        match deserialized {
            EmbeddingProviderConfig::OpenAI { api_key, model } => {
                assert_eq!(api_key, "test_key_123");
                assert_eq!(model, "text-embedding-ada-002");
            }
            _ => panic!("Deserialized to wrong variant"),
        }
    }

    #[cfg(feature = "ai-embeddings")]
    #[tokio::test]
    async fn test_processing_config_default() {
        let config = ProcessingConfig::default();
        assert_eq!(config.chunk_size, 50);
        assert_eq!(config.max_concurrent_chunks, 3);
    }

    #[test]
    fn test_estimate_optimal_chunk_size_openai() {
        let texts = vec![
            "Short text".to_string(),
            "A longer text with more words and content".to_string(),
            "Medium length text".to_string(),
        ];

        let provider = ProviderSettings::OpenAI {
            max_tokens_per_batch: 2000,
        };

        let chunk_size = estimate_optimal_chunk_size(&texts, &provider);
        assert!(chunk_size > 0);
        assert!(chunk_size <= 100);
    }

    #[test]
    fn test_estimate_optimal_chunk_size_ollama() {
        let texts = vec!["Test text 1".to_string(), "Test text 2".to_string()];

        let provider = ProviderSettings::Ollama {
            max_concurrent_requests: 4,
        };

        let chunk_size = estimate_optimal_chunk_size(&texts, &provider);
        assert!(chunk_size > 0);
        assert!(chunk_size <= 50);
    }

    #[test]
    fn test_create_adaptive_config_openai() {
        let texts = vec!["test".to_string(); 100];
        let provider_config = EmbeddingProviderConfig::OpenAI {
            api_key: "test".to_string(),
            model: "text-embedding-ada-002".to_string(),
        };

        let config = create_adaptive_config(&texts, &provider_config);
        assert!(config.chunk_size > 0);

        match config.provider_specific_settings {
            ProviderSettings::OpenAI {
                max_tokens_per_batch,
            } => {
                assert_eq!(max_tokens_per_batch, 8000);
            }
            _ => panic!("Expected OpenAI settings"),
        }
    }

    #[test]
    fn test_create_adaptive_config_ollama() {
        let texts = vec!["test".to_string(); 20];
        let provider_config = EmbeddingProviderConfig::Ollama {
            host: "localhost".to_string(),
            port: 11434,
            model: "nomic-embed-text".to_string(),
            dimensions: Some(768),
        };

        let config = create_adaptive_config(&texts, &provider_config);
        assert!(config.chunk_size > 0);

        match config.provider_specific_settings {
            ProviderSettings::Ollama {
                max_concurrent_requests,
            } => {
                assert_eq!(max_concurrent_requests, 4);
            }
            _ => panic!("Expected Ollama settings"),
        }
    }

    #[test]
    fn test_parallel_processing_filter_empty_texts() {
        let texts = vec![
            "Valid text".to_string(),
            "".to_string(),
            "   ".to_string(), // Only whitespace
            "Another valid text".to_string(),
        ];

        let non_empty: Vec<_> = texts
            .par_iter()
            .enumerate()
            .filter(|(_, text)| !text.trim().is_empty())
            .collect();

        assert_eq!(non_empty.len(), 2);
        assert_eq!(non_empty[0].0, 0); // First valid text at index 0
        assert_eq!(non_empty[1].0, 3); // Second valid text at index 3
    }

    // Mock implementation for testing trait methods
    struct MockEmbeddingClient;

    #[async_trait]
    impl EmbeddingClient for MockEmbeddingClient {
        async fn generate_embedding(
            &self,
            text: &str,
        ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
            // Return mock embedding based on text length
            let embedding_size = text.len().min(10); // Limit to reasonable size
            Ok(vec![0.1; embedding_size])
        }

        async fn generate_embeddings(
            &self,
            texts: &[String],
        ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
            let mut embeddings = Vec::new();
            for text in texts {
                embeddings.push(self.generate_embedding(text).await?);
            }
            Ok(embeddings)
        }

        async fn health_check(&self) -> StorageResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_embedding_client_trait_single() {
        let client = MockEmbeddingClient;

        let result = client.generate_embedding("test text").await;
        assert!(result.is_ok());

        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 9); // "test text" has 9 characters
        assert!(embedding.iter().all(|&x| x == 0.1));
    }

    #[tokio::test]
    async fn test_embedding_client_trait_multiple() {
        let client = MockEmbeddingClient;

        let texts = vec![
            "short".to_string(),
            "medium length text".to_string(),
            "a".to_string(),
        ];

        let result = client.generate_embeddings(&texts).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0].len(), 5); // "short"
        assert_eq!(embeddings[1].len(), 10); // Capped at 10
        assert_eq!(embeddings[2].len(), 1); // "a"
    }

    #[tokio::test]
    async fn test_embedding_client_trait_parallel_default() {
        let client = MockEmbeddingClient;

        let texts = vec!["test1".to_string(), "test2".to_string()];

        let result = client.generate_embeddings_parallel(&texts).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 5); // "test1"
        assert_eq!(embeddings[1].len(), 5); // "test2"
    }

    #[tokio::test]
    async fn test_embedding_client_trait_chunked() {
        let client = MockEmbeddingClient;

        let texts = vec![
            "chunk1".to_string(),
            "chunk2".to_string(),
            "chunk3".to_string(),
            "chunk4".to_string(),
        ];

        let result = client.generate_embeddings_chunked(&texts, 2).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 4);
        // Verify all embeddings are generated
        for embedding in embeddings.iter() {
            assert_eq!(embedding.len(), 6); // "chunk1".len() = 6
            assert!(embedding.iter().all(|&x| x == 0.1));
        }
    }

    #[tokio::test]
    async fn test_embedding_client_trait_chunked_single_chunk() {
        let client = MockEmbeddingClient;

        let texts = vec!["single".to_string()];

        let result = client.generate_embeddings_chunked(&texts, 5).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].len(), 6); // "single"
    }

    #[tokio::test]
    async fn test_embedding_client_trait_health_check() {
        let client = MockEmbeddingClient;

        let result = client.health_check().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_embedding_client_trait_empty_text() {
        let client = MockEmbeddingClient;

        let result = client.generate_embedding("").await;
        assert!(result.is_ok());

        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 0); // Empty text = 0 length
    }

    #[tokio::test]
    async fn test_embedding_client_trait_empty_texts_list() {
        let client = MockEmbeddingClient;

        let texts: Vec<String> = vec![];

        let result = client.generate_embeddings(&texts).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 0);
    }

    #[cfg(feature = "ai-embeddings")]
    #[test]
    fn test_openai_embedding_client_new() {
        let client = OpenAIEmbeddingClient::new(
            "test_api_key".to_string(),
            "text-embedding-ada-002".to_string(),
        );

        assert_eq!(client.api_key, "test_api_key");
        assert_eq!(client.model, "text-embedding-ada-002");
        assert_eq!(client.base_url, "https://api.openai.com/v1");
    }

    #[cfg(feature = "ai-embeddings")]
    #[test]
    fn test_openai_embedding_client_clone() {
        let client =
            OpenAIEmbeddingClient::new("original_key".to_string(), "original_model".to_string());

        let cloned = client.clone();

        assert_eq!(client.api_key, cloned.api_key);
        assert_eq!(client.model, cloned.model);
        assert_eq!(client.base_url, cloned.base_url);
    }

    #[test]
    fn test_embedding_provider_config_ollama_variant() {
        let config = EmbeddingProviderConfig::Ollama {
            host: "127.0.0.1".to_string(),
            port: 8080,
            model: "custom-model".to_string(),
            dimensions: None,
        };

        match config {
            EmbeddingProviderConfig::Ollama {
                host,
                port,
                model,
                dimensions,
            } => {
                assert_eq!(host, "127.0.0.1");
                assert_eq!(port, 8080);
                assert_eq!(model, "custom-model");
                assert!(dimensions.is_none());
            }
            _ => panic!("Expected Ollama variant"),
        }
    }

    #[test]
    fn test_embedding_provider_config_ollama_with_dimensions() {
        let config = EmbeddingProviderConfig::Ollama {
            host: "localhost".to_string(),
            port: 11434,
            model: "nomic-embed-text".to_string(),
            dimensions: Some(1024),
        };

        match config {
            EmbeddingProviderConfig::Ollama {
                dimensions: Some(dims),
                ..
            } => {
                assert_eq!(dims, 1024);
            }
            _ => panic!("Expected Ollama variant with dimensions"),
        }
    }

    #[test]
    fn test_embedding_provider_config_openai_variant() {
        let config = EmbeddingProviderConfig::OpenAI {
            api_key: "sk-test123".to_string(),
            model: "text-embedding-3-small".to_string(),
        };

        match config {
            EmbeddingProviderConfig::OpenAI { api_key, model } => {
                assert_eq!(api_key, "sk-test123");
                assert_eq!(model, "text-embedding-3-small");
            }
            _ => panic!("Expected OpenAI variant"),
        }
    }

    #[test]
    fn test_embedding_provider_config_deserialization_error() {
        let invalid_json = r#"{"unknown_variant": {"key": "value"}}"#;

        let result: Result<EmbeddingProviderConfig, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_embedding_provider_config_ollama_serialization() {
        let config = EmbeddingProviderConfig::Ollama {
            host: "test-host".to_string(),
            port: 9999,
            model: "test-model".to_string(),
            dimensions: Some(512),
        };

        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("Ollama"));
        assert!(serialized.contains("test-host"));
        assert!(serialized.contains("9999"));
        assert!(serialized.contains("512"));

        let deserialized: EmbeddingProviderConfig = serde_json::from_str(&serialized).unwrap();
        match deserialized {
            EmbeddingProviderConfig::Ollama {
                host,
                port,
                model,
                dimensions,
            } => {
                assert_eq!(host, "test-host");
                assert_eq!(port, 9999);
                assert_eq!(model, "test-model");
                assert_eq!(dimensions, Some(512));
            }
            _ => panic!("Deserialized to wrong variant"),
        }
    }
}
