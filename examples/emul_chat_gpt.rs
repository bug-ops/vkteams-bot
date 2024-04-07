#[macro_use]
extern crate log;

use tokio::time::*;
use vkteams_bot::{self, api::types::*};

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().expect("unable to load .env file");
    // Initialize logger
    pretty_env_logger::init();
    info!("Starting...");
    // Send message like text generation
    send(&Bot::default()).await;
}
// Emulate passing text from LLM to API
async fn send(bot: &Bot) {
    let txt = String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
    Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
    Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.");
    let mut i: Option<MsgId> = None;
    let mut html_parser: MessageTextParser = Default::default();
    // Get chat_id from .env file
    let chat_id = ChatId(
        std::env::var("VKTEAMS_CHAT_ID")
            .expect("Unable to find VKTEAMS_CHAT_ID in .env file")
            .to_string(),
    );
    // Split text by words
    for word in txt.split_whitespace() {
        // Add plain text word-by-word into message
        html_parser
            .add(MessageTextFormat::Plain(word.to_string()))
            .space();
        match i.as_ref() {
            // Next words add by editing previous message
            Some(msg_id) => {
                bot.messages_edit_text(
                    chat_id.to_owned(),
                    msg_id.to_owned(),
                    Some(html_parser.to_owned()),
                )
                .await
                .unwrap();
            }
            // First word send as new message
            None => {
                let res = bot
                    .messages_send_text(
                        chat_id.to_owned(),
                        Some(html_parser.to_owned()),
                        None,
                        None,
                        None,
                        None,
                    )
                    .await
                    .unwrap();
                i = res.msg_id;
            }
        }
        // Bot action typing
        bot.chats_send_actions(chat_id.to_owned(), ChatActions::Typing)
            .await
            .unwrap();
        // Add every word with 300 millis delay
        sleep(Duration::from_millis(300)).await;
    }
    // Bot action looking for message
    bot.chats_send_actions(chat_id.to_owned(), ChatActions::Looking)
        .await
        .unwrap();
}
