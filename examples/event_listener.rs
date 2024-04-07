#[macro_use]
extern crate log;

use anyhow::Result;
use vkteams_bot::{self, api::types::*};

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
pub fn print_out(res: &Result<ResponseEventsGet>) {
    match res {
        // If the result is Ok, convert it to a string and print it out
        Ok(r) => match serde_json::to_string(r) {
            Ok(s) => println!("{}", s),
            // If the result is an error, print error message
            Err(e) => println!("{}", e),
        },
        // If the result is an error, print error message
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
