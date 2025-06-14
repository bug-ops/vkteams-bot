use tracing::info;
use vkteams_bot::error::{BotError, Result};
use vkteams_bot::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file
    dotenvy::dotenv().map_err(|e| BotError::Config(e.to_string()))?;
    // Initialize logger
    let _guard = otlp::init().map_err(|e| BotError::Otlp(e.into()))?;
    info!("Starting...");
    // Create bot instance
    let bot = Bot::default();
    // Get information about myself
    let me = bot.send_api_request(RequestSelfGet::default()).await?;
    info!("User info:");
    info!("  user_id: {}", me.user_id.0);
    info!("  nick: {}", me.nick);
    info!("  first_name: {}", me.first_name);
    if let Some(about) = me.about {
        info!("  about: {}", about);
    } else {
        info!("  about: <none>");
    }
    if let Some(photo) = me.photo {
        for (i, photo) in photo.iter().enumerate() {
            info!("  photo[{}]: {}", i, photo.url);
        }
    } else {
        info!("  photo: <none>");
    }
    Ok(())
}
