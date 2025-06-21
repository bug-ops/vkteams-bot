-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS vector;

-- Events table - stores all VK Teams events
CREATE TABLE events (
    id BIGSERIAL PRIMARY KEY,
    event_id VARCHAR(255) UNIQUE NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    chat_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255),
    timestamp TIMESTAMPTZ NOT NULL,
    raw_payload JSONB NOT NULL,
    processed_data JSONB,
    embedding_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for events table
CREATE INDEX idx_events_chat_timestamp ON events(chat_id, timestamp DESC);
CREATE INDEX idx_events_type_timestamp ON events(event_type, timestamp DESC);
CREATE INDEX idx_events_user_timestamp ON events(user_id, timestamp DESC) WHERE user_id IS NOT NULL;
CREATE INDEX idx_events_raw_payload_gin ON events USING GIN(raw_payload);
CREATE INDEX idx_events_event_type ON events(event_type);

-- Messages table - detailed message information
CREATE TABLE messages (
    id BIGSERIAL PRIMARY KEY,
    event_id BIGINT NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    message_id VARCHAR(255) UNIQUE NOT NULL,
    chat_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    text TEXT,
    formatted_text TEXT,
    reply_to_message_id VARCHAR(255),
    forward_from_chat_id VARCHAR(255),
    forward_from_message_id VARCHAR(255),
    file_attachments JSONB,
    has_mentions BOOLEAN DEFAULT FALSE,
    mentions JSONB,
    timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for messages table
CREATE INDEX idx_messages_chat_timestamp ON messages(chat_id, timestamp DESC);
CREATE INDEX idx_messages_user_timestamp ON messages(user_id, timestamp DESC);
CREATE INDEX idx_messages_text_gin ON messages USING GIN(to_tsvector('english', COALESCE(text, '')));
CREATE INDEX idx_messages_has_mentions ON messages(has_mentions) WHERE has_mentions = true;

-- Contexts table - conversation contexts for MCP
CREATE TABLE contexts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    chat_id VARCHAR(255) NOT NULL,
    context_type VARCHAR(100) NOT NULL, -- 'conversation', 'topic', 'user_profile'
    summary TEXT NOT NULL,
    key_points JSONB,
    related_events JSONB, -- array of event_id values
    relevance_score FLOAT DEFAULT 1.0,
    valid_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for contexts table
CREATE INDEX idx_contexts_chat_type ON contexts(chat_id, context_type);
CREATE INDEX idx_contexts_relevance ON contexts(relevance_score DESC);
CREATE INDEX idx_contexts_valid_until ON contexts(valid_until) WHERE valid_until IS NOT NULL;

-- Embeddings table - vector representations
CREATE TABLE embeddings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_id BIGINT NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    content_type VARCHAR(100) NOT NULL, -- 'message', 'summary', 'context'
    text_content TEXT NOT NULL,
    embedding vector(1536), -- OpenAI Ada-002 dimensions
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Vector similarity index
CREATE INDEX ON embeddings USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);

-- Indexes for embeddings table
CREATE INDEX idx_embeddings_content_type ON embeddings(content_type);
CREATE INDEX idx_embeddings_event_id ON embeddings(event_id);

-- Trigger to update updated_at columns
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_events_updated_at BEFORE UPDATE ON events
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_contexts_updated_at BEFORE UPDATE ON contexts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();