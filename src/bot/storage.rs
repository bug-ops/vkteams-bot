//! Storage module
//! using Tarantool SDK for Rust
use std::fmt::Debug;
use std::sync::Arc;

use crate::prelude::*;
use anyhow::Result;
use tarantool::{
    net_box::{self, ConnOptions, Options},
    tuple::Tuple,
};

const VKTEAMS_TNT_URL: &str = "VKTEAMS_TNT_URL";
const VKTEAMS_TNT_USER: &str = "VKTEAMS_TNT_USER";
const VKTEAMS_TNT_PASS: &str = "VKTEAMS_TNT_PASS";

pub struct Tnt {
    pub(crate) conn: net_box::Conn,
}

impl Debug for Tnt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tnt").finish()
    }
}

impl Clone for Tnt {
    fn clone(&self) -> Self {
        self::Tnt::new()
    }
}

unsafe impl Send for Tnt {}

unsafe impl Sync for Tnt {}

impl Tnt {
    pub fn new() -> Self {
        let addr = std::env::var(VKTEAMS_TNT_URL).unwrap_or_else(|_| "[::1]:3301".to_string());
        let opts = ConnOptions {
            user: std::env::var(VKTEAMS_TNT_USER).unwrap(),
            password: std::env::var(VKTEAMS_TNT_PASS).unwrap(),
            ..Default::default()
        };
        let conn = match net_box::Conn::new(addr, opts, None) {
            Ok(conn) => conn,
            Err(e) => panic!("Unable to connect to Tarantool: {}", e),
        };
        Self { conn }
    }
}

impl Bot {
    pub fn store_events(&self, events: Vec<EventMessage>) -> Result<()> {
        let tnt = Arc::clone(&self.conn);
        let opt = Options {
            timeout: Some(std::time::Duration::from_secs(1)),
            ..Default::default()
        };
        for event in events {
            let msg = serde_json::to_string(&event).unwrap();
            let chat_id = match event.event_type {
                EventType::CallbackQuery(evt) => evt.chat.chat_id,
                EventType::DeleteMessage(evt) => evt.chat.chat_id,
                EventType::EditedMessage(evt) => evt.chat.chat_id,
                EventType::LeftChatMembers(evt) => evt.chat.chat_id,
                EventType::NewChatMembers(evt) => evt.chat.chat_id,
                EventType::NewMessage(evt) => evt.chat.chat_id,
                EventType::PinnedMessage(evt) => evt.chat.chat_id,
                EventType::UnpinnedMessage(evt) => evt.chat.chat_id,
                _ => ChatId(String::new()),
            };
            let tuple = Tuple::new(&(chat_id, event.event_id, msg))?;
            tnt.conn.call("event_save", &tuple, &opt)?;
        }
        Ok(())
    }
}

// #[proc]
// fn message_save<T>(msg: T, id: MsgId)
// where
//     T: BotRequest + Serialize + Encode + ToTupleBuffer,
// {
//     let space = Space::find("messages").unwrap();
//     // let tuple = Tuple::from(msg);
//     let result = space.insert(&msg);
//     println!("{:?}", result);
// }
// #[proc]
// fn message_update<T>(msg: T, id: MsgId)
// where
//     T: BotRequest + Serialize,
// {
// }

// #[proc]
// fn message_get<T>(id: MsgId) -> Result<T>
// where
//     T: BotRequest + Serialize,
// {
//     Ok(RequestMessagesSendText {
//         ..Default::default()
//     })
// }

// #[proc]
// fn message_delete(id: MsgId) -> Result<()> {}

// #[proc]
// fn event_save() {}

// #[proc]
// fn event_get(id: EventId) {}

// #[proc]
// fn event_delete(id: EventId) {}

// #[proc]
// fn file_save() {}

// #[proc]
// fn file_get(id: FileId) {}

// #[proc]
// fn file_delete(id: FileId) {}
