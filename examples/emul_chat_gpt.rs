#[macro_use]
extern crate log;

use tokio::time::*;
use vkteams_bot::prelude::*;

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
    const DEFAULT_STRING: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
    Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
    Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
    // Initializse mutable variables
    let mut id = MsgId(String::new());
    let mut html_parser = MessageTextParser::new();
    // Get chat_id from .env
    let chat_id = ChatId(
        std::env::var("VKTEAMS_CHAT_ID")
            .expect("Unable to find VKTEAMS_CHAT_ID in .env file")
            .to_string(),
    );
    // Split text by words
    for word in DEFAULT_STRING.split_whitespace() {
        // Add plain text word-by-word into message
        html_parser
            .add(MessageTextFormat::Plain(word.to_string()))
            .space();
        if id.0.is_empty() {
            // First word send by creating new message
            id = bot
                .send_api_request(
                    RequestMessagesSendText::new(chat_id.to_owned())
                        .set_text(html_parser.to_owned()),
                )
                .await
                .unwrap()
                .msg_id;
        } else {
            // Next words add by editing previous message
            bot.send_api_request(
                RequestMessagesEditText::new(chat_id.to_owned(), id.to_owned())
                    .set_text(html_parser.to_owned()),
            )
            .await
            .unwrap();
        };
        // Bot action typing
        match bot
            .send_api_request(RequestChatsSendAction::new(
                chat_id.to_owned(),
                ChatActions::Typing,
            ))
            .await
        {
            Ok(_) => {}
            Err(e) => {
                error!("Error: {}", e);
            }
        }
        // Add every word with 300 millis delay
        sleep(Duration::from_millis(300)).await;
    }
    // Bot action looking for message
    bot.send_api_request(RequestChatsSendAction::new(
        chat_id.to_owned(),
        ChatActions::Looking,
    ))
    .await
    .unwrap();
}
