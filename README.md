# VKTeams-Bot 🚀

[![Crates.io](https://img.shields.io/crates/v/vkteams-bot)](https://crates.io/crates/vkteams-bot)
[![Downloads](https://img.shields.io/crates/d/vkteams-bot)](https://crates.io/crates/vkteams-bot)
[![docs.rs](https://docs.rs/vkteams-bot/badge.svg)](https://docs.rs/vkteams-bot)
[![Build Status](https://github.com/bug-ops/vkteams-bot/workflows/Rust/badge.svg)](https://github.com/bug-ops/vkteams-bot/actions)
[![CodeQL](https://github.com/bug-ops/vkteams-bot/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/bug-ops/vkteams-bot/actions/workflows/github-code-scanning/codeql)
[![License](https://img.shields.io/crates/l/vkteams-bot)](LICENSE)

> **The modern, high-performance VK Teams Bot API toolkit for Rust** 🦀  
> Complete ecosystem: Client library + CLI tools + MCP server for AI integration

## ✨ Why VKTeams-Bot?

- **🚀 Blazing Fast**: Rust performance with zero-cost abstractions
- **🛠️ Complete Toolkit**: Library, CLI, and MCP server in one ecosystem  
- **🤖 AI-Ready**: Native Model Context Protocol (MCP) support for LLM integration
- **⚡ Developer-First**: Intuitive CLI with auto-completion and colored output
- **🏢 Enterprise-Grade**: Memory-safe, concurrent, production-ready
- **📦 Modular**: Use only what you need - each component works independently

## 🚀 Quick Start

### Install CLI (Fastest way to get started)

```bash
cargo install vkteams-bot-cli

# Set your credentials
export VKTEAMS_BOT_API_TOKEN="your_token_here"
export VKTEAMS_BOT_API_URL="your_api_url"

# Send your first message
vkteams-bot-cli send-text -u user123 -m "Hello from Rust! 🦀"
```

### Use as Library

```toml
[dependencies]
vkteams-bot = "0.9"
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

| Component | Description | Crate |
|-----------|-------------|-------|
| 📚 **Core Library** | High-performance async VK Teams Bot API client | [`vkteams-bot`](https://crates.io/crates/vkteams-bot) |
| 🖥️ **CLI Tool** | Feature-complete command-line interface | [`vkteams-bot-cli`](https://crates.io/crates/vkteams-bot-cli) |
| 🤖 **MCP Server** | AI/LLM integration via Model Context Protocol | [`vkteams-bot-mcp`](https://crates.io/crates/vkteams-bot-mcp) |
| ⚙️ **Macros** | Development productivity macros | [`vkteams-bot-macros`](https://crates.io/crates/vkteams-bot-macros) |

## 🎯 Use Cases

- **🏢 Enterprise Chat Automation**: HR bots, IT support, business process automation
- **🤖 AI-Powered Assistants**: LLM integration with Claude, ChatGPT via MCP
- **⚡ DevOps Integration**: CI/CD notifications, monitoring alerts, deployment status
- **📊 Business Intelligence**: Data reporting, analytics dashboards, scheduled reports
- **🔧 Internal Tools**: Custom workflows, approval processes, team coordination

## 🚀 CLI Highlights

```bash
# Interactive event monitoring with filtering
vkteams-bot-cli get-events -l true | grep "ALARM"

# Batch file operations
find ./reports -name "*.pdf" | xargs -I {} vkteams-bot-cli send-file -u team_lead -p {}

# Pipeline integration
echo "Deployment successful! ✅" | vkteams-bot-cli send-text -u devops_chat

# File management
vkteams-bot-cli get-file -i file123 -f ./downloads/report.pdf
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
        "VKTEAMS_BOT_API_URL": "your_api_url"
      }
    }
  }
}
```

Now Claude can send messages, manage files, and interact with your VK Teams directly!

## 🛠️ Development

```bash
# Clone and build
git clone https://github.com/bug-ops/vkteams-bot
cd vkteams-bot
cargo build --release

# Run tests
cargo test

# Check documentation
cargo doc --open
```

## 📖 Documentation

- **📚 API Docs**: [docs.rs/vkteams-bot](https://docs.rs/vkteams-bot)
- **🎯 VK Teams Bot API**: [teams.vk.com/botapi](https://teams.vk.com/botapi/?lang=en)
- **📝 Examples**: [GitHub Examples](https://github.com/bug-ops/vkteams-bot/tree/main/examples)
- **🤖 MCP Protocol**: [Model Context Protocol](https://spec.modelcontextprotocol.io/)
