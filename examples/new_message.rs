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
    const CODE_STRING: &str = "<!DOCTYPE html>\n<html>\n<head>\n<title>Page Title</title>\n</head>\n<body>\n</body>\n</html>";
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
        .send_api_request(
            RequestMessagesSendText::new(chat_id.to_owned())
                .set_text(
                    MessageTextParser::new()
                        .add(MessageTextFormat::Plain("Code below:".to_string()))
                        .next_line()
                        .add(MessageTextFormat::Pre(
                            CODE_STRING.to_string(),
                            Some("html".to_string()),
                        )),
                )
                .set_keyboard(
                    Keyboard::new()
                        .add_button(&ButtonKeyboard::url(
                            "Button url".to_string(),
                            "https://example.com".to_string(),
                            ButtonStyle::Primary,
                        ))
                        .add_row()
                        .add_button(&ButtonKeyboard::cb(
                            "Callback".to_string(),
                            "CB".to_string(),
                            ButtonStyle::Attention,
                        )),
                ),
        )
        .await
    {
        Ok(res) => {
            if res.ok {
                debug!("Message id: {:?}", res.msg_id);
                bot.send_api_request(RequestChatsSendAction::new((
                    chat_id.to_owned(),
                    ChatActions::Looking,
                )))
                .await
                .unwrap();
            } else {
                error!("Error: {}", res.description);
            }
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }
}
