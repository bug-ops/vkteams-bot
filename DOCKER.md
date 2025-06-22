# Docker Setup for VKTeams Bot

This document describes how to run VKTeams Bot components using Docker and docker-compose.

## Quick Start

1. **Setup environment:**
   ```bash
   make setup
   # Edit .env file with your configuration
   ```

2. **Start daemon with full stack:**
   ```bash
   make up-daemon
   ```

3. **Or start MCP server only:**
   ```bash
   make up-mcp
   ```

## Available Services

### Core Services

- **vkteams-daemon**: CLI daemon for event processing and storage
- **vkteams-mcp**: MCP server for AI integration (Claude, ChatGPT)

### Storage Services

- **postgres**: Basic PostgreSQL database
- **postgres-vector**: PostgreSQL with pgvector extension for vector search
- **ollama**: Local embedding generation service

## Docker Profiles

Use profiles to control which services to run:

| Profile | Services | Purpose |
|---------|----------|---------|
| `relational-database` | postgres | Basic database only |
| `vector-search` | postgres-vector | Vector search capabilities |
| `embedding-local` | ollama | Local AI embeddings |
| `daemon` | vkteams-daemon | Event processing daemon |
| `mcp` | vkteams-mcp | MCP server for AI integration |

## Common Commands

### Building
```bash
make build              # Build all containers
make build-daemon       # Build daemon container only
make build-mcp          # Build MCP container only
```

### Running
```bash
make up                 # Basic setup (database only)
make up-daemon          # Daemon + vector search + embeddings
make up-mcp             # MCP server + vector search
make up-full            # All services
```

### Monitoring
```bash
make status             # Show service status
make logs               # Show all logs
make logs-daemon        # Show daemon logs
make logs-mcp           # Show MCP logs
make logs-db            # Show database logs
```

### Daemon Management
```bash
make daemon-status      # Check daemon status
make daemon-stop        # Stop daemon gracefully
make daemon-restart     # Restart daemon container
```

### Testing
```bash
make test-daemon        # Test daemon functionality
make test-mcp           # Test MCP server
```

### Development
```bash
make shell-daemon       # Open shell in daemon container
make shell-mcp          # Open shell in MCP container
```

### Cleanup
```bash
make down               # Stop all services
make down-volumes       # Stop and remove volumes
make clean              # Clean containers and images
make clean-all          # Full cleanup
```

## Configuration

### Environment Variables

Key environment variables in `.env` file:

```bash
# Required
VKTEAMS_BOT_API_TOKEN=your_bot_token
VKTEAMS_BOT_CHAT_ID=your_chat_id

# Database
DB_PASSWORD=vkteams_password

# Embeddings (for vector search)
EMBEDDING_PROVIDER=ollama
EMBEDDING_MODEL=nomic-embed-text

# Daemon
DAEMON_AUTO_SAVE=true
DAEMON_POLL_INTERVAL=30
```

### Volumes

Data is persisted in Docker volumes:

- `postgres_data`: PostgreSQL basic database
- `postgres_vector_data`: PostgreSQL with vector data
- `ollama_data`: Ollama models and cache
- `daemon_data`: Daemon application data
- `daemon_logs`: Daemon logs
- `daemon_pids`: Daemon PID files for process management
- `mcp_logs`: MCP server logs

### PID File Management

The daemon creates PID files in `/app/data/pids` inside the container, which is mounted to the `daemon_pids` volume. This ensures:

- Proper daemon status checking across container restarts
- Process monitoring and health checks
- Graceful shutdown handling

## Health Checks

Each service includes health checks:

- **Database**: `pg_isready` check
- **Ollama**: HTTP endpoint check  
- **Daemon**: `vkteams-bot-cli daemon status` check
- **MCP**: Version check

## Troubleshooting

### Common Issues

1. **Permission denied errors:**
   ```bash
   # Ensure volumes have correct permissions
   docker-compose down -v
   make up-daemon
   ```

2. **Database connection issues:**
   ```bash
   # Check if database is healthy
   make status
   make logs-db
   ```

3. **Daemon not starting:**
   ```bash
   # Check daemon logs and status
   make logs-daemon
   make daemon-status
   ```

4. **Vector search not working:**
   ```bash
   # Ensure postgres-vector profile is active
   make up-daemon  # includes vector-search profile
   ```

### Debug Mode

For detailed debugging:

```bash
# Set debug logging
echo "RUST_LOG=debug" >> .env

# Restart with debug logs
make down && make up-daemon
make logs-daemon
```

## Production Considerations

1. **Use external database** for production
2. **Set strong DB_PASSWORD** 
3. **Configure proper backup** for volumes
4. **Monitor resource usage** with daemon processing
5. **Use secrets management** for tokens
6. **Set up log rotation** for persistent logs

## Architecture

```
┌─────────────────┐    ┌─────────────────┐
│   vkteams-mcp   │    │ vkteams-daemon  │
│   (AI Server)   │    │ (Event Proc.)   │
└─────────┬───────┘    └─────────┬───────┘
          │                      │
          └──────────┬───────────┘
                     │
          ┌─────────────────┐    ┌─────────────────┐
          │ postgres-vector │    │     ollama      │
          │  (Storage)      │    │ (Embeddings)    │
          └─────────────────┘    └─────────────────┘
```

- **Daemon** processes VK Teams events and stores to database
- **MCP Server** provides AI integration via CLI bridge  
- **PostgreSQL** with pgvector stores messages and embeddings
- **Ollama** generates embeddings locally (optional)