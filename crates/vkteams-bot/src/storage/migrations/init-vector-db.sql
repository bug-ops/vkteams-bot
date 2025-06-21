-- Initialize VKTeams Bot Database with Vector Search Support
-- This script sets up the database with pgvector extension

-- Ensure the database exists
SELECT 'CREATE DATABASE vkteams_bot' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'vkteams_bot');

-- Connect to the vkteams_bot database
\c vkteams_bot;

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS vector;

-- Set up basic permissions
GRANT ALL PRIVILEGES ON DATABASE vkteams_bot TO vkteams;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO vkteams;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO vkteams;

-- Note: The actual table structure will be created by the application
-- using SQLx migrations when the CLI runs 'database init' command