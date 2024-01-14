<div>
<a href="https://docs.rs/vkteams-bot/latest/vkteams_bot/">
    <img src="https://img.shields.io/docsrs/vkteams-bot/latest">
</a>
<a href="https://crates.io/crates/vkteams-bot">
    <img src="https://img.shields.io/crates/v/vkteams-bot">
</a>
 <a href="https://github.com/k05h31/vkteams-bot/actions">
    <img src="https://github.com/k05h31/vkteams-bot/workflows/Rust/badge.svg">
</a>
</div>

# VK Teams Bot API client

VK Teams Bot API client written in Rust.

## Table of Contents

- [Environment](#environment)
- [Usage](#usage)

## Environment

1. Begin with bot API following [instructions](https://teams.vk.com/botapi/?lang=en)
2. Set environment variables or save in `.env` file
```bash
# Unix-like
$ export VKTEAMS_VKTEAMS_BOT_API_TOKEN=<Your token here> #require
$ export VKTEAMS_BOT_API_URL=<Your base api url> #require
$ export VKTEAMS_PROXY=<Proxy> #optional

# Windows
$ set VKTEAMS_VKTEAMS_BOT_API_TOKEN=<Your token here> #require
$ set VKTEAMS_BOT_API_URL=<Your base api url> #require
$ set VKTEAMS_PROXY=<Proxy> #optional
```
3. Put lines in you `Cargo.toml` file
```toml
[dependencies]
vkteams_bot = "0.3"
log = "0.4"
```

## Usage

```rust
#[macro_use]
extern crate log;
use vkteams_bot::{self, api::types::*};

#[tokio::main]
async fn main() {
    // CHeck .env file and init logger
    dotenvy::dotenv().expect("Unable to load .env file");
    pretty_env_logger::init();

    info!("Starting...");
    // Start bot with API version 1
    let bot = Bot::new(APIVersionUrl::V1);
    // Send answer to chat
    send_text_msg(&bot).await;
}

async fn send_text_msg(bot: &Bot) {
    // Send text message to chat
    // Get initial events from the API
    let chat_events = bot.get_events().await.unwrap();
    // Loop at all events
    for event in chat_events.events {
        // New keyboard
        let mut kb: Keyboard = Default::default();
        // New HTML parser
        let mut html_parser: MessageTextParser = Default::default();
        let chat = event.payload.chat.to_owned().unwrap();
        // Check event type
        match event.event_type {
            EventType::NewMessage => {
                kb.add_button(&ButtonKeyboard::cb(
                    String::from("test_callback"),      // Text
                    String::from("test_callback_data"), // Callback
                    ButtonStyle::Primary,
                ))
                .add_button(&ButtonKeyboard::url(
                    String::from("Example"),            // Text
                    String::from("https://example.com"),// Url
                    ButtonStyle::Attention,
                ));
                // Write HTML message
                html_parser
                    .add(MessageTextFormat::Bold(String::from("Test BOLD message")))
                    .next_line()
                    .add(MessageTextFormat::Link(
                        String::from("https://example.com"),
                        String::from("Example"),
                    ))
                    .next_line()
                    .add(MessageTextFormat::OrderedList(vec![
                        String::from("First"),
                        String::from("Second"),
                        String::from("Third"),
                    ]))
                    .next_line()
                    .add(MessageTextFormat::Mention(chat.chat_id.to_owned()));
                // Send message to chat
                bot.messages_send_text(
                    chat.chat_id.to_owned(),
                    Some(html_parser),
                    Some(kb),
                    None,
                    None,
                    None,).await.unwrap();
            }
            EventType::NewChatMembers => {
                // Remember self data
                let self_user_id = bot.self_get().await.unwrap().user_id.0;

                for member in event.payload.new_members.unwrap() {
                    // Check if self user is new chat member
                    if self_user_id == member.user_id.0 {
                        // Get chat admins list
                        let res = bot.chats_get_admins(chat.chat_id.to_owned()).await.unwrap();
                        // Check if self user is admin
                        for admin in res.admins.unwrap() {
                            if self_user_id == admin.user_id.0 {
                                // If self user is admin, set chat avatar
                                bot.chats_avatar_set(
                                    chat.chat_id.to_owned(),
                                    String::from("tests/test.jpg"),
                                )
                                .await
                                .unwrap();
                                break; // Exit from loop if self user is admin
                            };
                        }
                        break; // Exit from loop if self user is new chat member
                    }
                }
            }
            _ => {
                warn!("Not implmented EventType: {:?}. Skip", &event.event_type);
            }
        }
    }
}
```