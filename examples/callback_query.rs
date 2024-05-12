#[macro_use]
extern crate log;

use vkteams_bot::{self, api::types::*};

const CALLBACK_DATA: &str = "callback_button_#1";
const CALLBACK_TEXT: &str = "callback_text";
const VKTEAMS_CHAT_ID: &str = "VKTEAMS_CHAT_ID";

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().expect("unable to load .env file");
    // Initialize logger
    pretty_env_logger::init();
    info!("Starting...");
    // Make bot
    let bot = Bot::default();
    let mut html_parser: MessageTextParser = Default::default();
    // Get chat_id from .env file
    let chat_id = ChatId(
        std::env::var(VKTEAMS_CHAT_ID)
            .expect("Unable to find VKTEAMS_CHAT_ID in .env file")
            .to_string(),
    );
    // Add text
    html_parser
        .add(MessageTextFormat::Plain("Push the button".to_string()))
        .space();
    // Add button
    let keyboard = Keyboard::default()
        .add_button(&ButtonKeyboard::cb(
            CALLBACK_TEXT.to_string(),
            CALLBACK_DATA.to_string(), // Callback data
            ButtonStyle::Primary,
        ))
        .to_owned();

    bot.messages_send_text(chat_id, Some(html_parser), Some(keyboard), None, None, None)
        .await
        .unwrap();
    // Start event listener and pass result to a callback function
    bot.event_listener(callback).await;
}

// Callback function to print out the result
pub async fn callback(bot: Bot, res: ResponseEventsGet) {
    // Get events type callback query
    let events = res.events.iter().filter(|&e| {
        e.event_type == EventType::CallbackQuery
            && e.payload.query_id.is_some()
            && e.payload.callback_data.is_some()
    });
    // Answer callback query
    for event in events {
        match bot
            .messages_answer_callback_query(
                event.payload.query_id.to_owned().unwrap(),
                Some(
                    match event.payload.callback_data.as_ref().unwrap().as_str() {
                        CALLBACK_DATA => "Button pressed!",
                        _ => "WRONG button pressed!",
                    }
                    .to_string(),
                ),
                Some(ShowAlert(true)),
                None,
            )
            .await
        {
            Ok(_) => info!("Callback query answered"),
            Err(e) => error!("{:?}", e),
        }
    }
}
