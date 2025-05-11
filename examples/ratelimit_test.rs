#[macro_use]
extern crate log;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use vkteams_bot::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Загружаем .env файл
    dotenvy::dotenv().expect("unable to load .env file");
    // Инициализируем логгер
    let _guard = otlp::init().map_err(|e| BotError::Otlp(e.into()))?;
    info!("Starting rate limit tests...");

    // Создаем бота с включенным rate limit
    let bot = Arc::new(Bot::default());

    // Получаем chat_id из .env
    let chat_id = Arc::new(ChatId(
        std::env::var("VKTEAMS_CHAT_ID")
            .expect("Unable to find VKTEAMS_CHAT_ID in .env file")
            .to_string(),
    ));

    // Тест 1: Отправка сообщений с небольшой задержкой
    info!("Тест 1: Отправка сообщений с небольшой задержкой");
    for i in 0..5 {
        let bot = Arc::clone(&bot);
        let chat_id = Arc::clone(&chat_id);

        let request = RequestMessagesSendText::new((*chat_id).clone()).set_text(
            MessageTextParser::new()
                .add(MessageTextFormat::Plain(format!("Тест 1: Сообщение {}", i))),
        );

        match request {
            Ok(req) => {
                // Проверяем, что chat_id правильно извлекается из запроса
                if let Some(id) = req.get_chat_id() {
                    info!("Тест 1: ChatId из запроса: {:?}", id);
                } else {
                    error!("Тест 1: Не удалось получить ChatId из запроса");
                }

                match bot.send_api_request(req).await {
                    Ok(ApiResult::Success(_)) => info!("Тест 1: Запрос {} успешно выполнен", i),
                    Ok(ApiResult::Error(e)) => error!("Тест 1: Ошибка в запросе {}: {:?}", i, e),
                    Err(e) => error!("Тест 1: Ошибка в запросе {}: {:?}", i, e),
                }
            }
            Err(e) => error!("Тест 1: Ошибка создания запроса {}: {:?}", i, e),
        }

        sleep(Duration::from_millis(100)).await;
    }

    // Тест 2: Параллельная отправка сообщений
    info!("Тест 2: Параллельная отправка сообщений");
    let num_requests = 10;
    let semaphore = Arc::new(Semaphore::new(num_requests));
    let mut handles = vec![];

    for i in 0..num_requests {
        let bot = Arc::clone(&bot);
        let chat_id = Arc::clone(&chat_id);
        let semaphore = Arc::clone(&semaphore);

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            info!("Тест 2: Отправка запроса {}", i);

            let request = RequestMessagesSendText::new((*chat_id).clone()).set_text(
                MessageTextParser::new()
                    .add(MessageTextFormat::Plain(format!("Тест 2: Сообщение {}", i))),
            );

            match request {
                Ok(req) => {
                    // Проверяем, что chat_id правильно извлекается из запроса
                    if let Some(id) = req.get_chat_id() {
                        info!("Тест 2: ChatId из запроса: {:?}", id);
                    } else {
                        error!("Тест 2: Не удалось получить ChatId из запроса");
                    }

                    match bot.send_api_request(req).await {
                        Ok(ApiResult::Success(_)) => info!("Тест 2: Запрос {} успешно выполнен", i),
                        Ok(ApiResult::Error(e)) => {
                            error!("Тест 2: Ошибка в запросе {}: {:?}", i, e)
                        }
                        Err(e) => error!("Тест 2: Ошибка в запросе {}: {:?}", i, e),
                    }
                }
                Err(e) => error!("Тест 2: Ошибка создания запроса {}: {:?}", i, e),
            }
        });

        handles.push(handle);
    }

    // Ждем завершения всех запросов
    for handle in handles {
        handle.await.unwrap();
    }

    // Тест 3: Отправка сообщений после превышения лимита
    info!("Тест 3: Отправка сообщений после превышения лимита");
    let num_requests = 20;
    let mut handles = vec![];

    for i in 0..num_requests {
        let bot = Arc::clone(&bot);
        let chat_id = Arc::clone(&chat_id);

        let handle = tokio::spawn(async move {
            info!("Тест 3: Отправка запроса {}", i);

            let request = RequestMessagesSendText::new((*chat_id).clone()).set_text(
                MessageTextParser::new()
                    .add(MessageTextFormat::Plain(format!("Тест 3: Сообщение {}", i))),
            );

            match request {
                Ok(req) => {
                    // Проверяем, что chat_id правильно извлекается из запроса
                    if let Some(id) = req.get_chat_id() {
                        debug!("Тест 3: ChatId из запроса: {:?}", id);
                    } else {
                        error!("Тест 3: Не удалось получить ChatId из запроса");
                    }

                    match bot.send_api_request(req).await {
                        Ok(ApiResult::Success(_)) => info!("Тест 3: Запрос {} успешно выполнен", i),
                        Ok(ApiResult::Error(e)) => {
                            error!("Тест 3: Ошибка в запросе {}: {:?}", i, e)
                        }
                        Err(e) => error!("Тест 3: Ошибка отправки запроса {}: {:?}", i, e),
                    }
                }
                Err(e) => error!("Тест 3: Ошибка создания запроса {}: {:?}", i, e),
            }
        });

        handles.push(handle);
    }

    // Ждем завершения всех запросов
    for handle in handles {
        handle.await.unwrap();
    }

    // Тест 4: Проверка восстановления после превышения лимита
    info!("Тест 4: Проверка восстановления после превышения лимита");
    sleep(Duration::from_secs(2)).await; // Ждем полного восстановления лимитов

    for i in 0..3 {
        let bot = Arc::clone(&bot);
        let chat_id = Arc::clone(&chat_id);

        let request = RequestMessagesSendText::new((*chat_id).clone()).set_text(
            MessageTextParser::new().add(MessageTextFormat::Plain(format!(
                "Тест 4: Сообщение после восстановления {}",
                i
            ))),
        )?;

        // Проверяем, что chat_id правильно извлекается из запроса
        if let Some(id) = request.get_chat_id() {
            info!("Тест 4: ChatId из запроса: {:?}", id);
        } else {
            error!("Тест 4: Не удалось получить ChatId из запроса");
        }

        let response = bot.send_api_request(request).await;

        match response {
            Ok(ApiResult::Success(_)) => {
                info!("Тест 4: Запрос {} успешно выполнен после восстановления", i)
            }
            Ok(ApiResult::Error(e)) => error!(
                "Тест 4: Ошибка в запросе {} после восстановления: {:?}",
                i, e
            ),
            Err(e) => error!(
                "Тест 4: Ошибка в запросе {} после восстановления: {:?}",
                i, e
            ),
        }

        sleep(Duration::from_millis(100)).await;
    }

    info!("Все тесты завершены");
    Ok(())
}
