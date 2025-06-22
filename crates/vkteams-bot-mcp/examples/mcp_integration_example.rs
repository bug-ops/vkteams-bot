use std::process::Command;

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
        println!(
            "🔧 Executing CLI command: vkteams-bot-cli {}",
            cmd.join(" ")
        );

        // This is how MCP server internally calls CLI
        let output = Command::new("vkteams-bot-cli").args(&cmd).output();

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
                println!(
                    "⚠️  Could not execute CLI (make sure vkteams-bot-cli is installed): {}",
                    e
                );
            }
        }
    }

    Ok(())
}

/// MCP server architecture explanation  
async fn demonstrate_storage_context() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💾 MCP Architecture Overview:");

    println!("🏗️  MCP Server uses CLI-as-Backend pattern:");
    println!("   1. MCP receives tool calls from AI");
    println!("   2. MCP translates to CLI commands");
    println!("   3. CLI executes with full library access");
    println!("   4. Results returned to AI via MCP");

    println!("\n📋 Available storage operations via CLI:");
    println!("   • vkteams-bot-cli storage search-text <query>");
    println!("   • vkteams-bot-cli storage search-semantic <query>");
    println!("   • vkteams-bot-cli storage get-context -c <chat_id>");
    println!("   • vkteams-bot-cli storage stats");

    println!("\n🔗 This ensures:");
    println!("   ✅ Single source of truth (CLI has all logic)");
    println!("   ✅ Easy testing and debugging");
    println!("   ✅ Consistent behavior across interfaces");

    Ok(())
}
