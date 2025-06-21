//! Relational database operations using sqlx

use crate::storage::{StorageError, StorageResult};
use crate::storage::models::*;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// PostgreSQL database operations
#[derive(Debug, Clone)]
pub struct RelationalStore {
    pool: PgPool,
}

impl RelationalStore {
    /// Create a new relational store with connection pool
    pub async fn new(database_url: &str) -> StorageResult<Self> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| StorageError::Connection(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> StorageResult<()> {
        let migration_sql = include_str!("migrations/001_initial.sql");
        sqlx::query(migration_sql)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Migration(e.to_string()))?;

        Ok(())
    }

    /// Insert a new event
    pub async fn insert_event(&self, new_event: NewEvent) -> StorageResult<i64> {
        let row = sqlx::query(
            r#"
            INSERT INTO events (event_id, event_type, chat_id, user_id, timestamp, raw_payload, processed_data)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#
        )
        .bind(&new_event.event_id)
        .bind(&new_event.event_type)
        .bind(&new_event.chat_id)
        .bind(&new_event.user_id)
        .bind(&new_event.timestamp)
        .bind(&new_event.raw_payload)
        .bind(&new_event.processed_data)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    /// Insert a new message
    pub async fn insert_message(&self, new_message: NewMessage) -> StorageResult<i64> {
        let row = sqlx::query!(
            r#"
            INSERT INTO messages (event_id, message_id, chat_id, user_id, text, formatted_text, 
                                reply_to_message_id, forward_from_chat_id, forward_from_message_id,
                                file_attachments, has_mentions, mentions, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id
            "#,
            new_message.event_id,
            new_message.message_id,
            new_message.chat_id,
            new_message.user_id,
            new_message.text,
            new_message.formatted_text,
            new_message.reply_to_message_id,
            new_message.forward_from_chat_id,
            new_message.forward_from_message_id,
            new_message.file_attachments,
            new_message.has_mentions,
            new_message.mentions,
            new_message.timestamp
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    /// Insert a new context
    pub async fn insert_context(&self, new_context: NewContext) -> StorageResult<Uuid> {
        let row = sqlx::query!(
            r#"
            INSERT INTO contexts (chat_id, context_type, summary, key_points, related_events, relevance_score, valid_until)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
            new_context.chat_id,
            new_context.context_type,
            new_context.summary,
            new_context.key_points,
            new_context.related_events,
            new_context.relevance_score,
            new_context.valid_until
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    /// Get event by ID
    pub async fn get_event(&self, id: i64) -> StorageResult<Option<Event>> {
        let event = sqlx::query_as!(
            Event,
            "SELECT * FROM events WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(event)
    }

    /// Get events by chat ID with pagination
    pub async fn get_events_by_chat(
        &self,
        chat_id: &str,
        limit: i64,
        offset: i64,
    ) -> StorageResult<Vec<Event>> {
        let events = sqlx::query_as!(
            Event,
            r#"
            SELECT * FROM events 
            WHERE chat_id = $1 
            ORDER BY timestamp DESC 
            LIMIT $2 OFFSET $3
            "#,
            chat_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    /// Get recent events with optional time filter
    pub async fn get_recent_events(
        &self,
        chat_id: Option<&str>,
        since: Option<DateTime<Utc>>,
        limit: i64,
    ) -> StorageResult<Vec<Event>> {
        let events = match (chat_id, since) {
            (Some(chat_id), Some(since)) => {
                sqlx::query_as!(
                    Event,
                    r#"
                    SELECT * FROM events 
                    WHERE chat_id = $1 AND timestamp >= $2
                    ORDER BY timestamp DESC 
                    LIMIT $3
                    "#,
                    chat_id,
                    since,
                    limit
                )
                .fetch_all(&self.pool)
                .await?
            }
            (Some(chat_id), None) => {
                sqlx::query_as!(
                    Event,
                    r#"
                    SELECT * FROM events 
                    WHERE chat_id = $1
                    ORDER BY timestamp DESC 
                    LIMIT $2
                    "#,
                    chat_id,
                    limit
                )
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(since)) => {
                sqlx::query_as!(
                    Event,
                    r#"
                    SELECT * FROM events 
                    WHERE timestamp >= $1
                    ORDER BY timestamp DESC 
                    LIMIT $2
                    "#,
                    since,
                    limit
                )
                .fetch_all(&self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as!(
                    Event,
                    r#"
                    SELECT * FROM events 
                    ORDER BY timestamp DESC 
                    LIMIT $1
                    "#,
                    limit
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(events)
    }

    /// Full-text search in messages
    pub async fn search_messages(
        &self,
        query: &str,
        chat_id: Option<&str>,
        limit: i64,
    ) -> StorageResult<Vec<Message>> {
        let messages = match chat_id {
            Some(chat_id) => {
                sqlx::query_as!(
                    Message,
                    r#"
                    SELECT * FROM messages 
                    WHERE chat_id = $1 
                      AND to_tsvector('english', COALESCE(text, '')) @@ plainto_tsquery('english', $2)
                    ORDER BY timestamp DESC 
                    LIMIT $3
                    "#,
                    chat_id,
                    query,
                    limit
                )
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as!(
                    Message,
                    r#"
                    SELECT * FROM messages 
                    WHERE to_tsvector('english', COALESCE(text, '')) @@ plainto_tsquery('english', $1)
                    ORDER BY timestamp DESC 
                    LIMIT $2
                    "#,
                    query,
                    limit
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(messages)
    }

    /// Get statistics about stored events
    pub async fn get_stats(&self, chat_id: Option<&str>) -> StorageResult<EventStats> {
        let (total_events, total_messages, unique_chats, unique_users) = match chat_id {
            Some(chat_id) => {
                let row = sqlx::query!(
                    r#"
                    SELECT 
                        COUNT(DISTINCT e.id) as total_events,
                        COUNT(DISTINCT m.id) as total_messages,
                        1 as unique_chats,
                        COUNT(DISTINCT e.user_id) as unique_users
                    FROM events e
                    LEFT JOIN messages m ON e.id = m.event_id
                    WHERE e.chat_id = $1
                    "#,
                    chat_id
                )
                .fetch_one(&self.pool)
                .await?;

                (
                    row.total_events.unwrap_or(0),
                    row.total_messages.unwrap_or(0),
                    row.unique_chats.unwrap_or(0),
                    row.unique_users.unwrap_or(0),
                )
            }
            None => {
                let row = sqlx::query!(
                    r#"
                    SELECT 
                        COUNT(DISTINCT e.id) as total_events,
                        COUNT(DISTINCT m.id) as total_messages,
                        COUNT(DISTINCT e.chat_id) as unique_chats,
                        COUNT(DISTINCT e.user_id) as unique_users
                    FROM events e
                    LEFT JOIN messages m ON e.id = m.event_id
                    "#
                )
                .fetch_one(&self.pool)
                .await?;

                (
                    row.total_events.unwrap_or(0),
                    row.total_messages.unwrap_or(0),
                    row.unique_chats.unwrap_or(0),
                    row.unique_users.unwrap_or(0),
                )
            }
        };

        // Get time-based stats
        let time_stats = match chat_id {
            Some(chat_id) => {
                sqlx::query!(
                    r#"
                    SELECT 
                        COUNT(*) FILTER (WHERE timestamp >= NOW() - INTERVAL '24 hours') as events_last_24h,
                        COUNT(*) FILTER (WHERE timestamp >= NOW() - INTERVAL '7 days') as events_last_week,
                        MIN(timestamp) as oldest_event,
                        MAX(timestamp) as newest_event
                    FROM events
                    WHERE chat_id = $1
                    "#,
                    chat_id
                )
                .fetch_one(&self.pool)
                .await?
            }
            None => {
                sqlx::query!(
                    r#"
                    SELECT 
                        COUNT(*) FILTER (WHERE timestamp >= NOW() - INTERVAL '24 hours') as events_last_24h,
                        COUNT(*) FILTER (WHERE timestamp >= NOW() - INTERVAL '7 days') as events_last_week,
                        MIN(timestamp) as oldest_event,
                        MAX(timestamp) as newest_event
                    FROM events
                    "#
                )
                .fetch_one(&self.pool)
                .await?
            }
        };

        // Get storage size (approximate)
        let storage_size = sqlx::query!(
            "SELECT pg_total_relation_size('events') + pg_total_relation_size('messages') + pg_total_relation_size('contexts') as size"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(EventStats {
            total_events,
            total_messages,
            unique_chats,
            unique_users,
            events_last_24h: time_stats.events_last_24h.unwrap_or(0),
            events_last_week: time_stats.events_last_week.unwrap_or(0),
            oldest_event: time_stats.oldest_event,
            newest_event: time_stats.newest_event,
            storage_size_bytes: storage_size.size,
        })
    }

    /// Clean up old events
    pub async fn cleanup_old_events(&self, older_than_days: u32) -> StorageResult<u64> {
        let result = sqlx::query!(
            "DELETE FROM events WHERE created_at < NOW() - INTERVAL '1 day' * $1",
            older_than_days as i32
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Get database health status
    pub async fn health_check(&self) -> StorageResult<()> {
        sqlx::query!("SELECT 1 as health")
            .fetch_one(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    // Note: These tests require a PostgreSQL database with the test schema
    #[tokio::test]
    #[ignore] // Ignore by default, run with --ignored
    async fn test_relational_store_connection() {
        let store = RelationalStore::new("postgresql://postgres:password@localhost/test_db")
            .await
            .expect("Failed to connect to test database");

        store.health_check().await.expect("Health check failed");
    }

    #[tokio::test]
    #[ignore]
    async fn test_insert_and_get_event() {
        let store = RelationalStore::new("postgresql://postgres:password@localhost/test_db")
            .await
            .expect("Failed to connect to test database");

        let new_event = NewEvent {
            event_id: "test_event_1".to_string(),
            event_type: "newMessage".to_string(),
            chat_id: "test_chat".to_string(),
            user_id: Some("test_user".to_string()),
            timestamp: Utc::now(),
            raw_payload: json!({"test": "data"}),
            processed_data: None,
        };

        let event_id = store.insert_event(new_event).await.expect("Failed to insert event");
        let retrieved_event = store.get_event(event_id).await.expect("Failed to get event");

        assert!(retrieved_event.is_some());
        let event = retrieved_event.unwrap();
        assert_eq!(event.event_id, "test_event_1");
        assert_eq!(event.chat_id, "test_chat");
    }
}