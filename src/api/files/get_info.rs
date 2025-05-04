#![allow(unused_parens)]
//! File get info method `files/getInfo`
//! [More info](https://teams.vk.com/botapi/#/files/get_files_getInfo)
use crate::api::types::*;
use crate::bot::net::*;
use anyhow::{anyhow, Result};
use reqwest::Url;
use serde::{Deserialize, Serialize};

bot_api_method! {
    method = "files/getInfo",
    request = RequestFilesGetInfo {
        required {
            file_id: FileId,
        },
        optional {}
    },
    response = ResponseFilesGetInfo {
        #[serde(rename = "type", default)]
        file_type: String,
        #[serde(rename = "size", default)]
        file_size: u32,
        #[serde(rename = "filename", default)]
        file_name: String,
        #[serde(default)]
        url: String,
        ok: bool,
        #[serde(default)]
        description: String,
    },
}

impl ResponseFilesGetInfo {
    /// Download file data
    /// ## Parameters
    /// - `client`: [`reqwest::Client`] - reqwest client
    pub async fn download(&self, client: reqwest::Client) -> Result<Vec<u8>> {
        if !self.ok {
            return Err(anyhow!(self.description.to_owned()));
        }
        match Url::parse(&self.url.to_owned()) {
            Ok(url) => get_bytes_response(client, url).await,
            Err(e) => Err(e.into()),
        }
    }
}
