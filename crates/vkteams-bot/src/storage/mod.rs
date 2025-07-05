//! Storage module for VK Teams Bot events and context
//!
//! This module provides database and vector storage capabilities for saving
//! and searching VK Teams events with support for multiple backends.
//!
//! # Features
//!
//! - **Relational storage**: PostgreSQL with full ACID compliance
//! - **Vector search**: pgvector for semantic similarity search  
//! - **AI embeddings**: OpenAI and local models support
//! - **Event processing**: Automatic storage with configurable pipelines
//!
//! # Example
//!
//! ```rust,no_run
//! use vkteams_bot::{Bot, storage::{StorageManager, StorageConfig}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let bot = Bot::new(/* config */);
//!     
//!     // Setup storage
//!     let storage_config = StorageConfig {
//!         database_url: "postgresql://localhost/vkteams".to_string(),
//!         auto_migrate: true,
//!         // ... other config
//!     };
//!     
//!     let storage = StorageManager::new(&storage_config).await?;
//!     
//!     // Listen and store events automatically
//!     bot.get_events_by_poll(|event| {
//!         let storage = storage.clone();
//!         async move {
//!             storage.process_event(&event).await?;
//!             println!("Stored event: {}", event.event_id);
//!             Ok(())
//!         }
//!     }).await?;
//!     
//!     Ok(())
//! }
//! ```

#[cfg(feature = "storage")]
pub mod config;
#[cfg(feature = "storage")]
pub mod error;
#[cfg(feature = "storage")]
pub mod models;
// Temporarily disabled complex relational module
// TODO: Revisit later
// #[cfg(feature = "storage")]
// pub mod relational;
#[cfg(feature = "storage")]
pub mod manager;
#[cfg(feature = "storage")]
pub mod simple;

#[cfg(feature = "vector-search")]
pub mod vector;

#[cfg(feature = "ai-embeddings")]
pub mod embedding;

#[cfg(test)]
mod ssl_tests;
#[cfg(test)]
mod tests;

#[cfg(feature = "storage")]
pub use config::StorageConfig;
#[cfg(feature = "storage")]
pub use error::{StorageError, StorageResult};
#[cfg(feature = "storage")]
pub use manager::StorageManager;
#[cfg(feature = "storage")]
pub use models::*;

#[cfg(feature = "vector-search")]
pub use vector::{SearchQuery, SearchResult, VectorDocument, VectorStore};
