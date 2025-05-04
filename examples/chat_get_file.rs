#[macro_use]
extern crate log;
use anyhow::{Result, anyhow};
use vkteams_bot::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Check .env file and init logger
    dotenvy::dotenv().expect("Unable to load .env file");
    pretty_env_logger::init();
    info!("Starting...");
    // Make bot instance
    let bot = Bot::default();
    // Get events from the API
    match bot
        .send_api_request(RequestEventsGet::new(bot.get_last_event_id()))
        .await?
    {
        ApiResult::Success(res) => {
            let events = res.events;
            // Check if there are any events with new messages event type
            for event in events {
                if let EventType::NewMessage(payload) = event.event_type {
                    let parts = payload.parts;
                    // Check if there are any files in the message
                    for part in parts {
                        if let MessagePartsType::File(p) = part.part_type {
                            download_files(&bot, p.file_id).await?;
                        } else {
                            continue;
                        }
                    }
                } else {
                    continue;
                }
            }
        }
        ApiResult::Error { ok: _, description } => {
            error!("Error: {}", description);
            return Err(anyhow!("Error: {}", description));
        }
    }
    Ok(())
}
// Download files from messages
pub async fn download_files(bot: &Bot, file_id: FileId) -> Result<()> {
    // Get file info from the API
    match bot
        .send_api_request(RequestFilesGetInfo::new(file_id))
        .await?
    {
        // Download file data
        ApiResult::Success(file_info) => {
            let file_data = file_info.download(reqwest::Client::new()).await?;
            // Save file to the disk
            file_save(&file_info.file_name.to_owned(), file_data).await?;
        }
        ApiResult::Error { ok: _, description } => {
            error!("Error: {}", description);
            return Err(anyhow!("Error: {}", description));
        }
    }
    Ok(())
}
// Save file to the disk
pub async fn file_save(file_name: &str, file_data: Vec<u8>) -> Result<()> {
    let file_path = format!("tests/{}", file_name);
    tokio::fs::write(file_path.to_owned(), file_data).await?;
    Ok(())
}
