#[macro_use]
extern crate log;

use vkteams_bot::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file
    dotenvy::dotenv().expect("unable to load .env file");
    // Initialize logger
    let _guard = otlp::init().map_err(|e| BotError::Otlp(e.into()))?;
    info!("Starting...");
    // Make bot
    let bot = Bot::default();
    // Start event listener and pass result to a callback function
    bot.event_listener(print_out).await
}
// Callback function to print out the result
pub async fn print_out(_: Bot, res: ResponseEventsGet) -> Result<()> {
    // If the result is Ok, convert it to a string and print it out
    println!("{}", serde_json::to_string(&res)?);
    Ok(())
}
