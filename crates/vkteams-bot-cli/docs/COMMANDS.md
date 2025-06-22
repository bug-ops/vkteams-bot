# VK Teams Bot CLI Command Reference

This document provides a comprehensive reference for all available commands in the VK Teams Bot CLI.

## Global Options

These options can be used with any command:

| Option | Description | Default |
|--------|-------------|---------|
| `--config PATH` | Use custom config file | Auto-detected |
| `--save-config PATH` | Save current config to file | - |
| `--verbose` | Enable verbose logging | false |
| `--output FORMAT` | Output format (pretty, json, table, quiet) | pretty |

## Command Categories

1. [Messaging Commands](#messaging-commands)
2. [Chat Commands](#chat-commands)
3. [Scheduling Commands](#scheduling-commands)
4. [File Commands](#file-commands)
5. [Configuration Commands](#configuration-commands)
6. [Diagnostic Commands](#diagnostic-commands)
7. [Storage Commands](#storage-commands)
8. [Daemon Commands](#daemon-commands)

---

## Messaging Commands

### send-text
Send a text message to a user or chat.

```bash
vkteams-bot-cli send-text -u USER_ID -m "Message text"
```

**Options:**
- `-u, --user-id USER_ID` (required) - Recipient user or chat ID
- `-m, --message TEXT` (required) - Message text to send
- `-f, --format FORMAT` - Message format (MarkdownV2, HTML)
- `--reply-to MSG_ID` - Reply to specific message
- `--forward MSG_ID CHAT_ID` - Forward message from chat

**JSON Response:**
```json
{
  "success": true,
  "data": {
    "msgId": "123456789",
    "ok": true
  }
}
```

### send-file
Send a file to a user or chat with upload progress.

```bash
vkteams-bot-cli send-file -u USER_ID -p /path/to/file.pdf
```

**Options:**
- `-u, --user-id USER_ID` (required) - Recipient user or chat ID
- `-p, --path FILE_PATH` (required) - Path to file to send
- `-c, --caption TEXT` - File caption
- `--reply-to MSG_ID` - Reply to specific message

**JSON Response:**
```json
{
  "success": true,
  "data": {
    "msgId": "123456790",
    "ok": true,
    "fileId": "file_abc123"
  }
}
```

### send-voice
Send a voice message.

```bash
vkteams-bot-cli send-voice -u USER_ID -p /path/to/voice.ogg
```

**Options:**
- `-u, --user-id USER_ID` (required) - Recipient user or chat ID
- `-p, --path FILE_PATH` (required) - Path to voice file
- `--reply-to MSG_ID` - Reply to specific message

**Supported formats:** OGG, MP3, WAV, M4A, AAC

### edit-message
Edit an existing message.

```bash
vkteams-bot-cli edit-message -c CHAT_ID -m MSG_ID -t "New text"
```

**Options:**
- `-c, --chat-id CHAT_ID` (required) - Chat containing the message
- `-m, --msg-id MSG_ID` (required) - Message ID to edit
- `-t, --text TEXT` (required) - New message text
- `-f, --format FORMAT` - Message format

### delete-message
Delete a message.

```bash
vkteams-bot-cli delete-message -c CHAT_ID -m MSG_ID
```

**Options:**
- `-c, --chat-id CHAT_ID` (required) - Chat containing the message
- `-m, --msg-id MSG_ID` (required) - Message ID to delete

### pin-message / unpin-message
Pin or unpin a message in a chat.

```bash
vkteams-bot-cli pin-message -c CHAT_ID -m MSG_ID
vkteams-bot-cli unpin-message -c CHAT_ID -m MSG_ID
```

---

## Chat Commands

### get-chat-info
Get detailed information about a chat.

```bash
vkteams-bot-cli get-chat-info -c CHAT_ID
```

**JSON Response:**
```json
{
  "success": true,
  "data": {
    "type": "group",
    "title": "Project Chat",
    "about": "Team discussion",
    "public": false,
    "inviteLink": "https://...",
    "admins": ["admin1", "admin2"]
  }
}
```

### get-chat-members
Get list of chat members.

```bash
vkteams-bot-cli get-chat-members -c CHAT_ID
```

**Options:**
- `-c, --chat-id CHAT_ID` (required) - Chat ID
- `--cursor CURSOR` - Pagination cursor

### get-profile
Get user profile information.

```bash
vkteams-bot-cli get-profile -u USER_ID
```

### send-action
Send a chat action (typing indicator).

```bash
vkteams-bot-cli send-action -c CHAT_ID -a typing
```

**Available actions:**
- `typing` - Typing indicator
- `looking` - Looking at chat

### set-chat-title
Change chat title.

```bash
vkteams-bot-cli set-chat-title -c CHAT_ID -t "New Title"
```

### set-chat-about
Change chat description.

```bash
vkteams-bot-cli set-chat-about -c CHAT_ID -a "New description"
```

### set-chat-rules
Set chat rules.

```bash
vkteams-bot-cli set-chat-rules -c CHAT_ID -r "1. Be respectful\n2. No spam"
```

---

## Scheduling Commands

### schedule
Schedule messages to be sent later.

#### Schedule Text Message
```bash
vkteams-bot-cli schedule text -u CHAT_ID -m "Message" -t "2024-01-20 10:00:00"
```

**Time Options (mutually exclusive):**
- `-t, --time TIME` - Specific time (ISO format or relative like "1h", "tomorrow")
- `-c, --cron EXPR` - Cron expression for recurring
- `-i, --interval SECONDS` - Repeat interval in seconds

**Additional Options:**
- `--max-runs N` - Maximum number of executions

#### Schedule File
```bash
vkteams-bot-cli schedule file -u CHAT_ID -p /path/to/file -c "0 9 * * *"
```

#### Schedule Voice
```bash
vkteams-bot-cli schedule voice -u CHAT_ID -p /path/to/voice.ogg -i 3600
```

#### Schedule Action
```bash
vkteams-bot-cli schedule action -u CHAT_ID -a typing -t "30m"
```

### scheduler
Manage the scheduler daemon.

#### Start Scheduler
```bash
vkteams-bot-cli scheduler start
```

#### Stop Scheduler
```bash
vkteams-bot-cli scheduler stop
```

#### Check Status
```bash
vkteams-bot-cli scheduler status
```

**JSON Response:**
```json
{
  "success": true,
  "data": {
    "daemon_status": "running",
    "daemon_info": {
      "pid": 12345,
      "started_at": "2024-01-20 08:00:00 UTC"
    },
    "total_tasks": 5,
    "enabled_tasks": 4,
    "disabled_tasks": 1
  }
}
```

#### List Tasks
```bash
vkteams-bot-cli scheduler list
```

### task
Manage scheduled tasks.

#### Show Task
```bash
vkteams-bot-cli task show TASK_ID
```

#### Remove Task
```bash
vkteams-bot-cli task remove TASK_ID
```

#### Enable/Disable Task
```bash
vkteams-bot-cli task enable TASK_ID
vkteams-bot-cli task disable TASK_ID
```

#### Run Task Once
```bash
vkteams-bot-cli task run TASK_ID
```

---

## File Commands

### get-file
Download a file.

```bash
vkteams-bot-cli get-file -f FILE_ID -p /download/path/
```

**Options:**
- `-f, --file-id FILE_ID` (required) - File ID to download
- `-p, --path PATH` - Download directory (default: configured download dir)

---

## Configuration Commands

### setup
Interactive configuration wizard for first-time setup.

```bash
vkteams-bot-cli setup
```

### config
Manage CLI configuration.

```bash
# Show current configuration
vkteams-bot-cli config --show

# Initialize new config file
vkteams-bot-cli config --init

# Interactive configuration wizard
vkteams-bot-cli config --wizard
```

### examples
Show usage examples.

```bash
vkteams-bot-cli examples
```

### list-commands
Show all available commands with descriptions.

```bash
vkteams-bot-cli list-commands
```

### validate
Validate configuration and test bot connection.

```bash
vkteams-bot-cli validate
```

### completion
Generate shell completion scripts.

```bash
# Generate for specific shell
vkteams-bot-cli completion bash
vkteams-bot-cli completion zsh
vkteams-bot-cli completion fish
vkteams-bot-cli completion powershell

# Save to file
vkteams-bot-cli completion bash -o ~/.bash_completion.d/vkteams-bot-cli

# Install system-wide
vkteams-bot-cli completion bash --install

# Generate for all shells
vkteams-bot-cli completion bash --all
```

---

## Diagnostic Commands

### health-check
Check bot health and connection.

```bash
vkteams-bot-cli health-check
```

**Options:**
- `-d, --detailed` - Show detailed bot information

**JSON Response:**
```json
{
  "success": true,
  "data": {
    "ok": true,
    "userId": "bot123",
    "nick": "MyBot",
    "firstName": "VK Teams Bot",
    "about": "Official bot"
  }
}
```

### get-self
Get bot's own information.

```bash
vkteams-bot-cli get-self --detailed
```

### api-info
Show API endpoints and configuration.

```bash
vkteams-bot-cli api-info
```

### test-connection
Test connection to VK Teams API.

```bash
vkteams-bot-cli test-connection
```

### system-info
Show system and environment information.

```bash
vkteams-bot-cli system-info
```

### verify-webhook
Test webhook configuration.

```bash
vkteams-bot-cli verify-webhook -u https://myserver.com/webhook
```

### get-events
Get events from the bot's event stream.

```bash
# Get events once
vkteams-bot-cli get-events

# Long polling mode
vkteams-bot-cli get-events -l true
```

---

## Storage Commands

### storage
Manage vector storage for embeddings.

#### Initialize Storage
```bash
vkteams-bot-cli storage init
```

#### Show Information
```bash
vkteams-bot-cli storage info
```

#### Store Document
```bash
vkteams-bot-cli storage store -d "Document content" -m "metadata"
```

#### Search Similar
```bash
vkteams-bot-cli storage search -q "search query" -l 10
```

#### Get Document
```bash
vkteams-bot-cli storage get -i DOCUMENT_ID
```

#### Delete Document
```bash
vkteams-bot-cli storage delete -i DOCUMENT_ID
```

#### List Documents
```bash
vkteams-bot-cli storage list -l 20 -o 0
```

#### Clear Storage
```bash
vkteams-bot-cli storage clear --confirm
```

---

## Daemon Commands

### daemon
Run the bot in daemon mode for webhook handling.

```bash
# Start daemon with default settings
vkteams-bot-cli daemon start

# Start with custom webhook URL
vkteams-bot-cli daemon start --webhook-url https://myserver.com/webhook

# Start on specific port
vkteams-bot-cli daemon start --port 8080

# Enable hot reload
vkteams-bot-cli daemon start --hot-reload
```

**Options:**
- `--webhook-url URL` - Webhook URL for receiving events
- `--port PORT` - Port to listen on (default: 8080)
- `--hot-reload` - Enable configuration hot reload
- `--health-endpoint PATH` - Health check endpoint (default: /health)

---

## Exit Codes

The CLI uses the following exit codes:

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Connection error |
| 4 | API error |
| 5 | File I/O error |
| 6 | Invalid input |
| 7 | Timeout |
| 8 | Feature not supported |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `VKTEAMS_BOT_API_TOKEN` | Bot API token |
| `VKTEAMS_BOT_API_URL` | API base URL |
| `VKTEAMS_PROXY` | HTTP proxy URL |
| `VKTEAMS_LOG_LEVEL` | Log level (trace, debug, info, warn, error) |
| `VKTEAMS_DOWNLOAD_DIR` | Default download directory |
| `VKTEAMS_CONFIG_PATH` | Custom config file path |

## See Also

- [JSON Output Documentation](JSON_OUTPUT.md)
- [Configuration Guide](../README.md#configuration)
- [Examples](../README.md#examples)