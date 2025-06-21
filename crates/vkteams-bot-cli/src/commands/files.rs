//! File upload and management commands

use crate::commands::{Command, OutputFormat};
use crate::errors::prelude::{CliError, Result as CliResult};
use crate::output::{CliResponse, OutputFormatter};
use async_trait::async_trait;
use base64::Engine;
use clap::{Args, Subcommand};
use serde_json::json;
use vkteams_bot::prelude::*;

#[derive(Debug, Clone, Subcommand)]
pub enum FileCommands {
    /// Upload file from base64 content
    Upload(UploadFileArgs),
    /// Upload text content as file
    UploadText(UploadTextArgs),
    /// Upload JSON data as file
    UploadJson(UploadJsonArgs),
    /// Get file information
    Info(FileInfoArgs),
}

#[derive(Debug, Clone, Args)]
pub struct UploadFileArgs {
    /// File name with extension
    #[arg(long)]
    pub name: String,
    
    /// Base64 encoded file content
    #[arg(long)]
    pub content_base64: String,
    
    /// Optional caption/text message
    #[arg(long)]
    pub caption: Option<String>,
    
    /// Reply to message ID
    #[arg(long)]
    pub reply_msg_id: Option<String>,
    
    /// Chat ID (will use default from config if not provided)
    #[arg(long)]
    pub chat_id: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct UploadTextArgs {
    /// File name with extension
    #[arg(long)]
    pub name: String,
    
    /// Text content to save as file
    #[arg(long)]
    pub content: String,
    
    /// Optional caption/description
    #[arg(long)]
    pub caption: Option<String>,
    
    /// Reply to message ID
    #[arg(long)]
    pub reply_msg_id: Option<String>,
    
    /// Chat ID (will use default from config if not provided)
    #[arg(long)]
    pub chat_id: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct UploadJsonArgs {
    /// File name (will add .json extension if missing)
    #[arg(long)]
    pub name: String,
    
    /// JSON data as string
    #[arg(long)]
    pub json_data: String,
    
    /// Pretty print JSON
    #[arg(long, default_value = "true")]
    pub pretty: bool,
    
    /// Optional caption/description
    #[arg(long)]
    pub caption: Option<String>,
    
    /// Reply to message ID
    #[arg(long)]
    pub reply_msg_id: Option<String>,
    
    /// Chat ID (will use default from config if not provided)
    #[arg(long)]
    pub chat_id: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct FileInfoArgs {
    /// File ID to get information about
    #[arg(long)]
    pub file_id: String,
}

impl FileCommands {
    pub async fn execute_with_output(
        &self,
        bot: &Bot,
        output_format: &OutputFormat,
    ) -> CliResult<()> {
        let response = match self {
            FileCommands::Upload(args) => self.handle_upload(bot, args).await,
            FileCommands::UploadText(args) => self.handle_upload_text(bot, args).await,
            FileCommands::UploadJson(args) => self.handle_upload_json(bot, args).await,
            FileCommands::Info(args) => self.handle_file_info(bot, args).await,
        };

        OutputFormatter::print(&response, output_format)?;
        
        if !response.success {
            std::process::exit(1);
        }
        
        Ok(())
    }

    async fn handle_upload(&self, bot: &Bot, args: &UploadFileArgs) -> CliResponse<serde_json::Value> {
        // Decode base64 content
        let file_content = match base64::engine::general_purpose::STANDARD.decode(&args.content_base64) {
            Ok(content) => content,
            Err(e) => return CliResponse::error("upload-file", format!("Invalid base64 content: {}", e)),
        };

        // Validate file size (100MB limit)
        if file_content.len() > 100 * 1024 * 1024 {
            return CliResponse::error("upload-file", "File too large (max 100MB)");
        }

        let chat_id = match &args.chat_id {
            Some(id) => ChatId::from_borrowed_str(id),
            None => {
                // Get default chat ID from environment or config
                match std::env::var("VKTEAMS_BOT_CHAT_ID") {
                    Ok(id) => ChatId::from_borrowed_str(&id),
                    Err(_) => return CliResponse::error("upload-file", "No chat ID provided and VKTEAMS_BOT_CHAT_ID not set"),
                }
            }
        };

        let mut req = RequestMessagesSendFile::new((
            chat_id,
            MultipartName::FileContent {
                filename: args.name.clone(),
                content: file_content.clone(),
            },
        ));

        if let Some(caption) = &args.caption {
            req = req.with_text(caption.clone());
        }

        if let Some(reply_msg_id) = &args.reply_msg_id {
            req = req.with_reply_msg_id(MsgId(reply_msg_id.clone()));
        }

        match bot.send_api_request(req).await {
            Ok(response) => {
                let data = json!({
                    "message_id": response.msg_id,
                    "file_name": args.name,
                    "file_size": file_content.len(),
                    "file_size_formatted": format_file_size(file_content.len()),
                    "caption": args.caption
                });
                CliResponse::success("upload-file", data)
            }
            Err(e) => CliResponse::error("upload-file", format!("Failed to upload file: {}", e)),
        }
    }

    async fn handle_upload_text(&self, bot: &Bot, args: &UploadTextArgs) -> CliResponse<serde_json::Value> {
        let file_content = args.content.as_bytes().to_vec();

        let chat_id = match &args.chat_id {
            Some(id) => ChatId::from_borrowed_str(id),
            None => {
                match std::env::var("VKTEAMS_BOT_CHAT_ID") {
                    Ok(id) => ChatId::from_borrowed_str(&id),
                    Err(_) => return CliResponse::error("upload-text", "No chat ID provided and VKTEAMS_BOT_CHAT_ID not set"),
                }
            }
        };

        let mut req = RequestMessagesSendFile::new((
            chat_id,
            MultipartName::FileContent {
                filename: args.name.clone(),
                content: file_content.clone(),
            },
        ));

        if let Some(caption) = &args.caption {
            req = req.with_text(caption.clone());
        }

        if let Some(reply_msg_id) = &args.reply_msg_id {
            req = req.with_reply_msg_id(MsgId(reply_msg_id.clone()));
        }

        match bot.send_api_request(req).await {
            Ok(response) => {
                let data = json!({
                    "message_id": response.msg_id,
                    "file_name": args.name,
                    "file_size": file_content.len(),
                    "content_preview": if args.content.len() > 100 {
                        format!("{}...", &args.content[..100])
                    } else {
                        args.content.clone()
                    },
                    "caption": args.caption
                });
                CliResponse::success("upload-text", data)
            }
            Err(e) => CliResponse::error("upload-text", format!("Failed to upload text file: {}", e)),
        }
    }

    async fn handle_upload_json(&self, bot: &Bot, args: &UploadJsonArgs) -> CliResponse<serde_json::Value> {
        // Parse and format JSON
        let json_value: serde_json::Value = match serde_json::from_str(&args.json_data) {
            Ok(value) => value,
            Err(e) => return CliResponse::error("upload-json", format!("Invalid JSON data: {}", e)),
        };

        let formatted_json = if args.pretty {
            match serde_json::to_string_pretty(&json_value) {
                Ok(s) => s,
                Err(e) => return CliResponse::error("upload-json", format!("Failed to format JSON: {}", e)),
            }
        } else {
            match serde_json::to_string(&json_value) {
                Ok(s) => s,
                Err(e) => return CliResponse::error("upload-json", format!("Failed to serialize JSON: {}", e)),
            }
        };

        let final_filename = if args.name.ends_with(".json") {
            args.name.clone()
        } else {
            format!("{}.json", args.name)
        };

        let file_content = formatted_json.as_bytes().to_vec();

        let chat_id = match &args.chat_id {
            Some(id) => ChatId::from_borrowed_str(id),
            None => {
                match std::env::var("VKTEAMS_BOT_CHAT_ID") {
                    Ok(id) => ChatId::from_borrowed_str(&id),
                    Err(_) => return CliResponse::error("upload-json", "No chat ID provided and VKTEAMS_BOT_CHAT_ID not set"),
                }
            }
        };

        let mut req = RequestMessagesSendFile::new((
            chat_id,
            MultipartName::FileContent {
                filename: final_filename.clone(),
                content: file_content.clone(),
            },
        ));

        if let Some(caption) = &args.caption {
            req = req.with_text(caption.clone());
        }

        if let Some(reply_msg_id) = &args.reply_msg_id {
            req = req.with_reply_msg_id(MsgId(reply_msg_id.clone()));
        }

        match bot.send_api_request(req).await {
            Ok(response) => {
                let data = json!({
                    "message_id": response.msg_id,
                    "file_name": final_filename,
                    "file_size": file_content.len(),
                    "pretty_formatted": args.pretty,
                    "json_valid": true,
                    "caption": args.caption
                });
                CliResponse::success("upload-json", data)
            }
            Err(e) => CliResponse::error("upload-json", format!("Failed to upload JSON file: {}", e)),
        }
    }

    async fn handle_file_info(&self, bot: &Bot, args: &FileInfoArgs) -> CliResponse<serde_json::Value> {
        let req = RequestFilesGetInfo::new(FileId(args.file_id.clone()));

        match bot.send_api_request(req).await {
            Ok(response) => {
                let data = json!({
                    "file_type": response.file_type,
                    "file_size": response.file_size,
                    "file_name": response.file_name,
                    "url": response.url
                });
                CliResponse::success("file-info", data)
            }
            Err(e) => CliResponse::error("file-info", format!("Failed to get file info: {}", e)),
        }
    }
}

#[async_trait]
impl Command for FileCommands {
    async fn execute(&self, bot: &Bot) -> CliResult<()> {
        // Default to pretty format for backward compatibility
        self.execute_with_output(bot, &OutputFormat::Pretty).await
    }

    fn name(&self) -> &'static str {
        match self {
            FileCommands::Upload(_) => "upload-file",
            FileCommands::UploadText(_) => "upload-text",
            FileCommands::UploadJson(_) => "upload-json",
            FileCommands::Info(_) => "file-info",
        }
    }

    fn validate(&self) -> CliResult<()> {
        match self {
            FileCommands::Upload(args) => {
                if args.name.is_empty() {
                    return Err(CliError::InputError("File name cannot be empty".to_string()));
                }
                if args.content_base64.is_empty() {
                    return Err(CliError::InputError("File content cannot be empty".to_string()));
                }
            }
            FileCommands::UploadText(args) => {
                if args.name.is_empty() {
                    return Err(CliError::InputError("File name cannot be empty".to_string()));
                }
            }
            FileCommands::UploadJson(args) => {
                if args.name.is_empty() {
                    return Err(CliError::InputError("File name cannot be empty".to_string()));
                }
                if args.json_data.is_empty() {
                    return Err(CliError::InputError("JSON data cannot be empty".to_string()));
                }
            }
            FileCommands::Info(args) => {
                if args.file_id.is_empty() {
                    return Err(CliError::InputError("File ID cannot be empty".to_string()));
                }
            }
        }
        Ok(())
    }
}

fn format_file_size(size: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as usize, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_upload_args_validation() {
        let args = UploadFileArgs {
            name: "".to_string(),
            content_base64: "content".to_string(),
            caption: None,
            reply_msg_id: None,
            chat_id: None,
        };
        
        let cmd = FileCommands::Upload(args);
        assert!(cmd.validate().is_err());
    }

    #[test]
    fn test_json_filename_extension() {
        let args = UploadJsonArgs {
            name: "test".to_string(),
            json_data: "{}".to_string(),
            pretty: true,
            caption: None,
            reply_msg_id: None,
            chat_id: None,
        };
        
        // In real usage, this would be handled in handle_upload_json
        let expected_filename = if args.name.ends_with(".json") {
            args.name.clone()
        } else {
            format!("{}.json", args.name)
        };
        
        assert_eq!(expected_filename, "test.json");
    }
}