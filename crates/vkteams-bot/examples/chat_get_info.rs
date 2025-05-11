#[macro_use]
extern crate log;
use vkteams_bot::error::{BotError, Result};
use vkteams_bot::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file
    dotenvy::dotenv().map_err(|e| BotError::Config(e.to_string()))?;
    // Initialize logger
    let _guard = otlp::init().map_err(|e| BotError::Otlp(e.into()))?;
    info!("Starting...");
    // Send message like text generation
    let bot = Bot::default();
    // Get chat_id from .env
    let chat_id = ChatId(
        std::env::var("VKTEAMS_CHAT_ID")
            .map_err(|e| BotError::Config(e.to_string()))?
            .to_string(),
    );
    // Bot action typing
    bot.send_api_request(RequestChatsSendAction::new((
        chat_id.to_owned(),
        ChatActions::Typing,
    )))
    .await?;
    // Send message
    let response = bot
        .send_api_request(RequestChatsGetInfo::new(chat_id.to_owned()))
        .await?;

    match response.into_result()?.res {
        EnumChatsGetInfo::Channel(chat) => {
            info!("Channel: {:?}", chat.title.unwrap());
        }
        EnumChatsGetInfo::Group(chat) => {
            info!("Group: {:?}", chat.title.unwrap());
        }
        EnumChatsGetInfo::Private(chat) => {
            info!(
                "Private: {} {}",
                chat.first_name.unwrap(),
                chat.last_name.unwrap()
            );
        }
        EnumChatsGetInfo::None => {
            debug!("None");
        }
    }
    Ok(())
}
