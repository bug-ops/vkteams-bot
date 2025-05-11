use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
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
    let chat_id = Arc::new(ChatId(
        std::env::var("VKTEAMS_CHAT_ID")
            .expect("Unable to find VKTEAMS_CHAT_ID in .env file")
            .to_string(),
    ));

    // Test 1: Sending messages with small delay
    info!("Test 1: Sending messages with small delay");
    for i in 0..5 {
        let bot = Arc::clone(&bot);
        let chat_id = Arc::clone(&chat_id);

        if let Ok(request) = RequestMessagesSendText::new((*chat_id).clone()).set_text(
            MessageTextParser::new()
                .add(MessageTextFormat::Plain(format!("Test 1: Message {}", i))),
        ) {
            match bot.send_api_request(request).await {
                Ok(ApiResult::Success(_)) => {
                    info!("Test 1: Request {} completed successfully", i)
                }
                Ok(ApiResult::Error(e)) => error!("Test 1: Error in request {}: {:?}", i, e),
                Err(e) => error!("Test 1: Error in request {}: {:?}", i, e),
            }
        } else {
            error!("Test 1: Error creating request {}", i)
        }
        sleep(Duration::from_millis(100)).await;
    }
    // Test 2: Parallel message sending
    info!("Test 2: Parallel message sending");
    let num_requests = 10;
    let mut handles = vec![];

    for i in 0..num_requests {
        let bot = Arc::clone(&bot);
        let chat_id = Arc::clone(&chat_id);

        let handle = tokio::spawn(async move {
            info!("Test 2: Sending request {}", i);

            if let Ok(request) = RequestMessagesSendText::new((*chat_id).clone()).set_text(
                MessageTextParser::new()
                    .add(MessageTextFormat::Plain(format!("Test 2: Message {}", i))),
            ) {
                match bot.send_api_request(request).await {
                    Ok(ApiResult::Success(_)) => {
                        info!("Test 2: Request {} completed successfully", i)
                    }
                    Ok(ApiResult::Error(e)) => {
                        error!("Test 2: Error in request {}: {:?}", i, e)
                    }
                    Err(e) => error!("Test 2: Error in request {}: {:?}", i, e),
                }
            } else {
                error!("Test 2: Error creating request {}", i)
            }
        });

        handles.push(handle);
    }
    // Wait for all requests to complete
    for handle in handles {
        handle.await.unwrap();
    }
    // Test 3: Sending messages after exceeding the limit
    info!("Test 3: Sending messages after exceeding the limit");
    let num_requests = 20;
    let mut handles = vec![];

    for i in 0..num_requests {
        let bot = Arc::clone(&bot);
        let chat_id = Arc::clone(&chat_id);

        let handle = tokio::spawn(async move {
            info!("Test 3: Sending request {}", i);

            if let Ok(request) = RequestMessagesSendText::new((*chat_id).clone()).set_text(
                MessageTextParser::new()
                    .add(MessageTextFormat::Plain(format!("Test 3: Message {}", i))),
            ) {
                match bot.send_api_request(request).await {
                    Ok(ApiResult::Success(_)) => {
                        info!("Test 3: Request {} completed successfully", i)
                    }
                    Ok(ApiResult::Error(e)) => {
                        error!("Test 3: Error in request {}: {:?}", i, e)
                    }
                    Err(e) => error!("Test 3: Error sending request {}: {:?}", i, e),
                }
            } else {
                error!("Test 3: Error creating request {}", i)
            }
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Test 4: Checking recovery after exceeding the limit
    info!("Test 4: Checking recovery after exceeding the limit");
    sleep(Duration::from_secs(2)).await; // Wait for complete limit recovery

    for i in 0..3 {
        let bot = Arc::clone(&bot);
        let chat_id = Arc::clone(&chat_id);

        if let Ok(request) =
            RequestMessagesSendText::new((*chat_id).clone()).set_text(MessageTextParser::new().add(
                MessageTextFormat::Plain(format!("Test 4: Message after recovery {}", i)),
            ))
        {
            match bot.send_api_request(request).await {
                Ok(ApiResult::Success(_)) => {
                    info!(
                        "Test 4: Request {} completed successfully after recovery",
                        i
                    )
                }
                Ok(ApiResult::Error(e)) => {
                    error!("Test 4: Error in request {} after recovery: {:?}", i, e)
                }
                Err(e) => error!("Test 4: Error in request {} after recovery: {:?}", i, e),
            }
        } else {
            error!("Test 4: Error creating request {}", i)
        }

        sleep(Duration::from_millis(100)).await;
    }

    info!("All tests completed");
    Ok(())
}
