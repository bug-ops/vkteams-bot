use std::sync::Arc;
use tracing::{error, info};
use vkteams_bot::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file
    dotenvy::dotenv().expect("unable to load .env file");
    // Initialize logger
    let _guard = otlp::init().map_err(|e| BotError::Otlp(e.into()))?;
    info!("Starting rate limit tests...");
    // Create bot with rate limit enabled
    let bot = Arc::new(Bot::default());
    // Get chat_id from .env
    let chat_id = Arc::new(ChatId(std::env::var("VKTEAMS_CHAT_ID")?.to_string()));

    info!("Test: Parallel message sending");
    let num_requests = 100;
    let mut handles = vec![];

    for i in 0..num_requests {
        let bot = Arc::clone(&bot);
        let chat_id = Arc::clone(&chat_id);

        let handle = tokio::spawn(async move {
            info!("Test : Sending request {}", i);

            if let Ok(request) = RequestMessagesSendText::new((*chat_id).clone()).set_text(
                MessageTextParser::new()
                    .add(MessageTextFormat::Plain(format!("Test: Message {}", i))),
            ) {
                match bot.send_api_request(request).await {
                    Ok(_) => {
                        info!("Test: Request {} completed successfully", i)
                    }
                    Err(e) => error!("Test: Error in request {}: {:?}", i, e),
                }
            } else {
                error!("Test: Error creating request {}", i)
            }
        });

        handles.push(handle);
    }
    // Wait for all requests to complete
    for handle in handles {
        handle.await.unwrap();
    }

    info!("All tests completed");
    Ok(())
}
