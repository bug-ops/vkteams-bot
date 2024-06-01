#[macro_use]
extern crate log;
use std::vec::IntoIter;
use vkteams_bot::{self, api::types::*};

#[tokio::main]
async fn main() {
    // Check .env file and init logger
    dotenvy::dotenv().expect("Unable to load .env file");
    pretty_env_logger::init();
    info!("Starting...");
    // Start bot with API version 1
    let bot = Bot::new(APIVersionUrl::V1);
    // Get events with files
    let events = iter_get_events(&bot).await.filter(|event| {
        event.event_type == EventType::NewMessage // Check event type
        && event.payload.message_parts.is_some() // Check if the message contains parts
        && event
            .payload
            .message_parts
            .to_owned()
            .unwrap()
            .iter()
            .any(|parts| parts.part_type == MessagePartsType::File) // Check if the message contains a file
    });

    for event in events {
        if let Some(parts) = event.payload.message_parts.unwrap().iter().find(|&parts| {
            parts.part_type == MessagePartsType::File && parts.payload.file_id.is_some()
        }) {
            {
                download_files(&bot, parts).await;
            }
        }
    }
}
// Download files from messages
pub async fn download_files(bot: &Bot, parts: &MessageParts) {
    // Get file id from the message
    let file_id = parts.payload.file_id.to_owned().unwrap();
    // Get file info from the API
    match bot.files_get_info(file_id).await {
        // Download file data
        Ok(file_info) => {
            if !file_info.ok {
                error!("Error: {:?}", file_info.description);
                return;
            }
            match file_info.download(reqwest::Client::new()).await {
                // Save file to the disk
                Ok(file_data) => file_save(&file_info.file_name, file_data).await,
                Err(e) => {
                    error!("Error: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }
}
// Get events from the API
pub async fn iter_get_events(bot: &Bot) -> IntoIter<EventMessage> {
    bot.events_get()
        .await
        .unwrap()
        .events
        .to_owned()
        .into_iter()
}
// Save file to the disk
pub async fn file_save(file_name: &str, file_data: Vec<u8>) {
    let file_path = format!("tests/{}", file_name);
    match tokio::fs::write(file_path.to_owned(), file_data).await {
        Ok(_) => {
            info!("File saved: {}", file_path);
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }
}
