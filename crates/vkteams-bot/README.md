# VK Teams Bot API client

[![docs.rs](https://img.shields.io/docsrs/vkteams-bot/latest)](https://docs.rs/vkteams-bot/latest/vkteams_bot/)
[![crates.io](https://img.shields.io/crates/v/vkteams-bot)](https://crates.io/crates/vkteams-bot)
[![codecov](https://codecov.io/github/bug-ops/vkteams-bot/graph/badge.svg?token=XV23ZKSZRA&flag=vkteams-bot)](https://codecov.io/github/bug-ops/vkteams-bot)
[![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

## Table of Contents

- [Environment](#environment)
- [Usage](#usage-examples)

## Environment

There are two ways to initialize the bot:

### Option 1: Using Environment Variables (Default)

1. Begin with bot API following [instructions](https://teams.vk.com/botapi/?lang=en)

2. Set environment variables or save in `.env` file

```bash
# Unix-like
$ export VKTEAMS_BOT_API_TOKEN=<Your token here> #require
$ export VKTEAMS_BOT_API_URL=<Your base api url> #require
$ export VKTEAMS_BOT_CONFIG=<Your bot config path> #optional
$ export VKTEAMS_PROXY=<Proxy> #optional

# Windows
$ set VKTEAMS_BOT_API_TOKEN=<Your token here> #require
$ set VKTEAMS_BOT_API_URL=<Your base api url> #require
$ set VKTEAMS_BOT_CONFIG=<Your bot config path> #optional
$ set VKTEAMS_PROXY=<Proxy> #optional
```

3. Put lines in you `Cargo.toml` file

```toml
[dependencies]
vkteams_bot = { version = "0.9", features = ["full"] }
```

### Option 2: Direct Parameter Passing

Alternatively, you can create the bot by directly passing parameters:

```rust
let bot = Bot::with_default_version(
    "your_bot_token".to_string(),
    "https://api.example.com".to_string()
)?;
```

## Usage examples

[Examples:](examples)

- [new message](examples/new_message.rs)
- [new message / edit message](examples/emul_chat_gpt.rs)
- [event listener](examples/event_listener.rs)
- [answer callback query](examples/callback_query.rs)
- [chat - get info](examples/chat_get_info.rs)
- [chat admin - avatar set](examples/chat_admin_avatar_set.rs)
- [chat - download files](examples/chat_get_file.rs)
- [bot - webhook handler](examples/prometheus_webhook.rs)
- [bot - rate limit](examples/ratelimit_background.rs)
