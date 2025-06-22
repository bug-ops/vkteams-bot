//! Database models for VK Teams Bot events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "database")]
use sqlx::FromRow;

#[cfg(feature = "vector-search")]
use pgvector::Vector;

/// Event record in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(FromRow))]
pub struct Event {
    pub id: i64,
    pub event_id: String,
    pub event_type: String,
    pub chat_id: String,
    pub user_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub raw_payload: serde_json::Value,
    pub processed_data: Option<serde_json::Value>,
    pub embedding_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Message record with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(FromRow))]
pub struct Message {
    pub id: i64,
    pub event_id: i64,
    pub message_id: String,
    pub chat_id: String,
    pub user_id: String,
    pub text: Option<String>,
    pub formatted_text: Option<String>,
    pub reply_to_message_id: Option<String>,
    pub forward_from_chat_id: Option<String>,
    pub forward_from_message_id: Option<String>,
    pub file_attachments: Option<serde_json::Value>,
    pub has_mentions: bool,
    pub mentions: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Context record for MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(FromRow))]
pub struct Context {
    pub id: Uuid,
    pub chat_id: String,
    pub context_type: String,
    pub summary: String,
    pub key_points: Option<serde_json::Value>,
    pub related_events: Option<serde_json::Value>,
    pub relevance_score: f64,
    pub valid_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Embedding record for vector search
#[cfg(feature = "vector-search")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub id: Uuid,
    pub event_id: i64,
    pub content_type: String,
    pub text_content: String,
    pub embedding: Vector,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Statistics about stored events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStats {
    pub total_events: i64,
    pub total_messages: i64,
    pub unique_chats: i64,
    pub unique_users: i64,
    pub events_last_24h: i64,
    pub events_last_week: i64,
    pub oldest_event: Option<DateTime<Utc>>,
    pub newest_event: Option<DateTime<Utc>>,
    pub storage_size_bytes: Option<i64>,
}

/// Input structs for creating new records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEvent {
    pub event_id: String,
    pub event_type: String,
    pub chat_id: String,
    pub user_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub raw_payload: serde_json::Value,
    pub processed_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMessage {
    pub event_id: i64,
    pub message_id: String,
    pub chat_id: String,
    pub user_id: String,
    pub text: Option<String>,
    pub formatted_text: Option<String>,
    pub reply_to_message_id: Option<String>,
    pub forward_from_chat_id: Option<String>,
    pub forward_from_message_id: Option<String>,
    pub file_attachments: Option<serde_json::Value>,
    pub has_mentions: bool,
    pub mentions: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewContext {
    pub chat_id: String,
    pub context_type: String,
    pub summary: String,
    pub key_points: Option<serde_json::Value>,
    pub related_events: Option<serde_json::Value>,
    pub relevance_score: f64,
    pub valid_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEmbedding {
    pub event_id: i64,
    pub content_type: String,
    pub text_content: String,
    pub embedding: Vec<f32>,
    pub metadata: Option<serde_json::Value>,
}
