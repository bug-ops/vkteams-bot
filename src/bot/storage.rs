use crate::{api::types::*, prelude::RequestMessagesSendText};
use anyhow::Result;
use serde::Serialize;
use tarantool::{
    proc,
    space::Space,
    tuple::{Encode, ToTupleBuffer, Tuple},
};

#[proc]
fn message_save<T>(msg: T, id: MsgId)
where
    T: BotRequest + Serialize + Encode + ToTupleBuffer,
{
    let space = Space::find("messages").unwrap();
    // let tuple = Tuple::from(msg);
    let result = space.insert(&msg);
    println!("{:?}", result);
}
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
