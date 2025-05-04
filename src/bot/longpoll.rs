use crate::error::Result;
use crate::prelude::*;
use std::future::Future;
/// Listen for events and execute callback function
/// ## Parameters
/// - `func` - callback function with [`Result`] type [`ResponseEventsGet`] as argument
impl Bot {
    /// Слушает события и выполняет callback функцию
    /// ## Параметры
    /// - `func` - callback функция с типом [`Result`] и аргументом [`ResponseEventsGet`]
    ///
    /// ## Ошибки
    /// - `BotError::Api` - ошибка API при получении событий
    /// - `BotError::Network` - ошибка сети при получении событий
    /// - `BotError::Serialization` - ошибка десериализации ответа
    /// - `BotError::System` - ошибка при выполнении callback функции
    pub async fn event_listener<F, X>(&self, func: F) -> Result<()>
    where
        F: Fn(Bot, ResponseEventsGet) -> X,
        X: Future<Output = Result<()>> + Send + Sync + 'static,
    {
        loop {
            debug!("Получение событий с ID: {}", self.get_last_event_id());

            // Make a request to the API
            let req = RequestEventsGet::new(self.get_last_event_id()).with_poll_time(POLL_TIME);

            // Get response
            let res = self.send_api_request::<RequestEventsGet>(req).await?;

            match res.into_result()? {
                events if !events.events.is_empty() => {
                    debug!("Получено {} событий", events.events.len());

                    // Update last event id
                    let last_event_id = events.events[events.events.len() - 1].event_id;
                    self.set_last_event_id(last_event_id);
                    debug!("Обновлен ID последнего события: {}", last_event_id);

                    // Execute callback function
                    if let Err(e) = func(self.clone(), events).await {
                        error!("Ошибка при обработке событий: {}", e);
                        return Err(e);
                    }
                }
                _ => {
                    debug!("Событий не получено, продолжаем ожидание");
                    continue;
                }
            }
        }
    }
}
