# JSON Output Documentation

This document provides comprehensive information about the JSON output functionality in VK Teams Bot CLI.

## Overview

Starting from version 0.7.0, all CLI commands support structured JSON output format. This feature enables:

- **Programmatic integration** with other tools and scripts
- **Reliable parsing** of command results
- **Consistent error handling** across all commands
- **Easy automation** of bot operations

## Enabling JSON Output

Use the `--output json` flag before any command:

```bash
vkteams-bot-cli --output json <command> [options]
```

## JSON Response Structure

All commands return a consistent JSON structure:

```json
{
  "success": boolean,
  "data": object | null,
  "error": string | null,
  "timestamp": "ISO 8601 timestamp",
  "command": "command-name"
}
```

### Field Descriptions

- **success**: `true` if the command executed successfully, `false` otherwise
- **data**: Command-specific response data (null on error)
- **error**: Error message if the command failed (null on success)
- **timestamp**: ISO 8601 formatted timestamp of when the command was executed
- **command**: The name of the executed command

## Command-Specific Responses

### Messaging Commands

#### send-text
```bash
vkteams-bot-cli --output json send-text -u USER_ID -m "Hello"
```

Response:
```json
{
  "success": true,
  "data": {
    "msgId": "123456789",
    "ok": true
  },
  "error": null,
  "timestamp": "2024-01-20T10:30:45Z",
  "command": "send-text"
}
```

#### send-file
```bash
vkteams-bot-cli --output json send-file -u USER_ID -p file.pdf
```

Response:
```json
{
  "success": true,
  "data": {
    "msgId": "123456790",
    "ok": true,
    "fileId": "file_abc123"
  },
  "error": null,
  "timestamp": "2024-01-20T10:31:00Z",
  "command": "send-file"
}
```

### Chat Commands

#### get-chat-info
```bash
vkteams-bot-cli --output json get-chat-info -c CHAT_ID
```

Response:
```json
{
  "success": true,
  "data": {
    "type": "group",
    "title": "Project Chat",
    "about": "Discussion for the project",
    "public": false,
    "joinLink": null,
    "inviteLink": "https://...",
    "admins": ["admin1", "admin2"]
  },
  "error": null,
  "timestamp": "2024-01-20T10:32:00Z",
  "command": "get-chat-info"
}
```

### Scheduling Commands

#### scheduler list
```bash
vkteams-bot-cli --output json scheduler list
```

Response:
```json
{
  "success": true,
  "data": {
    "tasks": [
      {
        "id": "task-123",
        "enabled": true,
        "task_type": "Send text to chat456",
        "schedule": "Every day at 09:00",
        "run_count": 5,
        "max_runs": "unlimited",
        "next_run": "2024-01-21 09:00:00 UTC",
        "last_run": "2024-01-20 09:00:00 UTC",
        "created_at": "2024-01-15 10:00:00 UTC"
      }
    ],
    "total": 1
  },
  "error": null,
  "timestamp": "2024-01-20T10:33:00Z",
  "command": "scheduler-list"
}
```

### Diagnostic Commands

#### health-check
```bash
vkteams-bot-cli --output json health-check
```

Response:
```json
{
  "success": true,
  "data": {
    "ok": true,
    "userId": "bot123",
    "nick": "MyBot",
    "firstName": "VK Teams",
    "about": "Official bot",
    "photo": [
      {
        "url": "https://..."
      }
    ]
  },
  "error": null,
  "timestamp": "2024-01-20T10:34:00Z",
  "command": "health-check"
}
```

## Error Responses

When a command fails, the response structure remains consistent:

```json
{
  "success": false,
  "data": null,
  "error": "Detailed error message",
  "timestamp": "2024-01-20T10:35:00Z",
  "command": "failed-command"
}
```

## Using with jq

[jq](https://stedolan.github.io/jq/) is a powerful command-line JSON processor that works perfectly with our JSON output.

### Basic Examples

```bash
# Extract a specific field
vkteams-bot-cli --output json get-chat-info -c CHAT_ID | jq '.data.title'

# Check command success
vkteams-bot-cli --output json send-text -u USER -m "Test" | jq '.success'

# Get error message on failure
vkteams-bot-cli --output json invalid-command | jq '.error'

# Pretty print specific data
vkteams-bot-cli --output json scheduler list | jq '.data.tasks'
```

### Advanced Examples

```bash
# Filter active tasks
vkteams-bot-cli --output json scheduler list | \
  jq '.data.tasks[] | select(.enabled == true)'

# Count total messages sent today
vkteams-bot-cli --output json get-events | \
  jq '[.data.events[] | select(.type == "newMessage")] | length'

# Extract all chat IDs
vkteams-bot-cli --output json list-chats | \
  jq -r '.data[].chatId'

# Create CSV from task list
vkteams-bot-cli --output json scheduler list | \
  jq -r '.data.tasks[] | [.id, .task_type, .enabled, .next_run] | @csv'
```

## Shell Scripting Examples

### Error Handling
```bash
#!/bin/bash
RESULT=$(vkteams-bot-cli --output json send-text -u USER -m "Hello")

if [ $(echo $RESULT | jq -r '.success') == "true" ]; then
    MSG_ID=$(echo $RESULT | jq -r '.data.msgId')
    echo "Message sent successfully: $MSG_ID"
else
    ERROR=$(echo $RESULT | jq -r '.error')
    echo "Failed to send message: $ERROR" >&2
    exit 1
fi
```

### Batch Processing
```bash
#!/bin/bash
# Send messages to multiple users
USERS=("user1" "user2" "user3")

for user in "${USERS[@]}"; do
    RESULT=$(vkteams-bot-cli --output json send-text -u "$user" -m "Announcement")
    
    if [ $(echo $RESULT | jq -r '.success') == "true" ]; then
        echo "✓ Sent to $user"
    else
        echo "✗ Failed to send to $user: $(echo $RESULT | jq -r '.error')"
    fi
done
```

### Monitoring Script
```bash
#!/bin/bash
# Health check monitoring
while true; do
    HEALTH=$(vkteams-bot-cli --output json health-check)
    TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
    
    if [ $(echo $HEALTH | jq -r '.success') == "true" ]; then
        echo "[$TIMESTAMP] Bot is healthy"
    else
        echo "[$TIMESTAMP] ALERT: Bot health check failed!"
        # Send alert notification
        vkteams-bot-cli send-text -u admin-chat -m "Bot health check failed at $TIMESTAMP"
    fi
    
    sleep 300  # Check every 5 minutes
done
```

## Integration with Other Tools

### Python Example
```python
import json
import subprocess

def send_message(user_id, message):
    cmd = ['vkteams-bot-cli', '--output', 'json', 'send-text', '-u', user_id, '-m', message]
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    data = json.loads(result.stdout)
    
    if data['success']:
        return data['data']['msgId']
    else:
        raise Exception(f"Failed to send message: {data['error']}")

# Usage
try:
    msg_id = send_message("user123", "Hello from Python!")
    print(f"Message sent: {msg_id}")
except Exception as e:
    print(f"Error: {e}")
```

### Node.js Example
```javascript
const { exec } = require('child_process');
const util = require('util');
const execPromise = util.promisify(exec);

async function getScheduledTasks() {
    try {
        const { stdout } = await execPromise('vkteams-bot-cli --output json scheduler list');
        const result = JSON.parse(stdout);
        
        if (result.success) {
            return result.data.tasks;
        } else {
            throw new Error(result.error);
        }
    } catch (error) {
        console.error('Failed to get tasks:', error);
        throw error;
    }
}

// Usage
getScheduledTasks()
    .then(tasks => {
        console.log(`Found ${tasks.length} scheduled tasks`);
        tasks.forEach(task => {
            console.log(`- ${task.id}: ${task.task_type}`);
        });
    })
    .catch(console.error);
```

## Best Practices

1. **Always check the `success` field** before processing `data`
2. **Handle both success and error cases** in your scripts
3. **Use jq for complex data extraction** instead of text parsing
4. **Store results in variables** for multiple field extraction
5. **Use `--output json` consistently** in automated scripts
6. **Log timestamps** from responses for debugging
7. **Validate data existence** before accessing nested fields

## Troubleshooting

### Common Issues

1. **Parsing errors**: Ensure you're using `--output json` flag correctly
   ```bash
   # Correct
   vkteams-bot-cli --output json send-text -u USER -m "Test"
   
   # Incorrect (flag after command)
   vkteams-bot-cli send-text --output json -u USER -m "Test"
   ```

2. **Empty responses**: Check if the command requires authentication
   ```bash
   # Verify bot configuration
   vkteams-bot-cli --output json health-check
   ```

3. **jq errors**: Validate JSON structure first
   ```bash
   # Check if output is valid JSON
   vkteams-bot-cli --output json get-events | jq '.'
   ```

## Version Compatibility

JSON output is available starting from:
- vkteams-bot-cli: v0.7.0
- Requires vkteams-bot: v0.11.0 or higher

## See Also

- [VK Teams Bot CLI README](../README.md)
- [jq Manual](https://stedolan.github.io/jq/manual/)
- [Command Reference](COMMANDS.md)