# VKTeams-Bot (unofficial)

[![Crates.io](https://img.shields.io/crates/v/vkteams-bot)](https://crates.io/crates/vkteams-bot)
[![codecov](https://codecov.io/github/bug-ops/vkteams-bot/graph/badge.svg?token=XV23ZKSZRA)](https://codecov.io/github/bug-ops/vkteams-bot)
[![Downloads](https://img.shields.io/crates/d/vkteams-bot)](https://crates.io/crates/vkteams-bot)
[![docs.rs](https://docs.rs/vkteams-bot/badge.svg)](https://docs.rs/vkteams-bot)
[![Build Status](https://github.com/bug-ops/vkteams-bot/workflows/Rust/badge.svg)](https://github.com/bug-ops/vkteams-bot/actions)
[![CodeQL](https://github.com/bug-ops/vkteams-bot/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/bug-ops/vkteams-bot/actions/workflows/github-code-scanning/codeql)
[![License](https://img.shields.io/crates/l/vkteams-bot)](LICENSE)

> **The modern, high-performance VK Teams Bot API toolkit for Rust** 🦀  
> Complete ecosystem: Client library + CLI tools + MCP server + Storage infrastructure

## ✨ Why?

- **🚀 Fast**: Rust performance with zero-cost abstractions
- **🛠️ Complete Toolkit**: Library, CLI, MCP server, and storage in one ecosystem  
- **🤖 AI-Ready**: Native Model Context Protocol (MCP) support for LLM integration
- **💾 Smart Storage**: PostgreSQL + Vector search for semantic message analysis
- **⚡ Developer-First**: Intuitive CLI with auto-completion and colored output
- **🏢 Enterprise-Grade**: Memory-safe, concurrent, production-ready
- **📦 Modular**: Use only what you need - each component works independently

## 🚀 Quick Start

### Install CLI (Fastest way to get started)

```bash
cargo install vkteams-bot-cli

# Set your credentials (or use config file)
export VKTEAMS_BOT_API_TOKEN="your_token_here"
export VKTEAMS_BOT_API_URL="your_api_url"

# Send your first message
vkteams-bot-cli send-text -u user123 -m "Hello from Rust! 🦀"
```

### Use as Library

```toml
[dependencies]
vkteams-bot = "0.11"
tokio = { version = "1.0", features = ["full"] }
```

```rust
use vkteams_bot::Bot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bot = Bot::with_default_version("API_TOKEN", "API_URL");
    
    // Send a message
    bot.send_text("chat_id", "Hello, World! 🌍").await?;
    
    // Listen for events
    let events = bot.get_events().await?;
    println!("Received {} events", events.len());
    
    Ok(())
}
```

## 🔧 Complete Ecosystem

| Component | Description | Version |
|-----------|-------------|---------|
| 📚 **Core Library** | High-performance async VK Teams Bot API client | [`vkteams-bot`](https://crates.io/crates/vkteams-bot) v0.11 |
| 🖥️ **CLI Tool** | Feature-complete command-line interface with storage | [`vkteams-bot-cli`](https://crates.io/crates/vkteams-bot-cli) v0.7 |
| 🤖 **MCP Server** | AI/LLM integration via Model Context Protocol | [`vkteams-bot-mcp`](https://crates.io/crates/vkteams-bot-mcp) v0.3 |
| ⚙️ **Macros** | Development productivity macros | [`vkteams-bot-macros`](https://crates.io/crates/vkteams-bot-macros) |

## 🆕 Storage & AI Features

### 💾 Storage Infrastructure

- **PostgreSQL Integration**: Full event and message history storage
- **Vector Search**: Semantic search using pgvector extension
- **AI Embeddings**: OpenAI and Ollama support for text embeddings
- **Smart Search**: Full-text and semantic similarity search

### 🤖 MCP Integration

- **30+ AI Tools**: Messages, files, chats, storage operations
- **Context Management**: Automatic conversation context retrieval
- **CLI-as-Backend**: Unified architecture for consistency

### 🐳 Docker Support

```bash
# Start all services
docker-compose up -d

# Start only essential services
docker-compose --profile relational-database up -d

# Add vector search
docker-compose --profile vector-search up -d
```

## 🎯 Use Cases

- **🏢 Enterprise Chat Automation**: HR bots, IT support, business process automation
- **🤖 AI-Powered Assistants**: LLM integration with Claude, ChatGPT via MCP
- **⚡ DevOps Integration**: CI/CD notifications, monitoring alerts, deployment status
- **📊 Business Intelligence**: Data reporting, analytics dashboards, scheduled reports
- **🔍 Knowledge Management**: Semantic search across chat history
- **🔧 Internal Tools**: Custom workflows, approval processes, team coordination

## 🚀 CLI Highlights

```bash
# Interactive event monitoring with filtering
vkteams-bot-cli get-events -l true | grep "ALARM"

# Batch file operations
find ./reports -name "*.pdf" | xargs -I {} vkteams-bot-cli send-file -u team_lead -p {}

# Semantic search in message history
vkteams-bot-cli storage search-semantic "deployment issues last week"

# Get conversation context for AI
vkteams-bot-cli storage get-context -c chat123 --limit 50

# Storage statistics
vkteams-bot-cli storage stats
```

## 🤖 AI Integration (MCP)

Integrate VK Teams bots directly with AI assistants:

```json
// Claude Desktop config
{
  "mcpServers": {
    "vkteams-bot": {
      "command": "vkteams-bot-mcp",
      "env": {
        "VKTEAMS_BOT_API_TOKEN": "your_token",
        "VKTEAMS_BOT_API_URL": "your_api_url",
        "DATABASE_URL": "postgresql://localhost/vkteams"
      }
    }
  }
}
```

Now Claude can:

- Send messages and manage files
- Search chat history semantically
- Get conversation context
- Execute complex workflows

## ⚙️ Configuration

Create `.config/shared-config.toml`:

```toml
[api]
token = "your_bot_token"
url = "https://api.vk.com"

[storage]
[storage.database]
url = "postgresql://localhost/vkteams"
auto_migrate = true

[storage.embedding]
provider = "ollama"  # or "openai"
model = "nomic-embed-text"
endpoint = "http://localhost:11434"

[mcp]
enable_storage_tools = true
enable_file_tools = true
```

## 🛠️ Development

```bash
# Clone and build
git clone https://github.com/bug-ops/vkteams-bot
cd vkteams-bot
cargo build --release

# Run tests with coverage
cargo llvm-cov nextest report

# Check documentation
cargo doc --open

# Run with Docker
docker-compose up -d
```

## 📖 Documentation

- **📚 API Docs**: [docs.rs/vkteams-bot](https://docs.rs/vkteams-bot)
- **🎯 VK Teams Bot API**: [teams.vk.com/botapi](https://teams.vk.com/botapi/?lang=en)
- **📝 Examples**: [GitHub Examples](https://github.com/bug-ops/vkteams-bot/tree/main/examples)
- **🤖 MCP Protocol**: [Model Context Protocol](https://spec.modelcontextprotocol.io/)
