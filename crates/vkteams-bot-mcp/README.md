# VKTeams Bot MCP Server

[![crates.io](https://img.shields.io/crates/v/vkteams-bot-mcp)](https://crates.io/crates/vkteams-bot-mcp)
[![github.com](https://github.com/bug-ops/vkteams-bot/actions/workflows/Rust.yml/badge.svg)](https://github.com/bug-ops/vkteams-bot/actions)

**A Machine Control Protocol (MCP) server for VK Teams Bot API.**  
Easily integrate VK Teams bots with LLM agents, automation systems, or any external service via a universal protocol.

---

## ✨ Features

- 📤 **Send Messages**: Send text messages to VK Teams chats via MCP
- 🤖 **Bot Info**: Retrieve bot information
- 💬 **Chat Info**: Get chat details
- 📁 **File Info**: Query file metadata
- 📡 **Event Streaming**: Receive chat and bot events
- 🔌 **Universal Integration**: Works over stdin/stdout, perfect for LLMs and automation

---

## Quick Start

### 1. Set Environment Variables

```bash
export VKTEAMS_BOT_API_TOKEN=your_token_here
export VKTEAMS_BOT_API_URL=https://your-api-url
export VKTEAMS_BOT_CHAT_ID=your_chat_id
```

### 2. Build and Run

```bash
cargo build --release
./target/release/vkteams-bot-mcp
```

### 3. Example MCP Request

To send a message to the chat, send the following JSON to the server's stdin:

```json
{
  "tool": "send_text",
  "params": {
    "text": "Hello, world!"
  }
}
```

---

## Usage

The server communicates via the [MCP protocol](https://github.com/ai-forever/rmcp) over standard input/output.  
You can connect it to LLM agents, automation scripts, or use it as a standalone service.

### Supported Tools

- `send_text` — Send a text message to the chat
- `self_get` — Get bot information
- `chat_info` — Get chat information
- `file_info` — Get file information
- `events_get` — Get events from the chat

---

## Documentation

- [VK Teams Bot API](https://teams.vk.com/botapi/?lang=en)
- [MCP Protocol (MCP)](https://modelcontextprotocol.io/specification/2025-03-26)
