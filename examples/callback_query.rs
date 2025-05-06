#[macro_use]
extern crate log;
use vkteams_bot::prelude::*;

const CALLBACK_DATA: &str = "callback_button_#1";
const CALLBACK_TEXT: &str = "callback_text";
const VKTEAMS_CHAT_ID: &str = "VKTEAMS_CHAT_ID";

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file
    dotenvy::dotenv().expect("unable to load .env file");
    // Initialize logger
    let _guard = otlp::init().map_err(|e| BotError::Otlp(e.into()))?;
    info!("Starting...");
    // Make bot
    let bot = Bot::default();
    // Get chat_id from .env file
    let chat_id = ChatId(
        std::env::var(VKTEAMS_CHAT_ID)
            .expect("Unable to find VKTEAMS_CHAT_ID in .env file")
            .to_string(),
    );
    // Add text
    let html_parser = MessageTextParser::new()
        .add(MessageTextFormat::Plain("Push the button".to_string()))
        .space();
    // Add button with callback data
    let keyboard = Keyboard::new()
        .add_button(&ButtonKeyboard::cb(
            CALLBACK_TEXT.to_string(),
            CALLBACK_DATA.to_string(), // Callback data
            ButtonStyle::Primary,
        ))
        .to_owned();
    // Send message
    info!("Sending message...");
    match bot
        .send_api_request(
            RequestMessagesSendText::new(chat_id)
                .set_keyboard(keyboard)
                .set_text(html_parser),
        )
        .await?
    {
        ApiResult::Success(_) => info!("Message sent"),
        ApiResult::Error(e) => {
            error!("Error: {}", e.description);
            return Err(BotError::Api(e));
        }
    };
    // Start event listener and pass result to a callback function
    bot.event_listener(callback).await?;
    Ok(())
}

// Callback function to print out the result
pub async fn callback(bot: Bot, res: ResponseEventsGet) -> Result<()> {
    // Answer callback query
    for event in res.events {
        // Check if event is a callback query and get payload
        let payload = match &event.event_type {
            EventType::CallbackQuery(payload) => payload.to_owned(),
            _ => continue,
        };
        match bot
            .send_api_request(
                RequestMessagesAnswerCallbackQuery::new(payload.query_id)
                    .with_text(
                        match payload.callback_data.as_str() {
                            CALLBACK_DATA => "Button pressed!",
                            _ => "WRONG button pressed!",
                        }
                        .to_string(),
                    )
                    .with_show_alert(true),
            )
            .await?
        {
            ApiResult::Success(_) => info!("Callback query answered"),
            ApiResult::Error(e) => {
                error!("Error: {}", e.description);
                return Err(BotError::Api(e));
            }
        }
    }
    Ok(())
}
