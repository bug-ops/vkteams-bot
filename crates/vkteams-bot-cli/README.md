# VK Teams Bot API CLI

[![docs.rs](https://img.shields.io/docsrs/vkteams-bot-cli/latest)](https://docs.rs/vkteams-bot-cli/latest/vkteams_bot_cli/)
[![crates.io](https://img.shields.io/crates/v/vkteams-bot-cli)](https://crates.io/crates/vkteams-bot-cli)
[![github.com](https://github.com/k05h31/vkteams-bot-cli/workflows/Rust/badge.svg)](https://github.com/k05h31/vkteams-bot/actions)
[![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

🤖 **Powerful command-line interface for VK Teams Bot API** with interactive setup, progress bars, and comprehensive chat management features.

## ✨ Features

- 📤 **Message Operations**: Send text, files, and voice messages
- 💬 **Chat Management**: Get info, manage members, set titles and descriptions
- 🔧 **Message Editing**: Edit, delete, pin/unpin messages
- 📁 **File Handling**: Upload and download files with progress bars
- 🎯 **Event Monitoring**: Real-time event listening with long polling
- ⚙️ **Smart Configuration**: Interactive setup wizard and multiple config sources
- 🔍 **Diagnostics**: Built-in validation and connection testing
- 🎨 **User-Friendly**: Colored output, progress indicators, and helpful examples

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Commands](#commands)
- [Examples](#examples)
- [Advanced Usage](#advanced-usage)

## Installation

### From crates.io (Recommended)

```bash
cargo install vkteams-bot-cli
```

### From source

```bash
git clone https://github.com/bug-ops/vkteams-bot
cd vkteams-bot
cargo install --path crates/vkteams-bot-cli
```

## Quick Start

### 1. Get Your Bot Credentials

Follow the [VK Teams Bot API instructions](https://teams.vk.com/botapi/?lang=en) to create a bot and get your:

- **API Token**
- **API URL**

### 2. Interactive Setup

```bash
vkteams-bot-cli setup
```

This will guide you through the initial configuration process.

### 3. Test Your Setup

```bash
vkteams-bot-cli validate
```

### 4. Send Your First Message

```bash
vkteams-bot-cli send-text -u USER_ID -m "Hello from CLI!"
```

## Configuration

The CLI supports multiple configuration methods (in order of precedence):

### 1. Environment Variables

```bash
# Required
export VKTEAMS_BOT_API_TOKEN=your_token_here
export VKTEAMS_BOT_API_URL=your_api_url_here

# Optional
export VKTEAMS_PROXY=http://proxy:8080
export VKTEAMS_LOG_LEVEL=info
export VKTEAMS_DOWNLOAD_DIR=/path/to/downloads
```

### 2. Configuration File

The CLI automatically looks for config files in:

- Current directory: `cli_config.toml`
- User config: `~/.config/vkteams-bot/cli_config.toml`
- System config: `/etc/vkteams-bot/cli_config.toml` (Unix only)

### 3. Interactive Configuration

```bash
# Initial setup wizard
vkteams-bot-cli setup

# Update existing configuration
vkteams-bot-cli config --wizard

# View current configuration
vkteams-bot-cli config --show
```

## Commands

### 📤 Message Operations

| Command | Description | Example |
|---------|-------------|---------|
| `send-text` | Send text message | `vkteams-bot-cli send-text -u USER_ID -m "Hello!"` |
| `send-file` | Send file with progress | `vkteams-bot-cli send-file -u USER_ID -p file.pdf` |
| `send-voice` | Send voice message | `vkteams-bot-cli send-voice -u USER_ID -p voice.ogg` |
| `edit-message` | Edit existing message | `vkteams-bot-cli edit-message -c CHAT_ID -m MSG_ID -t "New text"` |
| `delete-message` | Delete message | `vkteams-bot-cli delete-message -c CHAT_ID -m MSG_ID` |
| `pin-message` | Pin message | `vkteams-bot-cli pin-message -c CHAT_ID -m MSG_ID` |
| `unpin-message` | Unpin message | `vkteams-bot-cli unpin-message -c CHAT_ID -m MSG_ID` |

### 💬 Chat Management

| Command | Description | Example |
|---------|-------------|---------|
| `get-chat-info` | Get chat details | `vkteams-bot-cli get-chat-info -c CHAT_ID` |
| `get-chat-members` | List chat members | `vkteams-bot-cli get-chat-members -c CHAT_ID` |
| `set-chat-title` | Change chat title | `vkteams-bot-cli set-chat-title -c CHAT_ID -t "New Title"` |
| `set-chat-about` | Set chat description | `vkteams-bot-cli set-chat-about -c CHAT_ID -a "Description"` |
| `send-action` | Send typing/looking action | `vkteams-bot-cli send-action -c CHAT_ID -a typing` |

### 📁 File Operations

| Command | Description | Example |
|---------|-------------|---------|
| `get-file` | Download file with progress | `vkteams-bot-cli get-file -f FILE_ID -p /downloads/` |

### 📡 Event Monitoring

| Command | Description | Example |
|---------|-------------|---------|
| `get-events` | Get events once | `vkteams-bot-cli get-events` |
| `get-events -l true` | Start long polling | `vkteams-bot-cli get-events -l true` |

### 🔍 Information & Diagnostics

| Command | Description | Example |
|---------|-------------|---------|
| `get-self` | Get bot information | `vkteams-bot-cli get-self` |
| `get-profile` | Get user profile | `vkteams-bot-cli get-profile -u USER_ID` |
| `validate` | Test configuration | `vkteams-bot-cli validate` |
| `examples` | Show usage examples | `vkteams-bot-cli examples` |
| `list-commands` | Show all commands | `vkteams-bot-cli list-commands` |

### ⚙️ Configuration

| Command | Description | Example |
|---------|-------------|---------|
| `setup` | Interactive setup wizard | `vkteams-bot-cli setup` |
| `config --show` | Show current config | `vkteams-bot-cli config --show` |
| `config --wizard` | Update config interactively | `vkteams-bot-cli config --wizard` |

## Examples

### Basic Messaging

```bash
# Send a simple text message
vkteams-bot-cli send-text -u user123 -m "Hello, World!"

# Send a file with automatic progress bar
vkteams-bot-cli send-file -u user123 -p /path/to/document.pdf

# Send a voice message
vkteams-bot-cli send-voice -u user123 -p /path/to/audio.ogg
```

### Chat Management

```bash
# Get detailed chat information
vkteams-bot-cli get-chat-info -c chat456

# List all members in a chat
vkteams-bot-cli get-chat-members -c chat456

# Update chat settings
vkteams-bot-cli set-chat-title -c chat456 -t "New Project Chat"
vkteams-bot-cli set-chat-about -c chat456 -a "Discussion for the new project"
```

### Message Operations

```bash
# Edit a message
vkteams-bot-cli edit-message -c chat456 -m msg789 -t "Updated message content"

# Pin an important message
vkteams-bot-cli pin-message -c chat456 -m msg789

# Delete a message
vkteams-bot-cli delete-message -c chat456 -m msg789
```

### File Operations

```bash
# Download a file to specific directory
vkteams-bot-cli get-file -f file123 -p /downloads/

# The file will be saved with its original name in the specified directory
```

### Event Monitoring

```bash
# Get events once
vkteams-bot-cli get-events

# Start continuous event monitoring
vkteams-bot-cli get-events -l true

# Monitor events and filter for specific types
vkteams-bot-cli get-events -l true | jq '.events[] | select(.type == "newMessage")'

# Monitor events and search for keywords
vkteams-bot-cli get-events -l true | grep -i "urgent"
```

### Chat Interaction

```bash
# Send typing indicator
vkteams-bot-cli send-action -c chat456 -a typing

# Send looking indicator  
vkteams-bot-cli send-action -c chat456 -a looking
```

## Advanced Usage

### Configuration Management

```bash
# Save configuration to custom location
vkteams-bot-cli --save-config /path/to/custom-config.toml

# Use custom configuration file
vkteams-bot-cli --config /path/to/custom-config.toml send-text -u user123 -m "Hello"

# Initialize configuration with defaults
vkteams-bot-cli config --init
```

### Scripting and Automation

```bash
#!/bin/bash
# Example script for daily notifications

CHAT_ID="your_chat_id"
MESSAGE="Daily reminder: Check your tasks!"

# Send notification
vkteams-bot-cli send-text -u "$CHAT_ID" -m "$MESSAGE"

# Send typing action first for more natural interaction
vkteams-bot-cli send-action -c "$CHAT_ID" -a typing
sleep 2
vkteams-bot-cli send-text -u "$CHAT_ID" -m "$MESSAGE"
```

### Environment-Specific Configuration

```bash
# Environment  
export VKTEAMS_BOT_API_URL="https://api.example.com"
export VKTEAMS_LOG_LEVEL="info"

# Validate current setup
vkteams-bot-cli validate
```

### Troubleshooting

```bash
# Test your configuration
vkteams-bot-cli validate

# Get detailed bot information
vkteams-bot-cli get-self --detailed

# Check configuration
vkteams-bot-cli config --show

# View all available commands with descriptions
vkteams-bot-cli list-commands

# See usage examples
vkteams-bot-cli examples
```

## Error Handling

The CLI provides clear error messages and appropriate exit codes:

- **Exit Code 0**: Success
- **Exit Code 64**: Usage error (invalid arguments)
- **Exit Code 69**: Service unavailable (API errors)
- **Exit Code 74**: I/O error (file operations)
- **Exit Code 70**: Software error (unexpected errors)

## Configuration File Format

Example `cli_config.toml`:

```toml
[api]
token = "your_bot_token"
url = "https://api.teams.vk.com"
timeout = 30
max_retries = 3

[files]
download_dir = "/home/user/downloads"
upload_dir = "/home/user/uploads"
max_file_size = 104857600  # 100MB
buffer_size = 65536        # 64KB

[logging]
level = "info"
format = "text"
colors = true

[ui]
show_progress = true
progress_style = "unicode"
progress_refresh_rate = 100

[proxy]
url = "http://proxy:8080"
# user = "username"     # optional
# password = "password" # optional
```
