<div>
<a href="https://docs.rs/vkteams-bot/latest/vkteams_bot/">
    <img src="https://img.shields.io/docsrs/vkteams-bot/latest" alt="docs.rs">
</a>
<a href="https://crates.io/crates/vkteams-bot">
    <img src="https://img.shields.io/crates/v/vkteams-bot" alt="crates.io">
</a>
 <a href="https://github.com/k05h31/vkteams-bot/actions">
    <img src="https://github.com/k05h31/vkteams-bot/workflows/Rust/badge.svg" alt="github.com">
</a>
 <a href="https://github.com/rust-secure-code/safety-dance/">
    <img src="https://img.shields.io/badge/unsafe-forbidden-success.svg" alt="Unsafe forbidden">
</a>
</div>

# VK Teams Bot API client

VK Teams Bot API client written in Rust.

## Table of Contents

- [Environment](#environment)
- [Usage](#usage-examples)

## Environment

1. Begin with bot API following [instructions](https://teams.vk.com/botapi/?lang=en)

2. Set environment variables or save in `.env` file

```bash
# Unix-like
$ export VKTEAMS_BOT_API_TOKEN=<Your token here> #require
$ export VKTEAMS_BOT_API_URL=<Your base api url> #require
$ export VKTEAMS_PROXY=<Proxy> #optional

# Windows
$ set VKTEAMS_BOT_API_TOKEN=<Your token here> #require
$ set VKTEAMS_BOT_API_URL=<Your base api url> #require
$ set VKTEAMS_PROXY=<Proxy> #optional
```

3. Put lines in you `Cargo.toml` file

```toml
[dependencies]
vkteams_bot = { version = "0.6", feature = ["full"] }
log = "0.4"
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