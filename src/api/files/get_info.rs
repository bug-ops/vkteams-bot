//! File get info method `files/getInfo`
//! [More info](https://teams.vk.com/botapi/#/files/get_files_getInfo)
use crate::api::types::*;
use crate::bot::net::*;
use anyhow::{anyhow, Result};
use reqwest::Url;
use serde::{Deserialize, Serialize};
/// File get info method `files/getInfo`
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestFilesGetInfo {
    pub file_id: FileId,
}
/// File get info method `files/getInfo`
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseFilesGetInfo {
    #[serde(rename = "type", default)]
    pub file_type: String,
    #[serde(rename = "size", default)]
    pub file_size: u32,
    #[serde(rename = "filename", default)]
    pub file_name: String,
    #[serde(default)]
    pub url: String,
    pub ok: bool,
    #[serde(default)]
    pub description: String,
}
impl BotRequest for RequestFilesGetInfo {
    const METHOD: &'static str = "files/getInfo";
    type RequestType = Self;
    type ResponseType = ResponseFilesGetInfo;
}
impl RequestFilesGetInfo {
    /// Create a new RequestFilesGetInfo
    /// ## Parameters
    /// - `file_id` - [`FileId`]
    pub fn new(file_id: FileId) -> Self {
        Self { file_id }
    }
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
