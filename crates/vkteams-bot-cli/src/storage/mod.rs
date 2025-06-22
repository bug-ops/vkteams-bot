//! Storage module for VK Teams Bot CLI
//!
//! This module provides database and vector storage capabilities for saving
//! and searching VK Teams events with support for multiple backends.

#[cfg(feature = "database")]
pub mod models;
#[cfg(feature = "database")]
pub mod schema;
#[cfg(feature = "database")]
pub mod relational;

#[cfg(feature = "vector-search")]
pub mod vector;

#[cfg(feature = "ai-embeddings")]
pub mod embedding;

pub mod manager;
pub mod error;

pub use error::{StorageError, StorageResult};
pub use manager::StorageManager;

#[cfg(feature = "database")]
pub use models::*;

#[cfg(feature = "vector-search")]
pub use vector::{VectorStore, VectorDocument, SearchQuery, SearchResult};