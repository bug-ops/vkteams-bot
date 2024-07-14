#[macro_use]
extern crate log;

use vkteams_bot::prelude::*;

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().expect("unable to load .env file");
    // Initialize logger
    pretty_env_logger::init();
    info!("Starting...");
    // Make bot
    let bot = Bot::default();
    // Start event listener and pass result to a callback function
    bot.event_listener(print_out).await;
}
// Callback function to print out the result
pub async fn print_out(_: Bot, res: ResponseEventsGet) {
    // If the result is Ok, convert it to a string and print it out
    match serde_json::to_string(&res) {
        Ok(s) => println!("{}", s),
        // If the result is an error, print error message
        Err(e) => println!("{}", e),
    }
}
