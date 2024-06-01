use crate::api::types::*;
use anyhow::{anyhow, Result};
use reqwest::Url;
use serde::{Deserialize, Serialize};
/// Request for method [`SendMessagesAPIMethods::FilesGetInfo`]
///
/// [`SendMessagesAPIMethods::FilesGetInfo`]: enum.SendMessagesAPIMethods.html#variant.FilesGetInfo
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestFilesGetInfo {
    pub file_id: FileId,
}
/// Response for method [`SendMessagesAPIMethods::FilesGetInfo`]
///
/// [`SendMessagesAPIMethods::FilesGetInfo`]: enum.SendMessagesAPIMethods.html#variant.FilesGetInfo
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseFilesGetInfo {
    #[serde(rename = "type")]
    pub file_type: Option<String>,
    #[serde(rename = "size")]
    pub file_size: Option<u64>,
    #[serde(rename = "filename")]
    pub file_name: Option<String>,
    pub url: Option<String>,
    pub ok: bool,
    pub description: Option<String>,
}
impl BotRequest for RequestFilesGetInfo {
    const METHOD: &'static str = "files/getInfo";
    type RequestType = Self;
    type ResponseType = ResponseFilesGetInfo;
    fn new(method: &Methods) -> Self {
        match method {
            Methods::FilesGetInfo(file_id) => Self {
                file_id: file_id.to_owned(),
            },
            _ => panic!("Wrong API method for RequestFilesGetInfo"),
        }
    }
}
impl ResponseFilesGetInfo {
    /// Download file data
    /// - `client` - reqwest client
    pub async fn download(&self, client: reqwest::Client) -> Result<Vec<u8>> {
        if !self.ok {
            return Err(anyhow!(self.description.to_owned().unwrap_or_default()));
        }
        match Url::parse(&self.url.to_owned().unwrap()) {
            Ok(url) => get_bytes_response(client, url).await,
            Err(e) => Err(e.into()),
        }
    }
}
