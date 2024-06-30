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
    // Remember self user_id
    let self_user_id = bot
        .send_api_request(RequestSelfGet::new())
        .await
        .unwrap()
        .to_owned()
        .user_id;
    // Get events with new chat members
    let events = iter_get_events(&bot).await.filter(|e| match &e.event_type {
        EventType::NewChatMembers(payload) => payload
            .new_members
            .iter()
            .any(|member| member.user_id == self_user_id),
        _ => false,
    });
    // Check if the bot is an admin in the chat
    for event in events {
        let payload = match &event.event_type {
            EventType::NewChatMembers(payload) => payload,
            _ => continue,
        };
        let chat = payload.chat.to_owned();
        // Get info about the chat
        match bot
            .send_api_request(RequestChatsGetInfo::new(chat.chat_id.to_owned()))
            .await
            .unwrap()
        {
            ResponseChatsGetInfo::Group(chat_info) => {
                info!("Chat info: {:?}", chat_info);
            }
            _ => continue,
        }
        // Get chat admins
        if iter_get_admins(&bot, chat.chat_id.to_owned())
            .await
            // Check if the bot is an admin
            .any(|admin| admin.user_id == self_user_id)
        {
            // Set avatar for the chat
            avatar_set(&bot, chat.chat_id.to_owned()).await;
        }
    }
}
// Get events from the API
pub async fn iter_get_events(bot: &Bot) -> IntoIter<EventMessage> {
    bot.send_api_request(RequestEventsGet::new(bot.get_last_event_id()))
        .await
        .unwrap()
        .events
        .to_owned()
        .into_iter()
}
// Get admins from the chat
pub async fn iter_get_admins(bot: &Bot, chat_id: ChatId) -> IntoIter<Admin> {
    bot.send_api_request(RequestChatsGetAdmins::new(chat_id))
        .await
        .unwrap()
        .admins
        .to_owned()
        .into_iter()
}
// Set avatar for the chat
pub async fn avatar_set(bot: &Bot, chat_id: ChatId) {
    match bot
        // tests folder contains test.jpg file
        .send_api_request(RequestChatsAvatarSet::new(
            chat_id,
            MultipartName::Image(String::from("tests/test.jpg")),
        ))
        .await
    {
        Ok(res) => info!("{:?}", res),
        Err(e) => error!("Error setting avatar: {:?}", e),
    }
}
