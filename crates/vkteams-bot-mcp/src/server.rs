//! New MCP Server implementation using CLI Bridge
//!
//! This module contains the new MCP server implementation that uses the CLI
//! bridge instead of direct library calls. This ensures single source of truth
//! for all business logic in the CLI.

use crate::cli_bridge::CliBridge;
use crate::errors::BridgeError;
use crate::mcp_bridge_trait::{McpDiagnostics, McpMessaging};
use rmcp::{
    RoleServer, ServerHandler,
    handler::server::tool::{Parameters, ToolRouter},
    model::{
        CallToolResult, Content, CreateElicitationRequestParam, ElicitationAction, ErrorCode,
        ErrorData, ServerCapabilities, ServerInfo,
    },
    schemars,
    service::Peer,
    tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::result::Result;
use std::sync::Arc;
use tracing::{error, warn};
use vkteams_bot::config::UnifiedConfig;

pub type MCPResult = Result<CallToolResult, ErrorData>;

#[derive(Debug, Clone)]
pub struct Server {
    pub cli: Arc<CliBridge>,
    pub config: UnifiedConfig,
    pub tool_router: ToolRouter<Self>,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert CLI bridge result to MCP result with enhanced error handling
/// This function is used internally by McpCliBridge implementation
#[inline]
pub(crate) fn convert_bridge_result(result: Result<Value, BridgeError>) -> MCPResult {
    match result {
        Ok(json_response) => {
            // CLI already returns structured JSON, just pass it through
            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string(&json_response).unwrap_or_else(|_| "{}".to_string()),
            )]))
        }
        Err(e) => {
            // Map error types to appropriate error codes
            let (code, message, data) = match &e {
                BridgeError::RateLimit(msg) => {
                    warn!("Rate limit error: {}", msg);
                    (-429, format!("Rate limit exceeded: {}", msg), None)
                }
                BridgeError::Timeout(duration) => {
                    error!("Command timed out after {:?}", duration);
                    (
                        -504,
                        format!("Command timed out after {:?}", duration),
                        None,
                    )
                }
                BridgeError::CliReturnedError(info) => {
                    error!("CLI returned error: {:?}", info);
                    let code = match info.code.as_deref() {
                        Some("NOT_FOUND") => -404,
                        Some("UNAUTHORIZED") => -401,
                        Some("FORBIDDEN") => -403,
                        Some("INVALID_INPUT") => -400,
                        Some("RATE_LIMIT") => -429,
                        _ => -500,
                    };
                    (code, info.message.clone(), info.details.clone())
                }
                BridgeError::CliNotFound(path) => {
                    error!("CLI not found at: {}", path);
                    (
                        -503,
                        format!("Service unavailable: CLI not found at {}", path),
                        None,
                    )
                }
                BridgeError::InvalidResponse(err) => {
                    error!("Invalid JSON response: {}", err);
                    (-502, format!("Invalid response from CLI: {}", err), None)
                }
                _ => {
                    error!("CLI bridge error: {}", e);
                    (-500, format!("Internal error: {}", e), None)
                }
            };

            Err(ErrorData {
                code: ErrorCode(code),
                message: message.into(),
                data,
            })
        }
    }
}

// Define input types for our tools
#[derive(Serialize, Deserialize, schemars::JsonSchema)]
pub struct SendTextRequest {
    pub text: String,
    pub chat_id: Option<String>,
    pub reply_msg_id: Option<String>,
}

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
pub struct SendFileRequest {
    pub file_path: String,
    pub chat_id: Option<String>,
    pub caption: Option<String>,
}

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetEventsRequest {
    pub last_event_id: Option<String>,
    pub poll_time: Option<u64>,
}

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetChatId {
    pub chat_id: String,
}

#[tool_router(router = tool_router)]
impl Server {
    pub fn new() -> Self {
        let mut config = Self::load_config();
        config.apply_env_overrides();

        let cli = CliBridge::new(&config).expect("Failed to create CLI bridge");

        Self {
            cli: Arc::new(cli),
            config,
            tool_router: Self::tool_router(),
        }
    }

    /// Load configuration from file or use defaults
    fn load_config() -> UnifiedConfig {
        // Try environment variable first (highest priority)
        if let Ok(config_path) = std::env::var("VKTEAMS_BOT_CONFIG") {
            if let Ok(config) = UnifiedConfig::load_from_file(&config_path) {
                return config;
            }
        }

        // Try to load from standard locations
        let config_paths = [
            "config.toml",
            "shared-config.toml",
            "/etc/vkteams-bot/config.toml",
        ];

        // Try static paths
        for path in &config_paths {
            match UnifiedConfig::load_from_file(path) {
                Ok(config) => {
                    return config;
                }
                Err(_) => {
                    // Silent continue for expected missing files
                }
            }
        }

        // Try user config directory
        if let Some(home_dir) = dirs::home_dir() {
            let user_home_path = home_dir.join(".config/vkteams-bot/config.toml");
            if let Ok(config) = UnifiedConfig::load_from_file(&user_home_path) {
                return config;
            }
        }

        // Fall back to default (env overrides will be applied in new/with_config)
        UnifiedConfig::default()
    }

    pub fn bridge(&self) -> Arc<CliBridge> {
        Arc::clone(&self.cli)
    }

    // === Messaging Commands ===

    #[tool(description = "Send text message to chat")]
    async fn send_text(
        &self,
        params: Parameters<SendTextRequest>,
        peer: Peer<RoleServer>,
    ) -> String {
        let mut request = params.0;

        // If chat_id is not provided, request it via elicitation
        if request.chat_id.is_none() {
            let elicitation_param = CreateElicitationRequestParam {
                message: "Please provide the chat_id where the message should be sent".to_string(),
                // requested_schema: GetChatId.json_schema(),
                requested_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "chat_id": {
                            "type": "string",
                            "description": "The ID of the chat where the message should be sent"
                        }
                    },
                    "required": ["chat_id"]
                }),
            };

            match peer.create_elicitation(elicitation_param).await {
                Ok(elicitation_result) => match elicitation_result.action {
                    ElicitationAction::Accept => {
                        if let Some(content) = elicitation_result.content {
                            if let Some(chat_id) = content.get("chat_id").and_then(|v| v.as_str()) {
                                request.chat_id = Some(chat_id.to_string());
                            } else {
                                return format!(
                                    "{{\"error\": \"Invalid chat_id format in elicitation response\"}}"
                                );
                            }
                        } else {
                            return format!(
                                "{{\"error\": \"No content provided in elicitation response\"}}"
                            );
                        }
                    }
                    ElicitationAction::Reject => {
                        return format!("{{\"error\": \"User rejected chat_id elicitation\"}}");
                    }
                    ElicitationAction::Cancel => {
                        return format!("{{\"error\": \"User cancelled chat_id elicitation\"}}");
                    }
                },
                Err(e) => {
                    return format!("{{\"error\": \"Failed to elicit chat_id: {}\"}}", e);
                }
            }
        }

        match self
            .cli
            .send_text_mcp(
                &request.text,
                request.chat_id.as_deref(),
                request.reply_msg_id.as_deref(),
            )
            .await
        {
            Ok(result) => serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string()),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Send file to chat")]
    async fn send_file(
        &self,
        params: Parameters<SendFileRequest>,
        peer: Peer<RoleServer>,
    ) -> String {
        let mut request = params.0;

        // If chat_id is not provided, request it via elicitation
        if request.chat_id.is_none() {
            let elicitation_param = CreateElicitationRequestParam {
                message: "Please provide the chat_id where the file should be sent".to_string(),
                requested_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "chat_id": {
                            "type": "string",
                            "description": "The ID of the chat where the file should be sent"
                        }
                    },
                    "required": ["chat_id"]
                }),
            };

            match peer.create_elicitation(elicitation_param).await {
                Ok(elicitation_result) => match elicitation_result.action {
                    ElicitationAction::Accept => {
                        if let Some(content) = elicitation_result.content {
                            if let Some(chat_id) = content.get("chat_id").and_then(|v| v.as_str()) {
                                request.chat_id = Some(chat_id.to_string());
                            } else {
                                return format!(
                                    "{{\"error\": \"Invalid chat_id format in elicitation response\"}}"
                                );
                            }
                        } else {
                            return format!(
                                "{{\"error\": \"No content provided in elicitation response\"}}"
                            );
                        }
                    }
                    ElicitationAction::Reject => {
                        return format!("{{\"error\": \"User rejected chat_id elicitation\"}}");
                    }
                    ElicitationAction::Cancel => {
                        return format!("{{\"error\": \"User cancelled chat_id elicitation\"}}");
                    }
                },
                Err(e) => {
                    return format!("{{\"error\": \"Failed to elicit chat_id: {}\"}}", e);
                }
            }
        }

        match self
            .cli
            .send_file_mcp(
                &request.file_path,
                request.chat_id.as_deref(),
                request.caption.as_deref(),
            )
            .await
        {
            Ok(result) => serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string()),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Get information about the bot")]
    async fn self_get(&self) -> String {
        match self.cli.get_self_mcp().await {
            Ok(result) => serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string()),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Get events from the bot")]
    async fn events_get(&self, params: Parameters<GetEventsRequest>) -> String {
        let request = params.0;
        match self
            .cli
            .get_events_mcp(request.last_event_id.as_deref(), request.poll_time)
            .await
        {
            Ok(result) => serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string()),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo {
        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_prompts()
            .build();
        ServerInfo {
            capabilities,
            instructions: Some(r#"VKTeams MCP Server — a server for managing a VK Teams bot via MCP (Model Context Protocol).
        
        This server uses CLI-as-backend architecture for unified command execution.
        
        Tools:
            - Send text messages to chat (send_text)
            - Get bot information (self_get)
            - Send files (send_file)
            - Get events (events_get)
            "#.into()),
            ..Default::default()
        }
    }
}
