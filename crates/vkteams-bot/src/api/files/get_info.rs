#![allow(unused_parens)]
//! File get info method `files/getInfo`
//! [More info](https://teams.vk.com/botapi/#/files/get_files_getInfo)
use crate::api::types::*;
use crate::bot::net::*;
use crate::error::{BotError, Result};
use reqwest::Url;
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
    },
}

impl ResponseFilesGetInfo {
    /// Download file data
    /// ## Parameters
    /// - `client`: [`reqwest::Client`] - reqwest client
    pub async fn download(&self, client: reqwest::Client) -> Result<Vec<u8>> {
        if self.url.is_empty() {
            return Err(BotError::Validation("URL is empty".to_string()));
        }
        get_bytes_response(client, Url::parse(&self.url)?).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;
    use tokio;

    #[tokio::test]
    async fn test_download_empty_url() {
        let info = ResponseFilesGetInfo {
            file_type: "".to_string(),
            file_size: 0,
            file_name: "".to_string(),
            url: "".to_string(),
        };
        let client = Client::new();
        let result = info.download(client).await;
        assert!(matches!(result, Err(BotError::Validation(_))));
    }

    #[tokio::test]
    async fn test_download_invalid_url() {
        let info = ResponseFilesGetInfo {
            file_type: "".to_string(),
            file_size: 0,
            file_name: "".to_string(),
            url: "not a url".to_string(),
        };
        let client = Client::new();
        let result = info.download(client).await;
        assert!(matches!(result, Err(BotError::Url(_))));
    }

    #[tokio::test]
    async fn test_download_network_error() {
        let info = ResponseFilesGetInfo {
            file_type: "".to_string(),
            file_size: 0,
            file_name: "".to_string(),
            url: "http://localhost:0".to_string(),
        };
        let client = Client::new();
        let result = info.download(client).await;
        assert!(matches!(result, Err(BotError::Network(_))));
    }

    // Для успешного случая нужен mock, если есть возможность внедрить dependency injection или test server
    // #[test]
    // fn test_download_success() {
    //     let url = "http://localhost:8000/testfile.txt";
    //     let result = download_file(url);
    //     assert!(result.is_ok());
    // }
}
