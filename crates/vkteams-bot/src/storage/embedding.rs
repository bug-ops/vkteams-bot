//! AI embedding generation for text content

use crate::storage::{StorageError, StorageResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>>;

    /// Generate embeddings for multiple texts
    async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>>;

    /// Health check
    async fn health_check(&self) -> StorageResult<()>;
}

/// OpenAI embedding client
#[cfg(feature = "ai-embeddings")]
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
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        use serde_json::json;
        
        let request_body = json!({
            "model": self.model,
            "input": text
        });

        let response = self.client
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
                        .map(|v| v.as_f64().map(|f| f as f32).ok_or("Invalid embedding value"))
                        .collect();
                    return Ok(embedding?);
                }
            }
        }
        
        Err("No embedding returned from OpenAI".into())
    }

    async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        use serde_json::json;
        
        let request_body = json!({
            "model": self.model,
            "input": texts
        });

        let response = self.client
            .post(format!("{}/embeddings", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;
        
        if let Some(data) = response_json["data"].as_array() {
            let mut embeddings = Vec::new();
            for item in data {
                if let Some(embedding_array) = item["embedding"].as_array() {
                    let embedding: Result<Vec<f32>, _> = embedding_array
                        .iter()
                        .map(|v| v.as_f64().map(|f| f as f32).ok_or("Invalid embedding value"))
                        .collect();
                    embeddings.push(embedding?);
                }
            }
            Ok(embeddings)
        } else {
            Err("No embeddings returned from OpenAI".into())
        }
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
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
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

    async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut embeddings = Vec::new();
        
        for text in texts {
            let embedding = self.generate_embedding(text).await?;
            embeddings.push(embedding);
        }
        
        Ok(embeddings)
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
        EmbeddingProviderConfig::Ollama { host, port, model, .. } => {
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
        "AI embeddings feature is not enabled".to_string()
    ))
}