use std::path::Path;
use base64::{Engine as _, engine::general_purpose};
use serde_json::json;
use vkteams_bot::{Bot, config::UnifiedConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 Advanced File Operations Example");
    
    let config = UnifiedConfig::from_file(".config/shared-config.toml")?;
    let bot = Bot::from_config(&config.api)?;
    
    let test_chat = "your_test_chat_id"; // Replace with actual chat ID
    
    // 1. Upload JSON data as formatted file
    demonstrate_json_upload(&bot, test_chat).await?;
    
    // 2. Upload base64 encoded content
    demonstrate_base64_upload(&bot, test_chat).await?;
    
    // 3. Batch file upload
    demonstrate_batch_upload(&bot, test_chat).await?;
    
    // 4. Create and upload text file on the fly
    demonstrate_text_file_creation(&bot, test_chat).await?;
    
    Ok(())
}

async fn demonstrate_json_upload(bot: &Bot, chat_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 JSON File Upload:");
    
    let analytics_data = json!({
        "report": "Monthly Team Analytics",
        "period": "2024-01",
        "metrics": {
            "total_messages": 15420,
            "active_users": 87,
            "files_shared": 234,
            "peak_activity_hour": 14
        },
        "top_channels": [
            {"name": "general", "messages": 5420},
            {"name": "development", "messages": 3890},
            {"name": "marketing", "messages": 2340}
        ],
        "summary": "High engagement month with 23% increase in activity"
    });
    
    // Create formatted JSON content
    let json_content = serde_json::to_string_pretty(&analytics_data)?;
    
    // Save to temporary file and upload
    let temp_path = "/tmp/analytics_report.json";
    tokio::fs::write(temp_path, json_content).await?;
    
    let response = bot.send_file(
        chat_id,
        temp_path,
        Some("📊 Monthly Analytics Report - January 2024")
    ).await?;
    
    println!("✅ JSON report uploaded: {:?}", response.file_id);
    
    // Clean up
    tokio::fs::remove_file(temp_path).await.ok();
    
    Ok(())
}

async fn demonstrate_base64_upload(bot: &Bot, chat_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🖼️  Base64 Image Upload:");
    
    // Create a simple SVG chart as example
    let svg_content = r#"<svg width="400" height="200" xmlns="http://www.w3.org/2000/svg">
        <rect width="400" height="200" fill="#f0f0f0"/>
        <text x="200" y="30" text-anchor="middle" font-family="Arial" font-size="16" fill="#333">
            Team Activity Chart
        </text>
        <rect x="50" y="60" width="40" height="100" fill="#4CAF50"/>
        <rect x="120" y="80" width="40" height="80" fill="#2196F3"/>
        <rect x="190" y="70" width="40" height="90" fill="#FF9800"/>
        <rect x="260" y="50" width="40" height="110" fill="#9C27B0"/>
        <text x="70" y="180" text-anchor="middle" font-size="12">Week 1</text>
        <text x="140" y="180" text-anchor="middle" font-size="12">Week 2</text>
        <text x="210" y="180" text-anchor="middle" font-size="12">Week 3</text>
        <text x="280" y="180" text-anchor="middle" font-size="12">Week 4</text>
    </svg>"#;
    
    // Encode to base64
    let base64_content = general_purpose::STANDARD.encode(svg_content.as_bytes());
    
    // Save base64 content to file and upload
    let temp_path = "/tmp/activity_chart.svg";
    tokio::fs::write(temp_path, svg_content).await?;
    
    let response = bot.send_file(
        chat_id,
        temp_path,
        Some("📈 Team Activity Visualization")
    ).await?;
    
    println!("✅ SVG chart uploaded: {:?}", response.file_id);
    println!("   Base64 size: {} chars", base64_content.len());
    
    // Clean up
    tokio::fs::remove_file(temp_path).await.ok();
    
    Ok(())
}

async fn demonstrate_batch_upload(bot: &Bot, chat_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📦 Batch File Upload:");
    
    // Create multiple temporary files
    let files = vec![
        ("/tmp/report1.txt", "Team Meeting Notes - January 15\n\n- Discussed Q1 goals\n- Assigned tasks\n- Next meeting: Jan 22"),
        ("/tmp/report2.txt", "Sprint Retrospective\n\n- What went well: Good collaboration\n- What to improve: Code review speed\n- Action items: Update process docs"),
        ("/tmp/report3.txt", "Technical Debt Analysis\n\n- Identified 15 critical issues\n- Estimated 40 hours of work\n- Priority: Database optimization"),
    ];
    
    // Create files
    for (path, content) in &files {
        tokio::fs::write(path, content).await?;
    }
    
    // Upload each file
    for (i, (path, _)) in files.iter().enumerate() {
        let filename = Path::new(path).file_name().unwrap().to_str().unwrap();
        
        let response = bot.send_file(
            chat_id,
            path,
            Some(&format!("📄 Document {}/{}: {}", i + 1, files.len(), filename))
        ).await?;
        
        println!("✅ Uploaded {}: {:?}", filename, response.file_id);
        
        // Clean up
        tokio::fs::remove_file(path).await.ok();
    }
    
    Ok(())
}

async fn demonstrate_text_file_creation(bot: &Bot, chat_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📝 Dynamic Text File Creation:");
    
    // Generate system status report
    let system_status = format!(
        "VK Teams Bot System Status Report\n\
         Generated: {}\n\n\
         🚀 Bot Status: Operational\n\
         💾 Storage: Connected\n\
         🤖 MCP Server: Active\n\
         📊 Messages Processed: 1,247\n\
         🔍 Searches Performed: 89\n\
         📁 Files Handled: 23\n\n\
         Recent Activities:\n\
         - Semantic search implemented ✅\n\
         - Vector database optimized ✅\n\
         - MCP integration completed ✅\n\
         - File operations enhanced ✅\n\n\
         System Health: EXCELLENT 💚",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    
    // Save and upload
    let temp_path = "/tmp/system_status.txt";
    tokio::fs::write(temp_path, system_status).await?;
    
    let response = bot.send_file(
        chat_id,
        temp_path,
        Some("🔧 System Status Report - Auto-generated")
    ).await?;
    
    println!("✅ System status report uploaded: {:?}", response.file_id);
    
    // Clean up
    tokio::fs::remove_file(temp_path).await.ok();
    
    Ok(())
}