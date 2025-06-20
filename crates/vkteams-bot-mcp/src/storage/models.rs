use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StoredEvent {
    pub id: String,
    pub event_id: u32,
    pub chat_id: String,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub raw_data: String, // JSON событие
    pub processed: bool,
    pub created_at: DateTime<Utc>,
}

impl StoredEvent {
    pub fn new(
        event_id: u32,
        chat_id: String,
        event_type: String,
        raw_data: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            event_id,
            chat_id,
            event_type,
            timestamp: now,
            raw_data,
            processed: false,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StoredMessage {
    pub id: String,
    pub event_id: String, // FK к StoredEvent
    pub message_id: String,
    pub chat_id: String,
    pub user_id: String,
    pub content: String,
    pub message_type: String,
    pub timestamp: DateTime<Utc>,
    pub reply_to: Option<String>,
    pub forwarded_from: Option<String>,
    pub edited: bool,
    pub deleted: bool,
    pub created_at: DateTime<Utc>,
}

impl StoredMessage {
    pub fn new(
        event_id: String,
        message_id: String,
        chat_id: String,
        user_id: String,
        content: String,
        message_type: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            event_id,
            message_id,
            chat_id,
            user_id,
            content,
            message_type,
            timestamp: now,
            reply_to: None,
            forwarded_from: None,
            edited: false,
            deleted: false,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StoredFile {
    pub id: String,
    pub message_id: String, // FK к StoredMessage
    pub file_id: String,
    pub filename: String,
    pub file_type: String,
    pub size: i64,
    pub url: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl StoredFile {
    pub fn new(
        message_id: String,
        file_id: String,
        filename: String,
        file_type: String,
        size: i64,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message_id,
            file_id,
            filename,
            file_type,
            size,
            url: None,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventAnalytics {
    pub total_events: i64,
    pub total_messages: i64,
    pub unique_users: i64,
    pub events_by_type: Vec<EventTypeCount>,
    pub messages_by_hour: Vec<MessageHourCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTypeCount {
    pub event_type: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHourCount {
    pub hour: i32,
    pub count: i64,
}