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
    let self_user_id = bot.self_get().await.unwrap().to_owned().user_id;
    // Get events with new chat members
    let events = iter_get_events(&bot).await.filter(|event| {
        event.event_type == EventType::NewChatMembers // Check event type
        && event.payload.chat.is_some()
        && event.payload.new_members.is_some()
        && event
            .payload
            .new_members
            .to_owned()
            .unwrap()
            .iter()
            .any(|member| member.user_id == self_user_id) // Check if the bot is a new chat member
    });
    // Check if the bot is an admin in the chat
    for event in events {
        let chat_id = event.payload.chat.unwrap().chat_id;
        // Get chat admins
        if iter_get_admins(&bot, chat_id.to_owned())
            .await
            // Check if the bot is an admin
            .any(|admin| admin.user_id == self_user_id)
        {
            // Set avatar for the chat
            avatar_set(&bot, chat_id.to_owned()).await;
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
// Get admins from the chat
pub async fn iter_get_admins(bot: &Bot, chat_id: ChatId) -> IntoIter<Admin> {
    bot.chats_get_admins(chat_id)
        .await
        .unwrap()
        .admins
        .unwrap()
        .to_owned()
        .into_iter()
}
// Set avatar for the chat
pub async fn avatar_set(bot: &Bot, chat_id: ChatId) {
    match bot
        // tests folder contains test.jpg file
        .chats_avatar_set(chat_id, String::from("tests/test.jpg"))
        .await
    {
        Ok(res) => info!("{:?}", res),
        Err(e) => error!("Error setting avatar: {:?}", e),
    }
}
