use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePool, Row, Executor};
use crate::storage::{models::*, migrations::run_migrations};

#[derive(Debug)]
pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;
        run_migrations(&pool).await?;
        Ok(Self { pool })
    }

    // События
    pub async fn store_event(&self, event: &StoredEvent) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO events (id, event_id, chat_id, event_type, timestamp, raw_data, processed, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&event.id)
        .bind(event.event_id as i64)
        .bind(&event.chat_id)
        .bind(&event.event_type)
        .bind(&event.timestamp)
        .bind(&event.raw_data)
        .bind(event.processed)
        .bind(&event.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_events_since(
        &self,
        chat_id: &str,
        since: DateTime<Utc>,
        limit: i32,
    ) -> Result<Vec<StoredEvent>, sqlx::Error> {
        let events = sqlx::query_as!(
            StoredEvent,
            r#"
            SELECT id, event_id, chat_id, event_type, timestamp, raw_data, processed, created_at
            FROM events 
            WHERE chat_id = ? AND timestamp >= ?
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
            chat_id,
            since,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    pub async fn get_latest_event_id(&self, chat_id: &str) -> Result<Option<u32>, sqlx::Error> {
        let result = sqlx::query!(
            "SELECT MAX(event_id) as max_event_id FROM events WHERE chat_id = ?",
            chat_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.and_then(|row| row.max_event_id.map(|id| id as u32)))
    }

    pub async fn mark_event_as_processed(&self, event_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE events SET processed = TRUE WHERE id = ?",
            event_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Сообщения
    pub async fn store_message(&self, message: &StoredMessage) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO messages (
                id, event_id, message_id, chat_id, user_id, content, message_type, 
                timestamp, reply_to, forwarded_from, edited, deleted, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            message.id,
            message.event_id,
            message.message_id,
            message.chat_id,
            message.user_id,
            message.content,
            message.message_type,
            message.timestamp,
            message.reply_to,
            message.forwarded_from,
            message.edited,
            message.deleted,
            message.created_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_messages_since(
        &self,
        chat_id: &str,
        since: DateTime<Utc>,
        limit: i32,
    ) -> Result<Vec<StoredMessage>, sqlx::Error> {
        let messages = sqlx::query_as!(
            StoredMessage,
            r#"
            SELECT id, event_id, message_id, chat_id, user_id, content, message_type,
                   timestamp, reply_to, forwarded_from, edited, deleted, created_at
            FROM messages 
            WHERE chat_id = ? AND timestamp >= ?
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
            chat_id,
            since,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }

    pub async fn search_messages(
        &self,
        chat_id: &str,
        query: &str,
        limit: i32,
    ) -> Result<Vec<StoredMessage>, sqlx::Error> {
        let messages = sqlx::query_as!(
            StoredMessage,
            r#"
            SELECT m.id, m.event_id, m.message_id, m.chat_id, m.user_id, m.content, m.message_type,
                   m.timestamp, m.reply_to, m.forwarded_from, m.edited, m.deleted, m.created_at
            FROM messages m
            JOIN messages_fts fts ON m.rowid = fts.rowid
            WHERE fts.chat_id = ? AND messages_fts MATCH ?
            ORDER BY m.timestamp DESC
            LIMIT ?
            "#,
            chat_id,
            query,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }

    pub async fn update_message_as_edited(&self, message_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE messages SET edited = TRUE WHERE message_id = ?",
            message_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_message_as_deleted(&self, message_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE messages SET deleted = TRUE WHERE message_id = ?",
            message_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Файлы
    pub async fn store_file(&self, file: &StoredFile) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO files (id, message_id, file_id, filename, file_type, size, url, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            file.id,
            file.message_id,
            file.file_id,
            file.filename,
            file.file_type,
            file.size,
            file.url,
            file.created_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Аналитика
    pub async fn get_event_analytics(
        &self,
        chat_id: &str,
        days_back: u64,
    ) -> Result<EventAnalytics, sqlx::Error> {
        let since = Utc::now() - chrono::Duration::days(days_back as i64);

        // Общая статистика
        let total_events = sqlx::query!(
            "SELECT COUNT(*) as count FROM events WHERE chat_id = ? AND timestamp >= ?",
            chat_id,
            since
        )
        .fetch_one(&self.pool)
        .await?
        .count;

        let total_messages = sqlx::query!(
            "SELECT COUNT(*) as count FROM messages WHERE chat_id = ? AND timestamp >= ?",
            chat_id,
            since
        )
        .fetch_one(&self.pool)
        .await?
        .count;

        let unique_users = sqlx::query!(
            "SELECT COUNT(DISTINCT user_id) as count FROM messages WHERE chat_id = ? AND timestamp >= ?",
            chat_id,
            since
        )
        .fetch_one(&self.pool)
        .await?
        .count;

        // События по типам
        let events_by_type: Vec<EventTypeCount> = sqlx::query!(
            r#"
            SELECT event_type, COUNT(*) as count 
            FROM events 
            WHERE chat_id = ? AND timestamp >= ?
            GROUP BY event_type
            ORDER BY count DESC
            "#,
            chat_id,
            since
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| EventTypeCount {
            event_type: row.event_type,
            count: row.count,
        })
        .collect();

        // Сообщения по часам
        let messages_by_hour: Vec<MessageHourCount> = sqlx::query!(
            r#"
            SELECT CAST(strftime('%H', timestamp) AS INTEGER) as hour, COUNT(*) as count
            FROM messages 
            WHERE chat_id = ? AND timestamp >= ?
            GROUP BY hour
            ORDER BY hour
            "#,
            chat_id,
            since
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| MessageHourCount {
            hour: row.hour.unwrap_or(0),
            count: row.count,
        })
        .collect();

        Ok(EventAnalytics {
            total_events,
            total_messages,
            unique_users,
            events_by_type,
            messages_by_hour,
        })
    }

    pub async fn get_conversation_context(
        &self,
        chat_id: &str,
        hours_back: u64,
        limit: i32,
    ) -> Result<Vec<StoredMessage>, sqlx::Error> {
        let since = Utc::now() - chrono::Duration::hours(hours_back as i64);
        
        let messages = sqlx::query_as!(
            StoredMessage,
            r#"
            SELECT id, event_id, message_id, chat_id, user_id, content, message_type,
                   timestamp, reply_to, forwarded_from, edited, deleted, created_at
            FROM messages 
            WHERE chat_id = ? AND timestamp >= ? AND deleted = FALSE
            ORDER BY timestamp ASC
            LIMIT ?
            "#,
            chat_id,
            since,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }
}