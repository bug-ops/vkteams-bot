use crate::api::types::*;
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
    pub file_type: String,
    #[serde(rename = "size")]
    pub file_size: u64,
    #[serde(rename = "filename")]
    pub file_name: String,
    pub url: String,
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
