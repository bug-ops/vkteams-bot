# VK Teams Bot API Cli

[![crates.io](https://img.shields.io/crates/v/vkteams-bot-cli)](https://crates.io/crates/vkteams-bot-cli)
[![codecov](https://codecov.io/github/bug-ops/vkteams-bot/graph/badge.svg?token=XV23ZKSZRA&flag=vkteams-bot-cli)](https://codecov.io/github/bug-ops/vkteams-bot)
[![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

🤖 **Powerful command-line interface for VK Teams Bot API** with interactive setup, progress bars, and comprehensive chat management features.

## ✨ Features

- 📤 **Message Operations**: Send text, files, and voice messages
- 💬 **Chat Management**: Get info, manage members, set titles and descriptions
- 🔧 **Message Editing**: Edit, delete, pin/unpin messages
- 📁 **File Handling**: Upload and download files with progress bars
- 🎯 **Event Monitoring**: Real-time event listening with long polling
- ⏰ **Message Scheduling**: Schedule messages with cron expressions, intervals, or specific times
- 🤖 **Task Automation**: Background scheduler daemon with task management
- ⚙️ **Smart Configuration**: Interactive setup wizard and multiple config sources
- 🔍 **Diagnostics**: Built-in validation and connection testing
- 🎨 **User-Friendly**: Colored output, progress indicators, and helpful examples

## Table of Contents

- [Installation](#installation)
- [Shell Completion](#shell-completion)
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

## Shell Completion

Shell completion provides intelligent Tab completion for all commands, options, and file paths.

### 🚀 One-Command Setup

```bash
# Build CLI
cargo build --release

# Install completions using the CLI
vkteams-bot-cli completion bash --install
vkteams-bot-cli completion zsh --install
```

### What You Get

After installation, use Tab completion for:

- **Commands**: `vkteams-bot-cli s[Tab]` → `send-text`, `send-file`, `schedule`
- **Options**: `vkteams-bot-cli config --[Tab]` → `--show`, `--wizard`, `--help`
- **File paths**: `vkteams-bot-cli send-file -p /path/[Tab]` → auto-complete files

### Runtime Generation

```bash
# Generate completion for your shell
vkteams-bot-cli completion bash --output completion.bash
vkteams-bot-cli completion zsh --install

# Generate to stdout
vkteams-bot-cli completion bash
```

### Manual Installation

#### Bash

```bash
# Generate and source the completion script
vkteams-bot-cli completion bash > ~/.local/share/bash-completion/completions/vkteams-bot-cli

# Add to your ~/.bashrc
echo 'source ~/.local/share/bash-completion/completions/vkteams-bot-cli' >> ~/.bashrc
```

#### Zsh

```bash
# Generate completion script
vkteams-bot-cli completion zsh > ~/.local/share/zsh/site-functions/_vkteams-bot-cli

# Ensure completions are enabled in ~/.zshrc
echo 'autoload -Uz compinit && compinit' >> ~/.zshrc
```

#### Fish

```bash
# Fish automatically loads completions from this directory
vkteams-bot-cli completion fish > ~/.config/fish/completions/vkteams-bot-cli.fish
```

#### PowerShell

```powershell
# Generate completion script
vkteams-bot-cli completion powershell > vkteams-bot-cli-completion.ps1

# Add to your PowerShell profile
Add-Content $PROFILE ". $(pwd)\vkteams-bot-cli-completion.ps1"
```

### Build from Source with Completions

```bash
# Clone and build
git clone https://github.com/bug-ops/vkteams-bot
cd vkteams-bot/crates/vkteams-bot-cli
cargo build --release

# Install completions using the CLI
./target/release/vkteams-bot-cli completion bash --install
./target/release/vkteams-bot-cli completion zsh --install

# Completions are now available in your shell! 🎉
```

> 📋 **Build Guide**: See [BUILD.md](BUILD.md) for detailed build instructions, troubleshooting, and advanced options.

## Quick Start

| `completion` | Generate shell completions | `vkteams-bot-cli completion bash` |

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

### 5. Set Up Shell Completion (Optional)

```bash
# Install for your shell
vkteams-bot-cli completion bash --install
vkteams-bot-cli completion zsh --install
vkteams-bot-cli completion fish --install

# Or generate to file
vkteams-bot-cli completion bash > ~/.local/share/bash-completion/completions/vkteams-bot-cli
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

### ⏰ Message Scheduling

| Command | Description | Example |
|---------|-------------|---------|
| `schedule text` | Schedule text message | `vkteams-bot-cli schedule text -u CHAT_ID -m "Hello" -t "2024-01-01 10:00"` |
| `schedule file` | Schedule file message | `vkteams-bot-cli schedule file -u CHAT_ID -p file.pdf -c "0 9 * * *"` |
| `schedule voice` | Schedule voice message | `vkteams-bot-cli schedule voice -u CHAT_ID -p voice.ogg -i 3600` |
| `schedule action` | Schedule chat action | `vkteams-bot-cli schedule action -u CHAT_ID -a typing -t "30m"` |

### 🔧 Scheduler Management

| Command | Description | Example |
|---------|-------------|---------|
| `scheduler start` | Start scheduler daemon | `vkteams-bot-cli scheduler start` |
| `scheduler stop` | Stop scheduler daemon | `vkteams-bot-cli scheduler stop` |
| `scheduler status` | Show scheduler status | `vkteams-bot-cli scheduler status` |
| `scheduler list` | List all scheduled tasks | `vkteams-bot-cli scheduler list` |

### 📋 Task Management

| Command | Description | Example |
|---------|-------------|---------|
| `task show` | Show task details | `vkteams-bot-cli task show TASK_ID` |
| `task remove` | Remove scheduled task | `vkteams-bot-cli task remove TASK_ID` |
| `task enable` | Enable disabled task | `vkteams-bot-cli task enable TASK_ID` |
| `task disable` | Disable active task | `vkteams-bot-cli task disable TASK_ID` |
| `task run` | Run task immediately | `vkteams-bot-cli task run TASK_ID` |

### ⚙️ Configuration

| Command | Description | Example |
|---------|-------------|---------|
| `setup` | Interactive setup wizard | `vkteams-bot-cli setup` |
| `config --show` | Show current config | `vkteams-bot-cli config --show` |
| `config --wizard` | Update config interactively | `vkteams-bot-cli config --wizard` |
| `completion` | Generate shell completions | `vkteams-bot-cli completion bash` |

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

### Message Scheduling

```bash
# Schedule a text message for a specific date and time
vkteams-bot-cli schedule text -u chat456 -m "Meeting reminder" -t "2024-01-15 09:00:00"

# Schedule a daily message using cron expression (9 AM every day)
vkteams-bot-cli schedule text -u chat456 -m "Daily standup time!" -c "0 9 * * *"

# Schedule a file to be sent every hour
vkteams-bot-cli schedule file -u chat456 -p "/path/to/report.pdf" -i 3600

# Schedule a voice message for tomorrow
vkteams-bot-cli schedule voice -u chat456 -p "/path/to/announcement.ogg" -t "tomorrow"

# Schedule a typing action in 30 minutes
vkteams-bot-cli schedule action -u chat456 -a typing -t "30m"

# Schedule a weekly reminder (every Monday at 10 AM)
vkteams-bot-cli schedule text -u chat456 -m "Weekly team meeting in 1 hour" -c "0 10 * * 1"

# Schedule a one-time message with limited runs
vkteams-bot-cli schedule text -u chat456 -m "Limited offer!" -t "2h" --max-runs 1
```

### Scheduler Management

```bash
# Start the scheduler daemon (runs in background)
vkteams-bot-cli scheduler start

# Check scheduler status and active tasks
vkteams-bot-cli scheduler status

# List all scheduled tasks with details
vkteams-bot-cli scheduler list

# Stop the scheduler daemon
vkteams-bot-cli scheduler stop
```

### Task Management

```bash
# Show detailed information about a specific task
vkteams-bot-cli task show a1b2c3d4-e5f6-7890-abcd-ef1234567890

# Run a scheduled task immediately (one-time execution)
vkteams-bot-cli task run a1b2c3d4-e5f6-7890-abcd-ef1234567890

# Temporarily disable a task
vkteams-bot-cli task disable a1b2c3d4-e5f6-7890-abcd-ef1234567890

# Re-enable a disabled task
vkteams-bot-cli task enable a1b2c3d4-e5f6-7890-abcd-ef1234567890

# Permanently remove a task
vkteams-bot-cli task remove a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

### Advanced Scheduling Examples

```bash
# Business hours notifications (Monday to Friday, 9 AM to 5 PM)
vkteams-bot-cli schedule text -u team-chat -m "Working hours reminder" -c "0 9-17 * * 1-5"

# End of month reports (last day of every month at 6 PM)
vkteams-bot-cli schedule file -u reports-chat -p "/reports/monthly.pdf" -c "0 18 L * *"

# Hourly health checks during business hours
vkteams-bot-cli schedule text -u monitoring-chat -m "System health check" -c "0 9-17 * * 1-5"

# Multiple interval scheduling
vkteams-bot-cli schedule text -u alerts-chat -m "Every 5 minutes alert" -i 300
vkteams-bot-cli schedule text -u daily-chat -m "Every 30 minutes update" -i 1800

# Relative time scheduling
vkteams-bot-cli schedule text -u chat456 -m "In 1 hour" -t "1h"
vkteams-bot-cli schedule text -u chat456 -m "Tomorrow morning" -t "tomorrow"
vkteams-bot-cli schedule text -u chat456 -m "Next week" -t "7d"
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

# Generate shell completions for better UX
vkteams-bot-cli completion bash --install
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

### Scheduler Automation Scripts

```bash
#!/bin/bash
# Setup automated notifications for a development team

TEAM_CHAT="dev-team-chat"
ALERTS_CHAT="alerts-chat"

# Daily standup reminder
vkteams-bot-cli schedule text \
  -u "$TEAM_CHAT" \
  -m "🏃‍♂️ Daily standup in 15 minutes! Please prepare your updates." \
  -c "0 9 * * 1-5"

# End of sprint reminders
vkteams-bot-cli schedule text \
  -u "$TEAM_CHAT" \
  -m "📊 Sprint ends tomorrow. Please update your task status." \
  -c "0 17 * * 4"

# Weekly retrospective
vkteams-bot-cli schedule text \
  -u "$TEAM_CHAT" \
  -m "🔄 Weekly retrospective today at 4 PM. What went well this week?" \
  -c "0 16 * * 5"

# System maintenance notifications
vkteams-bot-cli schedule text \
  -u "$ALERTS_CHAT" \
  -m "🔧 Scheduled maintenance window starts in 1 hour" \
  -t "2024-01-20 01:00:00"

echo "All scheduled notifications have been set up!"
```

```bash
#!/bin/bash
# Scheduler management script

case "$1" in
  start)
    echo "Starting VK Teams Bot scheduler..."
    vkteams-bot-cli scheduler start
    ;;
  stop)
    echo "Stopping VK Teams Bot scheduler..."
    vkteams-bot-cli scheduler stop
    ;;
  status)
    echo "Checking scheduler status..."
    vkteams-bot-cli scheduler status
    ;;
  list)
    echo "Listing all scheduled tasks..."
    vkteams-bot-cli scheduler list
    ;;
  *)
    echo "Usage: $0 {start|stop|status|list}"
    exit 1
    ;;
esac
```

### Environment-Specific Configuration

```bash
# Environment
export VKTEAMS_BOT_API_URL="https://api.example.com"
export VKTEAMS_LOG_LEVEL="info"

# Validate current setup
vkteams-bot-cli validate

# Set up completion for development environment
vkteams-bot-cli completion bash --install
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

# Install shell completion for better command discovery
vkteams-bot-cli completion bash --install
vkteams-bot-cli completion zsh --install
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
url = "https://example.com"
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

[rate_limit]
enabled = false        # Disable rate limiting for CLI usage
limit = 1000          # Maximum requests per time window
duration = 60         # Time window in seconds
retry_delay = 500     # Delay between retries in milliseconds
retry_attempts = 3    # Maximum retry attempts

[scheduler]
data_file = "/home/user/.config/vkteams-bot/scheduler_tasks.json"
check_interval = 60    # seconds
max_task_history = 100
```

## Scheduler Features

### Schedule Types

The CLI supports three types of scheduling:

1. **One-time scheduling** - Execute once at a specific time
2. **Cron-based scheduling** - Use cron expressions for complex recurring schedules
3. **Interval-based scheduling** - Repeat every N seconds/minutes/hours

### Time Formats

**Absolute time:**

- `2024-01-15 14:30:00` - Specific date and time
- `2024-01-15 14:30` - Date and time (seconds default to 00)
- `2024-01-15` - Date only (time defaults to 00:00:00)

**Relative time:**

- `30s` - 30 seconds from now
- `5m` - 5 minutes from now
- `2h` - 2 hours from now
- `1d` - 1 day from now
- `1w` - 1 week from now
- `tomorrow` - Tomorrow at the same time
- `now` - Right now

**Cron expressions:**

- `0 9 * * *` - Every day at 9:00 AM
- `0 */2 * * *` - Every 2 hours
- `0 9 * * 1-5` - Weekdays at 9:00 AM
- `0 0 1 * *` - First day of every month at midnight
- `*/30 * * * *` - Every 30 minutes

### Task Management

Each scheduled task has:

- **Unique ID** - UUID for task identification
- **Status** - Active/disabled state
- **Run count** - Number of times executed
- **Next run time** - When the task will run next
- **Maximum runs** - Optional limit on executions

### Scheduler Daemon

The scheduler runs as a background process that:

- Checks for due tasks every minute
- Executes tasks at their scheduled time
- Manages task state and run counts
- Automatically disables completed one-time tasks
- Persists task data between restarts
