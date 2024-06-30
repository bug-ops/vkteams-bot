#[macro_use]
extern crate log;
use vkteams_bot::{self, api::types::*};

#[tokio::main]
async fn main() {
    // Check .env file and init logger
    dotenvy::dotenv().expect("Unable to load .env file");
    pretty_env_logger::init();
    info!("Starting...");
    // Make bot instance
    let bot = Bot::default();
    // Get events from the API
    match bot
        .send_api_request(RequestEventsGet::new(bot.get_last_event_id()))
        .await
    {
        Ok(res) => {
            let events = res.events;
            // Check if there are any events with new messages event type
            for event in events {
                if let EventType::NewMessage(payload) = event.event_type {
                    let parts = payload.parts;
                    // Check if there are any files in the message
                    for part in parts {
                        if let MessagePartsType::File(p) = part.part_type {
                            download_files(&bot, p.file_id).await;
                        } else {
                            continue;
                        }
                    }
                } else {
                    continue;
                }
            }
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }
}
// Download files from messages
pub async fn download_files(bot: &Bot, file_id: FileId) {
    // Get file info from the API
    match bot
        .send_api_request(RequestFilesGetInfo::new(file_id))
        .await
    {
        // Download file data
        Ok(file_info) => {
            if !file_info.ok {
                error!("Error: {:?}", file_info.description);
                return;
            }
            match file_info.download(reqwest::Client::new()).await {
                // Save file to the disk
                Ok(file_data) => file_save(&file_info.file_name.to_owned(), file_data).await,
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
