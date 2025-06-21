# VKTeams Bot Docker Deployment

This directory contains Docker Compose configuration for running the VKTeams Bot stack with CLI and MCP server components.

## Quick Start

### 1. Setup Environment

```bash
# Copy environment template
cp .env.example .env

# Edit .env file with your VK Teams Bot credentials
nano .env
```

Required environment variables:
- `VKTEAMS_BOT_API_TOKEN` - Your VK Teams Bot API token
- `VKTEAMS_BOT_CHAT_ID` - Default chat ID for operations

### 2. Start the Stack

```bash
# Basic deployment (PostgreSQL + CLI + MCP)
docker-compose up -d

# With vector search support
docker-compose --profile vector-search up -d

# With local embeddings (includes Ollama)
docker-compose --profile embedding-local up -d

# Full deployment with all features
docker-compose --profile vector-search --profile embedding-local up -d
```

### 3. Initialize Database

The database will be automatically initialized on first startup. You can also run it manually:

```bash
docker-compose run --rm db-init
```

### 4. Check Status

```bash
# View running services
docker-compose ps

# View logs
docker-compose logs -f

# View logs for specific service
docker-compose logs -f vkteams-mcp
```

## Services

### Core Services

- **postgres** - PostgreSQL database for message storage
- **vkteams-cli** - VK Teams Bot CLI tool
- **vkteams-mcp** - MCP server for external integrations
- **db-init** - One-time database initialization

### Optional Services (Profiles)

- **postgres-vector** (profile: `vector-search`) - PostgreSQL with pgvector for semantic search
- **ollama** (profile: `embedding-local`) - Local embedding service

## Configuration

### Shared Configuration

Both CLI and MCP server use the unified configuration from `shared-config.toml`. You can mount your own configuration:

```yaml
volumes:
  - ./your-config.toml:/app/config/config.toml:ro
```

### Environment Variables

Key environment variables:

```bash
# API Configuration
VKTEAMS_BOT_API_TOKEN=your_token
VKTEAMS_BOT_API_URL=https://api.vk.com
VKTEAMS_BOT_CHAT_ID=your_chat_id

# Database
DATABASE_URL=postgresql://vkteams:password@postgres:5432/vkteams_bot
DB_PASSWORD=vkteams_password

# Embeddings (if using local Ollama)
EMBEDDING_PROVIDER=ollama
EMBEDDING_MODEL=nomic-embed-text
EMBEDDING_ENDPOINT=http://ollama:11434

# Logging
RUST_LOG=info
```

## Usage Examples

### Using CLI Tool

```bash
# Run CLI commands directly
docker-compose exec vkteams-cli vkteams-bot-cli send-text --message "Hello World"

# One-off CLI command
docker-compose run --rm vkteams-cli send-text --message "Hello World"

# Interactive CLI session
docker-compose run --rm vkteams-cli bash
```

### Using MCP Server

The MCP server runs as a persistent service. Connect to it using MCP clients:

```bash
# Check MCP server status
docker-compose logs vkteams-mcp

# Connect to MCP server (stdio transport)
docker-compose exec vkteams-mcp vkteams-bot-mcp
```

### Database Operations

```bash
# Database statistics
docker-compose run --rm vkteams-cli database stats

# Search messages
docker-compose run --rm vkteams-cli search text "hello world"

# Semantic search (if vector search enabled)
docker-compose run --rm vkteams-cli search semantic "greeting"
```

## Development

### Building Images

```bash
# Build all images
docker-compose build

# Build specific service
docker-compose build vkteams-mcp

# Force rebuild without cache
docker-compose build --no-cache
```

### Debug Mode

For development, you can run with debug logging:

```bash
RUST_LOG=debug docker-compose up
```

### Accessing Databases

```bash
# Connect to main PostgreSQL
docker-compose exec postgres psql -U vkteams vkteams_bot

# Connect to vector PostgreSQL
docker-compose exec postgres-vector psql -U vkteams vkteams_bot
```

## Persistence

Data is persisted in Docker volumes:

- `postgres_data` - Main database data
- `postgres_vector_data` - Vector database data (if enabled)
- `ollama_data` - Ollama models (if enabled)
- `cli_downloads` - Downloaded files
- `cli_uploads` - Uploaded files
- `mcp_logs` - MCP server logs

## Troubleshooting

### Common Issues

1. **Permission Denied**
   ```bash
   # Check file permissions
   ls -la shared-config.toml
   chmod 644 shared-config.toml
   ```

2. **Database Connection Failed**
   ```bash
   # Check database health
   docker-compose ps postgres
   docker-compose logs postgres
   ```

3. **MCP Server Not Responding**
   ```bash
   # Check CLI binary in MCP container
   docker-compose exec vkteams-mcp which vkteams-bot-cli
   docker-compose exec vkteams-mcp vkteams-bot-cli --version
   ```

### Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f vkteams-mcp

# Last 100 lines
docker-compose logs --tail 100 vkteams-cli
```

### Clean Up

```bash
# Stop all services
docker-compose down

# Remove volumes (data will be lost!)
docker-compose down -v

# Remove images
docker-compose down --rmi all

# Full cleanup
docker-compose down -v --rmi all --remove-orphans
```

## Architecture

```
┌─────────────────┐    ┌─────────────────┐
│   MCP Server    │    │   CLI Tool      │
│  (Port: stdio)  │    │  (Commands)     │
└─────────┬───────┘    └─────────┬───────┘
          │                      │
          └──────────┬───────────┘
                     │ CLI Bridge
          ┌──────────▼───────────┐
          │    PostgreSQL DB     │
          │   (Port: 5432)       │
          └──────────────────────┘
                     │
          ┌──────────▼───────────┐
          │  Ollama (Optional)   │
          │   (Port: 11434)      │
          └──────────────────────┘
```

The MCP server communicates with the CLI tool via subprocess calls, ensuring a single source of truth for all business logic.