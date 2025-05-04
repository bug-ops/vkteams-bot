#[macro_use]
extern crate log;
use anyhow::{Result, anyhow};
use std::vec::IntoIter;
use vkteams_bot::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Check .env file and init logger
    dotenvy::dotenv().expect("Unable to load .env file");
    pretty_env_logger::init();
    info!("Starting...");
    // Start bot with API version 1
    let bot = Bot::new(APIVersionUrl::V1);
    // Remember self user_id
    let self_user_id = match bot.send_api_request(RequestSelfGet::new(())).await? {
        ApiResult::Success(res) => res.user_id,
        ApiResult::Error { ok: _, description } => {
            error!("Error: {:?}", description);
            return Err(anyhow!("Error: {:?}", description));
        }
    };

    // Get events with new chat members
    let events = iter_get_events(&bot)
        .await?
        .filter(|e| match &e.event_type {
            EventType::NewChatMembers(payload) => payload
                .new_members
                .iter()
                .any(|member| member.user_id == self_user_id),
            _ => false,
        });
    // Check if the bot is an admin in the chat
    for event in events {
        let payload = match &event.event_type {
            EventType::NewChatMembers(payload) => payload.as_ref(),
            _ => continue,
        };
        let chat = payload.chat.clone();
        // Get info about the chat
        match bot
            .send_api_request(RequestChatsGetInfo::new(chat.chat_id.clone()))
            .await?
        {
            ApiResult::Success(res) => match res.res {
                EnumChatsGetInfo::Group(chat_info) => {
                    info!("Chat info: {:?}", chat_info);
                }
                _ => continue,
            },
            _ => continue,
        }
        // Get chat admins
        let is_admin = iter_get_admins(&bot, chat.chat_id.clone())
            .await?
            .any(|admin| admin.user_id == self_user_id);
        if is_admin {
            // Set avatar for the chat
            avatar_set(&bot, chat.chat_id.clone()).await;
        }
    }
    Ok(())
}
// Get events from the API
pub async fn iter_get_events(bot: &Bot) -> Result<IntoIter<EventMessage>> {
    match bot
        .send_api_request(RequestEventsGet::new(bot.get_last_event_id()))
        .await?
    {
        ApiResult::Success(res) => Ok(res.events.into_iter()),
        ApiResult::Error { ok: _, description } => {
            error!("Error: {:?}", description);
            return Err(anyhow!("Error: {:?}", description));
        }
    }
}
// Get admins from the chat
pub async fn iter_get_admins(bot: &Bot, chat_id: ChatId) -> Result<IntoIter<Admin>> {
    match bot
        .send_api_request(RequestChatsGetAdmins::new(chat_id))
        .await?
    {
        ApiResult::Success(res) => Ok(res.admins.into_iter()),
        ApiResult::Error { ok: _, description } => {
            error!("Error: {:?}", description);
            return Err(anyhow!("Error: {:?}", description));
        }
    }
}
// Set avatar for the chat
pub async fn avatar_set(bot: &Bot, chat_id: ChatId) {
    match bot
        // tests folder contains test.jpg file
        .send_api_request(RequestChatsAvatarSet::new((
            chat_id,
            MultipartName::Image(String::from("tests/test.jpg")),
        )))
        .await
    {
        Ok(res) => info!("{:?}", res),
        Err(e) => error!("Error setting avatar: {:?}", e),
    }
}
