//! Storage and database commands

use crate::commands::{Command, OutputFormat};
use crate::errors::prelude::{Result as CliResult};
use crate::output::{CliResponse, OutputFormatter};
use async_trait::async_trait;
use clap::Subcommand;
use serde_json::json;
use vkteams_bot::prelude::*;

use vkteams_bot::storage::{StorageManager, StorageConfig};
use vkteams_bot::storage::config::{DatabaseConfig, StorageSettings};

#[derive(Debug, Clone, Subcommand)]
pub enum StorageCommands {
    /// Database operations
    Database {
        #[command(subcommand)]
        action: DatabaseAction,
    },
    /// Search operations
    Search {
        #[command(subcommand)]
        action: SearchAction,
    },
    /// Context management
    Context {
        #[command(subcommand)]
        action: ContextAction,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum DatabaseAction {
    /// Initialize database and run migrations
    Init,
    /// Get database statistics
    Stats {
        #[arg(long)]
        chat_id: Option<String>,
        #[arg(long)]
        since: Option<String>,
    },
    /// Clean up old events
    Cleanup {
        #[arg(long, default_value = "365")]
        older_than_days: u32,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum SearchAction {
    /// Semantic search using vector similarity
    Semantic {
        /// Search query
        query: String,
        #[arg(long)]
        chat_id: Option<String>,
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    /// Full-text search in messages
    Text {
        /// Search query
        query: String,
        #[arg(long)]
        chat_id: Option<String>,
        #[arg(long, default_value = "10")]
        limit: i64,
    },
    /// Advanced search with filters
    Advanced {
        #[arg(long)]
        user_id: Option<String>,
        #[arg(long)]
        event_type: Option<String>,
        #[arg(long)]
        since: Option<String>,
        #[arg(long)]
        until: Option<String>,
        #[arg(long, default_value = "10")]
        limit: i64,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ContextAction {
    /// Get conversation context
    Get {
        #[arg(long)]
        chat_id: Option<String>,
        #[arg(long, value_enum, default_value = "recent")]
        context_type: ContextType,
        #[arg(long)]
        timeframe: Option<String>,
    },
    /// Create new context
    Create {
        #[arg(long)]
        chat_id: String,
        #[arg(long)]
        summary: String,
        #[arg(long)]
        context_type: String,
    },
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ContextType {
    Recent,
    Topic,
    UserProfile,
}

impl StorageCommands {
    pub async fn execute_with_output(
        &self,
        _bot: &Bot,
        output_format: &OutputFormat,
    ) -> CliResult<()> {
        {
            let response = match self {
                StorageCommands::Database { action } => self.handle_database(action).await,
                StorageCommands::Search { action } => self.handle_search(action).await,
                StorageCommands::Context { action } => self.handle_context(action).await,
            };

            OutputFormatter::print(&response, output_format)?;
            
            if !response.success {
                std::process::exit(1);
            }
        }
        
        Ok(())
    }

    pub async fn get_storage_manager(&self) -> std::result::Result<StorageManager, String> {
        // Try to load storage configuration
        let config = match self.load_storage_config().await {
            Ok(config) => config,
            Err(e) => return Err(format!("Failed to load storage configuration: {}", e)),
        };

        match StorageManager::new(&config).await {
            Ok(storage) => Ok(storage),
            Err(e) => Err(format!("Failed to initialize storage manager: {}", e)),
        }
    }

    pub async fn load_storage_config(&self) -> std::result::Result<StorageConfig, String> {
        // Try to load from main library configuration first
        #[cfg(feature = "storage")]
        {
            use vkteams_bot::config::get_config;
            
            // Try to load from configuration file
            if let Ok(main_config) = get_config() {
                return Ok(main_config.get_storage_config());
            }
        }
        
        // Fallback to environment-based configuration if main config fails
        let database_url = std::env::var("DATABASE_URL")
            .or_else(|_| std::env::var("VKTEAMS_BOT_DATABASE_URL"))
            .unwrap_or_else(|_| "postgresql://localhost/vkteams_bot".to_string());

        let config = StorageConfig {
            database: DatabaseConfig {
                url: database_url,
                max_connections: 20,
                connection_timeout: 30,
                auto_migrate: true,
            },
            settings: StorageSettings {
                event_retention_days: 365,
                cleanup_interval_hours: 24,
                batch_size: 100,
                max_memory_events: 10000,
            },
            ..Default::default()
        };

        Ok(config)
    }

    pub async fn handle_database(&self, action: &DatabaseAction) -> CliResponse<serde_json::Value> {
        let command_name = match action {
            DatabaseAction::Init => "database-init",
            DatabaseAction::Stats { .. } => "database-stats",
            DatabaseAction::Cleanup { .. } => "database-cleanup",
        };

        let storage = match self.get_storage_manager().await {
            Ok(storage) => storage,
            Err(e) => return CliResponse::error(command_name, e.to_string()),
        };

        match action {
            DatabaseAction::Init => {
                match storage.initialize().await {
                    Ok(_) => {
                        let data = json!({
                            "message": "Database initialized successfully",
                            "migrations_applied": true
                        });
                        CliResponse::success("database-init", data)
                    }
                    Err(e) => CliResponse::error("database-init", format!("Failed to initialize database: {}", e)),
                }
            }
            DatabaseAction::Stats { chat_id, since: _since } => {
                match storage.get_stats(chat_id.as_deref()).await {
                    Ok(stats) => {
                        let data = json!({
                            "total_events": stats.total_events,
                            "total_messages": stats.total_messages,
                            "unique_chats": stats.unique_chats,
                            "unique_users": stats.unique_users,
                            "events_last_24h": stats.events_last_24h,
                            "events_last_week": stats.events_last_week,
                            "oldest_event": stats.oldest_event,
                            "newest_event": stats.newest_event,
                            "storage_size_bytes": stats.storage_size_bytes
                        });
                        CliResponse::success("database-stats", data)
                    }
                    Err(e) => CliResponse::error("database-stats", format!("Failed to get stats: {}", e)),
                }
            }
            DatabaseAction::Cleanup { older_than_days } => {
                match storage.cleanup_old_data(*older_than_days).await {
                    Ok(deleted_count) => {
                        let data = json!({
                            "deleted_events": deleted_count,
                            "older_than_days": older_than_days
                        });
                        CliResponse::success("database-cleanup", data)
                    }
                    Err(e) => CliResponse::error("database-cleanup", format!("Failed to cleanup: {}", e)),
                }
            }
        }
    }

    pub async fn handle_search(&self, action: &SearchAction) -> CliResponse<serde_json::Value> {
        let command_name = match action {
            SearchAction::Semantic { .. } => "search-semantic",
            SearchAction::Text { .. } => "search-text",
            SearchAction::Advanced { .. } => "search-advanced",
        };

        let storage = match self.get_storage_manager().await {
            Ok(storage) => storage,
            Err(e) => return CliResponse::error(command_name, e.to_string()),
        };

        match action {
            SearchAction::Semantic { query, chat_id, limit } => {
                #[cfg(feature = "vector-search")]
                {
                    match storage.search_similar_events(query, chat_id.as_deref(), *limit).await {
                        Ok(results) => {
                            let data = json!({
                                "query": query,
                                "results_count": results.len(),
                                "results": results.into_iter().map(|r| json!({
                                    "id": r.id,
                                    "content": r.content,
                                    "metadata": r.metadata,
                                    "score": r.score,
                                    "created_at": r.created_at
                                })).collect::<Vec<_>>()
                            });
                            CliResponse::success("search-semantic", data)
                        }
                        Err(e) => CliResponse::error("search-semantic", format!("Semantic search failed: {}", e)),
                    }
                }
                #[cfg(not(feature = "vector-search"))]
                {
                    let _ = (query, chat_id, limit); // Avoid unused variable warnings
                    CliResponse::error("search-semantic", "Vector search feature not enabled")
                }
            }
            SearchAction::Text { query, chat_id, limit } => {
                match storage.search_messages(query, chat_id.as_deref(), *limit).await {
                    Ok(messages) => {
                        let data = json!({
                            "query": query,
                            "results_count": messages.len(),
                            "messages": messages.into_iter().map(|m| json!({
                                "id": m.id,
                                "message_id": m.message_id,
                                "user_id": m.user_id,
                                "text": m.text,
                                "timestamp": m.timestamp,
                                "chat_id": m.chat_id
                            })).collect::<Vec<_>>()
                        });
                        CliResponse::success("search-text", data)
                    }
                    Err(e) => CliResponse::error("search-text", format!("Search failed: {}", e)),
                }
            }
            SearchAction::Advanced { user_id: _, event_type: _, since: _, until: _, limit: _ } => {
                // TODO: Implement advanced search with filters
                CliResponse::error("search-advanced", "Advanced search not implemented yet")
            }
        }
    }

    pub async fn handle_context(&self, action: &ContextAction) -> CliResponse<serde_json::Value> {
        let command_name = match action {
            ContextAction::Get { .. } => "context-get",
            ContextAction::Create { .. } => "context-create",
        };

        let storage = match self.get_storage_manager().await {
            Ok(storage) => storage,
            Err(e) => return CliResponse::error(command_name, e.to_string()),
        };

        match action {
            ContextAction::Get { chat_id, context_type: _, timeframe: _ } => {
                let default_chat_id = std::env::var("VKTEAMS_BOT_CHAT_ID").unwrap_or_else(|_| "default".to_string());
                let chat_id_ref = chat_id.as_deref().unwrap_or(&default_chat_id);
                
                // Get recent events as context
                match storage.get_recent_events(Some(chat_id_ref), None, 20).await {
                        Ok(events) => {
                            let data = json!({
                                "chat_id": chat_id_ref,
                                "context_type": "recent",
                                "events_count": events.len(),
                                "events": events.into_iter().map(|e| json!({
                                    "id": e.id,
                                    "event_id": e.event_id,
                                    "event_type": e.event_type,
                                    "timestamp": e.timestamp,
                                    "user_id": e.user_id
                                })).collect::<Vec<_>>()
                            });
                            CliResponse::success("context-get", data)
                        }
                        Err(e) => CliResponse::error("context-get", format!("Failed to get context: {}", e)),
                    }
            }
            ContextAction::Create { chat_id: _, summary: _, context_type: _ } => {
                // TODO: Implement context creation
                CliResponse::error("context-create", "Context creation not implemented yet")
            }
        }
    }
}

#[async_trait]
impl Command for StorageCommands {
    async fn execute(&self, bot: &Bot) -> CliResult<()> {
        self.execute_with_output(bot, &OutputFormat::Pretty).await
    }

    fn name(&self) -> &'static str {
        match self {
            StorageCommands::Database { .. } => "database",
            StorageCommands::Search { .. } => "search",
            StorageCommands::Context { .. } => "context",
        }
    }

    fn validate(&self) -> CliResult<()> {
        // Add validation logic here if needed
        Ok(())
    }
}

#[cfg(test)]
mod storage_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_commands_name() {
        let cmd = StorageCommands::Database { 
            action: DatabaseAction::Init 
        };
        assert_eq!(cmd.name(), "database");

        let cmd = StorageCommands::Search { 
            action: SearchAction::Text { 
                query: "test".to_string(), 
                chat_id: None, 
                limit: 10 
            } 
        };
        assert_eq!(cmd.name(), "search");
    }

    #[test]
    fn test_context_type_enum() {
        let context_type = ContextType::Recent;
        assert!(matches!(context_type, ContextType::Recent));
    }
}