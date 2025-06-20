-- События
CREATE TABLE events (
    id TEXT PRIMARY KEY,
    event_id INTEGER NOT NULL UNIQUE,
    chat_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    raw_data TEXT NOT NULL,
    processed BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Сообщения
CREATE TABLE messages (
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
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (event_id) REFERENCES events (id)
);

-- Файлы
CREATE TABLE files (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL,
    file_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    file_type TEXT NOT NULL,
    size INTEGER NOT NULL,
    url TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (message_id) REFERENCES messages (id)
);

-- Индексы для производительности
CREATE INDEX idx_events_chat_timestamp ON events (chat_id, timestamp);
CREATE INDEX idx_events_type ON events (event_type);
CREATE INDEX idx_events_event_id ON events (event_id);

CREATE INDEX idx_messages_chat_user ON messages (chat_id, user_id);
CREATE INDEX idx_messages_timestamp ON messages (timestamp);
CREATE INDEX idx_messages_message_id ON messages (message_id);
CREATE INDEX idx_messages_event_id ON messages (event_id);

CREATE INDEX idx_files_message_id ON files (message_id);
CREATE INDEX idx_files_file_id ON files (file_id);

-- Полнотекстовый поиск по содержимому сообщений
CREATE VIRTUAL TABLE messages_fts USING fts5(
    content,
    chat_id,
    user_id,
    timestamp,
    content='messages',
    content_rowid='rowid'
);

-- Триггеры для автоматического обновления FTS индекса
CREATE TRIGGER messages_fts_insert AFTER INSERT ON messages BEGIN
    INSERT INTO messages_fts(rowid, content, chat_id, user_id, timestamp) 
    VALUES (new.rowid, new.content, new.chat_id, new.user_id, new.timestamp);
END;

CREATE TRIGGER messages_fts_update AFTER UPDATE ON messages BEGIN
    UPDATE messages_fts SET 
        content = new.content, 
        chat_id = new.chat_id, 
        user_id = new.user_id, 
        timestamp = new.timestamp 
    WHERE rowid = old.rowid;
END;

CREATE TRIGGER messages_fts_delete AFTER DELETE ON messages BEGIN
    DELETE FROM messages_fts WHERE rowid = old.rowid;
END;