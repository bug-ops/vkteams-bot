use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePool, Row};
use crate::storage::models::*;

#[derive(Debug)]
pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;
        
        // Создаем таблицы если их нет
        sqlx::raw_sql(r#"
            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                event_id INTEGER NOT NULL UNIQUE,
                chat_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                timestamp DATETIME NOT NULL,
                raw_data TEXT NOT NULL,
                processed BOOLEAN DEFAULT FALSE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE INDEX IF NOT EXISTS idx_events_chat_timestamp ON events (chat_id, timestamp);
            CREATE INDEX IF NOT EXISTS idx_events_type ON events (event_type);
            CREATE INDEX IF NOT EXISTS idx_events_event_id ON events (event_id);
        "#).execute(&pool).await?;

        sqlx::raw_sql(r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                event_id TEXT NOT NULL,
                message_id TEXT NOT NULL,
                chat_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                content TEXT NOT NULL,
                message_type TEXT NOT NULL,
                timestamp DATETIME NOT NULL,
                reply_to TEXT,
                forwarded_from TEXT,
                edited BOOLEAN DEFAULT FALSE,
                deleted BOOLEAN DEFAULT FALSE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE INDEX IF NOT EXISTS idx_messages_chat_user ON messages (chat_id, user_id);
            CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages (timestamp);
            CREATE INDEX IF NOT EXISTS idx_messages_message_id ON messages (message_id);
            CREATE INDEX IF NOT EXISTS idx_messages_event_id ON messages (event_id);
        "#).execute(&pool).await?;

        sqlx::raw_sql(r#"
            CREATE TABLE IF NOT EXISTS files (
                id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                file_id TEXT NOT NULL,
                filename TEXT NOT NULL,
                file_type TEXT NOT NULL,
                size INTEGER NOT NULL,
                url TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE INDEX IF NOT EXISTS idx_files_message_id ON files (message_id);
            CREATE INDEX IF NOT EXISTS idx_files_file_id ON files (file_id);
        "#).execute(&pool).await?;
        
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
        let rows = sqlx::query(
            r#"
            SELECT id, event_id, chat_id, event_type, timestamp, raw_data, processed, created_at
            FROM events 
            WHERE chat_id = ? AND timestamp >= ?
            ORDER BY timestamp DESC
            LIMIT ?
            "#
        )
        .bind(chat_id)
        .bind(&since)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::new();
        for row in rows {
            events.push(StoredEvent {
                id: row.get("id"),
                event_id: row.get::<i64, _>("event_id") as u32,
                chat_id: row.get("chat_id"),
                event_type: row.get("event_type"),
                timestamp: row.get("timestamp"),
                raw_data: row.get("raw_data"),
                processed: row.get("processed"),
                created_at: row.get("created_at"),
            });
        }

        Ok(events)
    }

    pub async fn get_latest_event_id(&self, chat_id: &str) -> Result<Option<u32>, sqlx::Error> {
        let row = sqlx::query("SELECT MAX(event_id) as max_event_id FROM events WHERE chat_id = ?")
            .bind(chat_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.and_then(|r| r.get::<Option<i64>, _>("max_event_id").map(|id| id as u32)))
    }

    pub async fn mark_event_as_processed(&self, event_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE events SET processed = TRUE WHERE id = ?")
            .bind(event_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Сообщения
    pub async fn store_message(&self, message: &StoredMessage) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO messages (
                id, event_id, message_id, chat_id, user_id, content, message_type, 
                timestamp, reply_to, forwarded_from, edited, deleted, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&message.id)
        .bind(&message.event_id)
        .bind(&message.message_id)
        .bind(&message.chat_id)
        .bind(&message.user_id)
        .bind(&message.content)
        .bind(&message.message_type)
        .bind(&message.timestamp)
        .bind(&message.reply_to)
        .bind(&message.forwarded_from)
        .bind(message.edited)
        .bind(message.deleted)
        .bind(&message.created_at)
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
        let rows = sqlx::query(
            r#"
            SELECT id, event_id, message_id, chat_id, user_id, content, message_type,
                   timestamp, reply_to, forwarded_from, edited, deleted, created_at
            FROM messages 
            WHERE chat_id = ? AND timestamp >= ?
            ORDER BY timestamp DESC
            LIMIT ?
            "#
        )
        .bind(chat_id)
        .bind(&since)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(StoredMessage {
                id: row.get("id"),
                event_id: row.get("event_id"),
                message_id: row.get("message_id"),
                chat_id: row.get("chat_id"),
                user_id: row.get("user_id"),
                content: row.get("content"),
                message_type: row.get("message_type"),
                timestamp: row.get("timestamp"),
                reply_to: row.get("reply_to"),
                forwarded_from: row.get("forwarded_from"),
                edited: row.get("edited"),
                deleted: row.get("deleted"),
                created_at: row.get("created_at"),
            });
        }

        Ok(messages)
    }

    pub async fn search_messages(
        &self,
        chat_id: &str,
        query: &str,
        limit: i32,
    ) -> Result<Vec<StoredMessage>, sqlx::Error> {
        let search_pattern = format!("%{}%", query);
        let rows = sqlx::query(
            r#"
            SELECT id, event_id, message_id, chat_id, user_id, content, message_type,
                   timestamp, reply_to, forwarded_from, edited, deleted, created_at
            FROM messages
            WHERE chat_id = ? AND content LIKE ?
            ORDER BY timestamp DESC
            LIMIT ?
            "#
        )
        .bind(chat_id)
        .bind(&search_pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(StoredMessage {
                id: row.get("id"),
                event_id: row.get("event_id"),
                message_id: row.get("message_id"),
                chat_id: row.get("chat_id"),
                user_id: row.get("user_id"),
                content: row.get("content"),
                message_type: row.get("message_type"),
                timestamp: row.get("timestamp"),
                reply_to: row.get("reply_to"),
                forwarded_from: row.get("forwarded_from"),
                edited: row.get("edited"),
                deleted: row.get("deleted"),
                created_at: row.get("created_at"),
            });
        }

        Ok(messages)
    }

    pub async fn update_message_as_edited(&self, message_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE messages SET edited = TRUE WHERE message_id = ?")
            .bind(message_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_message_as_deleted(&self, message_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE messages SET deleted = TRUE WHERE message_id = ?")
            .bind(message_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Файлы
    pub async fn store_file(&self, file: &StoredFile) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO files (id, message_id, file_id, filename, file_type, size, url, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&file.id)
        .bind(&file.message_id)
        .bind(&file.file_id)
        .bind(&file.filename)
        .bind(&file.file_type)
        .bind(file.size)
        .bind(&file.url)
        .bind(&file.created_at)
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
        let total_events_row = sqlx::query("SELECT COUNT(*) as count FROM events WHERE chat_id = ? AND timestamp >= ?")
            .bind(chat_id)
            .bind(&since)
            .fetch_one(&self.pool)
            .await?;
        let total_events = total_events_row.get::<i64, _>("count");

        let total_messages_row = sqlx::query("SELECT COUNT(*) as count FROM messages WHERE chat_id = ? AND timestamp >= ?")
            .bind(chat_id)
            .bind(&since)
            .fetch_one(&self.pool)
            .await?;
        let total_messages = total_messages_row.get::<i64, _>("count");

        let unique_users_row = sqlx::query("SELECT COUNT(DISTINCT user_id) as count FROM messages WHERE chat_id = ? AND timestamp >= ?")
            .bind(chat_id)
            .bind(&since)
            .fetch_one(&self.pool)
            .await?;
        let unique_users = unique_users_row.get::<i64, _>("count");

        // События по типам
        let events_by_type_rows = sqlx::query(
            r#"
            SELECT event_type, COUNT(*) as count 
            FROM events 
            WHERE chat_id = ? AND timestamp >= ?
            GROUP BY event_type
            ORDER BY count DESC
            "#
        )
        .bind(chat_id)
        .bind(&since)
        .fetch_all(&self.pool)
        .await?;

        let events_by_type: Vec<EventTypeCount> = events_by_type_rows
            .into_iter()
            .map(|row| EventTypeCount {
                event_type: row.get("event_type"),
                count: row.get("count"),
            })
            .collect();

        // Сообщения по часам
        let messages_by_hour_rows = sqlx::query(
            r#"
            SELECT CAST(strftime('%H', timestamp) AS INTEGER) as hour, COUNT(*) as count
            FROM messages 
            WHERE chat_id = ? AND timestamp >= ?
            GROUP BY hour
            ORDER BY hour
            "#
        )
        .bind(chat_id)
        .bind(&since)
        .fetch_all(&self.pool)
        .await?;

        let messages_by_hour: Vec<MessageHourCount> = messages_by_hour_rows
            .into_iter()
            .map(|row| MessageHourCount {
                hour: row.get::<Option<i32>, _>("hour").unwrap_or(0),
                count: row.get("count"),
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
        
        let rows = sqlx::query(
            r#"
            SELECT id, event_id, message_id, chat_id, user_id, content, message_type,
                   timestamp, reply_to, forwarded_from, edited, deleted, created_at
            FROM messages 
            WHERE chat_id = ? AND timestamp >= ? AND deleted = FALSE
            ORDER BY timestamp ASC
            LIMIT ?
            "#
        )
        .bind(chat_id)
        .bind(&since)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(StoredMessage {
                id: row.get("id"),
                event_id: row.get("event_id"),
                message_id: row.get("message_id"),
                chat_id: row.get("chat_id"),
                user_id: row.get("user_id"),
                content: row.get("content"),
                message_type: row.get("message_type"),
                timestamp: row.get("timestamp"),
                reply_to: row.get("reply_to"),
                forwarded_from: row.get("forwarded_from"),
                edited: row.get("edited"),
                deleted: row.get("deleted"),
                created_at: row.get("created_at"),
            });
        }

        Ok(messages)
    }
}