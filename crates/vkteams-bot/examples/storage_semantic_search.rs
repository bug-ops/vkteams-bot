use std::time::Duration;
use tokio::time::sleep;
use vkteams_bot::{Bot, storage::StorageManager, config::UnifiedConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from .config/shared-config.toml
    let config = UnifiedConfig::from_file(".config/shared-config.toml")?;
    
    // Initialize bot and storage
    let bot = Bot::from_config(&config.api)?;
    let storage = StorageManager::from_config(&config.storage).await?;
    
    println!("🚀 Starting VK Teams Bot with semantic search...");
    
    // Monitor and save events with semantic indexing
    loop {
        // Get new events
        let events = bot.get_events().await?;
        
        if !events.is_empty() {
            println!("📥 Processing {} new events", events.len());
            
            // Save events to database with automatic embedding generation
            storage.save_events(&events).await?;
            
            // Demonstrate semantic search
            let search_queries = vec![
                "project deadlines",
                "meeting schedule", 
                "technical issues",
                "deployment status"
            ];
            
            for query in search_queries {
                println!("\n🔍 Searching for: '{}'", query);
                
                let results = storage.search_semantic(query, 5).await?;
                
                if results.is_empty() {
                    println!("   No results found");
                } else {
                    for (i, message) in results.iter().enumerate() {
                        println!(
                            "   {}. {} (similarity: {:.3})\n      \"{}\"",
                            i + 1,
                            message.from_name.as_deref().unwrap_or("Unknown"),
                            message.similarity_score.unwrap_or(0.0),
                            message.text.chars().take(100).collect::<String>()
                                + if message.text.len() > 100 { "..." } else { "" }
                        );
                    }
                }
            }
        }
        
        // Check storage statistics
        let stats = storage.get_stats().await?;
        println!("\n📊 Storage stats: {} events, {} messages stored", 
                 stats.total_events, stats.total_messages);
        
        sleep(Duration::from_secs(10)).await;
    }
}