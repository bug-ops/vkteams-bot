#![allow(unused_parens)]
//! Get the events that have occurred since the last event id method `events/get`
//! [More info](https://teams.vk.com/botapi/#/events/get_events_get)
use crate::api::types::*;
bot_api_method! {
    method = "events/get",
    request = RequestEventsGet {
        required {
            last_event_id: EventId,
        },
        optional {
            poll_time: u64,
        }
    },
    response = ResponseEventsGet {
        events: Vec<EventMessage>,
    },
}

#[cfg(test)]
use crate::prelude::*;
#[test]
fn test_print_out() {
    let j = r#"
        {
          "events": [
            {
              "eventId": 1299,
              "payload": {
                "chat": {
                  "chatId": "123456@chat.agent",
                  "title": "TEST",
                  "type": "group"
                },
                "from": {
                  "firstName": "Test",
                  "lastName": "Tests",
                  "userId": "test@examle.com"
                },
                "msgId": "1111117815001117942",
                "parts": [
                  {
                    "payload": { "fileId": "zFDf9...1bg" },
                    "type": "file"
                  }
                ],
                "text": "https://files-n.internal.example.com/get/zFDf9...1bg",
                "timestamp": 1747470306
              },
              "type": "newMessage"
            }
          ],
          "ok": true
        }
        "#;
    match serde_json::from_str::<ResponseEventsGet>(j) {
        Ok(response) => println!("{:?}", response),
        Err(e) => {
            eprintln!("Error deserializing response: {}", e);
        }
    };
}
