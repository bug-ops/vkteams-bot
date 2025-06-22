use std::process::Command;
use serde_json::{json, Value};
use vkteams_bot::{Bot, storage::StorageManager, config::UnifiedConfig};

/// Example of how MCP server integrates with CLI backend
/// This shows the CLI-as-Backend pattern used by the MCP server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 VK Teams MCP Integration Example");
    
    // This simulates how MCP server calls CLI commands
    demonstrate_mcp_cli_bridge().await?;
    
    // Show direct library usage with storage
    demonstrate_storage_context().await?;
    
    Ok(())
}

/// Simulates how MCP server calls CLI commands
async fn demonstrate_mcp_cli_bridge() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📡 MCP CLI Bridge Example:");
    
    // Simulate MCP tool calls through CLI
    let cli_commands = vec![
        // Send message
        vec!["send-text", "-u", "test_chat", "-m", "Hello from MCP!"],
        
        // Search messages semantically  
        vec!["storage", "search-semantic", "project deadlines"],
        
        // Get storage statistics
        vec!["storage", "stats"],
        
        // Get conversation context
        vec!["storage", "get-context", "-c", "test_chat", "--limit", "10"],
    ];
    
    for cmd in cli_commands {
        println!("🔧 Executing CLI command: vkteams-bot-cli {}", cmd.join(" "));
        
        // This is how MCP server internally calls CLI
        let output = Command::new("vkteams-bot-cli")
            .args(&cmd)
            .output();
            
        match output {
            Ok(result) => {
                if result.status.success() {
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    println!("✅ CLI Response: {}", stdout.trim());
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    println!("❌ CLI Error: {}", stderr.trim());
                }
            }
            Err(e) => {
                println!("⚠️  Could not execute CLI (make sure vkteams-bot-cli is installed): {}", e);
            }
        }
    }
    
    Ok(())
}

/// Direct library usage for storage and context management
async fn demonstrate_storage_context() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💾 Storage Context Example:");
    
    let config_result = UnifiedConfig::from_file(".config/shared-config.toml");
    
    match config_result {
        Ok(config) => {
            let storage = StorageManager::from_config(&config.storage).await?;
            
            // Get conversation context (useful for AI)
            let context = storage.get_context("example_chat", 20).await?;
            
            println!("📝 Retrieved {} context messages", context.len());
            
            // Build context for AI prompt
            let mut ai_context = String::new();
            for msg in context.iter().take(5) {
                ai_context.push_str(&format!(
                    "{}: {}\n", 
                    msg.from_name.as_deref().unwrap_or("User"),
                    msg.text
                ));
            }
            
            if !ai_context.is_empty() {
                println!("🤖 AI Context Preview:\n{}", ai_context);
            }
            
            // Search for related messages
            let related = storage.search_semantic("meeting tomorrow", 3).await?;
            println!("🔍 Found {} semantically related messages", related.len());
            
        }
        Err(e) => {
            println!("⚠️  Could not load config ({}), skipping storage demo", e);
        }
    }
    
    Ok(())
}

/// Example MCP tool definitions that the server provides
fn example_mcp_tools() -> Value {
    json!({
        "tools": [
            {
                "name": "send_message",
                "description": "Send a text message to a VK Teams chat",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "chatId": {"type": "string"},
                        "text": {"type": "string"}
                    },
                    "required": ["chatId", "text"]
                }
            },
            {
                "name": "search_messages",
                "description": "Search messages semantically using AI embeddings",
                "inputSchema": {
                    "type": "object", 
                    "properties": {
                        "query": {"type": "string"},
                        "limit": {"type": "number", "default": 10}
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "get_context",
                "description": "Get conversation context for AI processing",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "chatId": {"type": "string"},
                        "limit": {"type": "number", "default": 50}
                    },
                    "required": ["chatId"]
                }
            },
            {
                "name": "upload_file",
                "description": "Upload and send a file to a chat",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "chatId": {"type": "string"},
                        "filePath": {"type": "string"},
                        "caption": {"type": "string"}
                    },
                    "required": ["chatId", "filePath"]
                }
            }
        ]
    })
}