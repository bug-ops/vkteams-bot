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
    // Send message like text generation
    let bot = Bot::default();
    // Get chat_id from .env
    let chat_id = ChatId(
        std::env::var("VKTEAMS_CHAT_ID")
            .expect("Unable to find VKTEAMS_CHAT_ID in .env file")
            .to_string(),
    );
    // Bot action typing
    bot.send_api_request(RequestChatsSendAction::new((
        chat_id.to_owned(),
        ChatActions::Typing,
    )))
    .await
    .unwrap();
    // Send message
    match bot
        .send_api_request(RequestChatsGetInfo::new(chat_id.to_owned()))
        .await
    {
        Ok(res) => match res.res {
            EnumChatsGetInfo::Channel(chat) => {
                info!("Channel: {:?}", chat.title.unwrap());
            }
            EnumChatsGetInfo::Group(chat) => {
                info!("Group: {:?}", chat.title.unwrap());
            }
            EnumChatsGetInfo::Private(chat) => {
                info!(
                    "Private: {} {}",
                    chat.first_name.unwrap(),
                    chat.last_name.unwrap()
                );
            }
            EnumChatsGetInfo::None => {
                debug!("None");
            }
        },
        Err(e) => {
            error!("Error: {}", e);
        }
    }
}
