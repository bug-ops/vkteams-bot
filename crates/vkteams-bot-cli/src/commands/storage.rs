//! Storage and database commands

use crate::commands::{Command, OutputFormat};
use crate::errors::prelude::Result as CliResult;
use crate::output::{CliResponse, OutputFormatter};
use async_trait::async_trait;
use clap::Subcommand;
use serde_json::json;
use vkteams_bot::prelude::*;

use vkteams_bot::storage::config::{DatabaseConfig, StorageSettings};
use vkteams_bot::storage::{StorageConfig, StorageManager};

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
    /// Get vector store performance metrics
    VectorMetrics,
    /// Perform vector store maintenance (vacuum, analyze, reindex)
    VectorMaintenance,
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
                ssl: Default::default(),
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
            DatabaseAction::VectorMetrics => "database-vector-metrics",
            DatabaseAction::VectorMaintenance => "database-vector-maintenance",
        };

        let storage = match self.get_storage_manager().await {
            Ok(storage) => storage,
            Err(e) => return CliResponse::error(command_name, e.to_string()),
        };

        match action {
            DatabaseAction::Init => match storage.initialize().await {
                Ok(_) => {
                    let data = json!({
                        "message": "Database initialized successfully",
                        "migrations_applied": true
                    });
                    CliResponse::success("database-init", data)
                }
                Err(e) => CliResponse::error(
                    "database-init",
                    format!("Failed to initialize database: {}", e),
                ),
            },
            DatabaseAction::Stats {
                chat_id,
                since: _since,
            } => match storage.get_stats(chat_id.as_deref()).await {
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
                Err(e) => {
                    CliResponse::error("database-stats", format!("Failed to get stats: {}", e))
                }
            },
            DatabaseAction::Cleanup { older_than_days } => {
                match storage.cleanup_old_data(*older_than_days).await {
                    Ok(deleted_count) => {
                        let data = json!({
                            "deleted_events": deleted_count,
                            "older_than_days": older_than_days
                        });
                        CliResponse::success("database-cleanup", data)
                    }
                    Err(e) => {
                        CliResponse::error("database-cleanup", format!("Failed to cleanup: {}", e))
                    }
                }
            }
            DatabaseAction::VectorMetrics => {
                #[cfg(feature = "vector-search")]
                {
                    match storage.get_vector_metrics().await {
                        Ok(Some(metrics)) => {
                            let data = json!({
                                "collection_name": metrics.collection_name,
                                "total_documents": metrics.total_documents,
                                "total_size_bytes": metrics.total_size_bytes,
                                "total_size_mb": metrics.total_size_bytes as f64 / 1024.0 / 1024.0,
                                "index_size_bytes": metrics.index_size_bytes,
                                "index_size_mb": metrics.index_size_bytes as f64 / 1024.0 / 1024.0,
                                "dimensions": metrics.dimensions,
                                "performance": {
                                    "total_queries": metrics.total_queries,
                                    "failed_queries": metrics.failed_queries,
                                    "success_rate": if metrics.total_queries > 0 {
                                        1.0 - (metrics.failed_queries as f64 / metrics.total_queries as f64)
                                    } else { 0.0 },
                                    "avg_query_time_ms": metrics.avg_query_time_ms,
                                    "last_query_time_ms": metrics.last_query_time_ms
                                },
                                "index_usage": {
                                    "index_scans": metrics.index_usage.index_scans,
                                    "index_tuples_read": metrics.index_usage.index_tuples_read,
                                    "index_tuples_fetched": metrics.index_usage.index_tuples_fetched,
                                    "cache_hit_ratio": metrics.index_usage.cache_hit_ratio,
                                    "index_blocks_read": metrics.index_usage.index_blocks_read,
                                    "index_blocks_hit": metrics.index_usage.index_blocks_hit
                                },
                                "last_maintenance": metrics.last_maintenance
                            });
                            CliResponse::success("database-vector-metrics", data)
                        }
                        Ok(None) => CliResponse::error(
                            "database-vector-metrics",
                            "Vector store not configured",
                        ),
                        Err(e) => CliResponse::error(
                            "database-vector-metrics",
                            format!("Failed to get vector metrics: {}", e),
                        ),
                    }
                }
                #[cfg(not(feature = "vector-search"))]
                {
                    CliResponse::error(
                        "database-vector-metrics",
                        "Vector search feature not enabled",
                    )
                }
            }
            DatabaseAction::VectorMaintenance => {
                #[cfg(feature = "vector-search")]
                {
                    match storage.perform_vector_maintenance().await {
                        Ok(()) => {
                            let data = json!({
                                "message": "Vector store maintenance completed successfully",
                                "operations": ["VACUUM ANALYZE", "REINDEX"],
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            });
                            CliResponse::success("database-vector-maintenance", data)
                        }
                        Err(e) => CliResponse::error(
                            "database-vector-maintenance",
                            format!("Failed to perform maintenance: {}", e),
                        ),
                    }
                }
                #[cfg(not(feature = "vector-search"))]
                {
                    CliResponse::error(
                        "database-vector-maintenance",
                        "Vector search feature not enabled",
                    )
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
            SearchAction::Semantic {
                query,
                chat_id,
                limit,
            } => {
                #[cfg(feature = "vector-search")]
                {
                    match storage
                        .search_similar_events(query, chat_id.as_deref(), *limit)
                        .await
                    {
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
                        Err(e) => CliResponse::error(
                            "search-semantic",
                            format!("Semantic search failed: {}", e),
                        ),
                    }
                }
                #[cfg(not(feature = "vector-search"))]
                {
                    let _ = (query, chat_id, limit); // Avoid unused variable warnings
                    CliResponse::error("search-semantic", "Vector search feature not enabled")
                }
            }
            SearchAction::Text {
                query,
                chat_id,
                limit,
            } => {
                match storage
                    .search_messages(query, chat_id.as_deref(), *limit)
                    .await
                {
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
            SearchAction::Advanced {
                user_id,
                event_type,
                since,
                until,
                limit,
            } => {
                // Parse date filters
                let since_date = match since.as_ref().map(|s| parse_datetime(s)) {
                    Some(Ok(date)) => Some(date),
                    Some(Err(_)) => {
                        return CliResponse::error(
                            "search-advanced",
                            "Invalid 'since' date format. Use ISO 8601 format (e.g., 2023-01-01T00:00:00Z)",
                        );
                    }
                    None => None,
                };

                let until_date = match until.as_ref().map(|s| parse_datetime(s)) {
                    Some(Ok(date)) => Some(date),
                    Some(Err(_)) => {
                        return CliResponse::error(
                            "search-advanced",
                            "Invalid 'until' date format. Use ISO 8601 format (e.g., 2023-01-01T00:00:00Z)",
                        );
                    }
                    None => None,
                };

                match storage
                    .search_events_advanced(
                        user_id.as_deref(),
                        event_type.as_deref(),
                        since_date,
                        until_date,
                        *limit,
                    )
                    .await
                {
                    Ok(events) => {
                        let data = json!({
                            "filters": {
                                "user_id": user_id,
                                "event_type": event_type,
                                "since": since,
                                "until": until,
                                "limit": limit
                            },
                            "results_count": events.len(),
                            "events": events.into_iter().map(|e| json!({
                                "id": e.id,
                                "event_id": e.event_id,
                                "event_type": e.event_type,
                                "chat_id": e.chat_id,
                                "user_id": e.user_id,
                                "timestamp": e.timestamp,
                                "processed_data": e.processed_data
                            })).collect::<Vec<_>>()
                        });
                        CliResponse::success("search-advanced", data)
                    }
                    Err(e) => CliResponse::error(
                        "search-advanced",
                        format!("Advanced search failed: {}", e),
                    ),
                }
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
            ContextAction::Get {
                chat_id,
                context_type: _,
                timeframe: _,
            } => {
                let default_chat_id =
                    std::env::var("VKTEAMS_BOT_CHAT_ID").unwrap_or_else(|_| "default".to_string());
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
                    Err(e) => {
                        CliResponse::error("context-get", format!("Failed to get context: {}", e))
                    }
                }
            }
            ContextAction::Create {
                chat_id,
                summary,
                context_type,
            } => {
                // Create a context document based on the provided summary
                let context_id = uuid::Uuid::new_v4().to_string();

                // Store context as a vector document if vector search is enabled
                #[cfg(feature = "vector-search")]
                {
                    use std::collections::HashMap;
                    use vkteams_bot::storage::VectorDocument;

                    let mut metadata_map = HashMap::new();
                    metadata_map.insert(
                        "chat_id".to_string(),
                        serde_json::Value::String(chat_id.clone()),
                    );
                    metadata_map.insert(
                        "context_type".to_string(),
                        serde_json::Value::String(format!("{:?}", context_type)),
                    );
                    metadata_map.insert(
                        "created_at".to_string(),
                        serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
                    );

                    let metadata =
                        serde_json::to_value(metadata_map).unwrap_or(serde_json::Value::Null);

                    // Get dimensions from storage configuration
                    let dimensions = storage.get_embedding_dimensions();
                    
                    let document = VectorDocument {
                        id: context_id.clone(),
                        content: summary.clone(),
                        metadata,
                        embedding: pgvector::Vector::from(vec![0.0; dimensions]), // Placeholder embedding with correct dimensions
                        created_at: chrono::Utc::now(),
                    };

                    match storage.store_vector_document(&document).await {
                        Ok(_) => {
                            let data = json!({
                                "context_id": context_id,
                                "chat_id": chat_id,
                                "summary": summary,
                                "context_type": format!("{:?}", context_type),
                                "created_at": chrono::Utc::now().to_rfc3339(),
                                "status": "created"
                            });
                            CliResponse::success("context-create", data)
                        }
                        Err(e) => CliResponse::error(
                            "context-create",
                            format!("Failed to create context: {}", e),
                        ),
                    }
                }

                #[cfg(not(feature = "vector-search"))]
                {
                    let _ = (chat_id, summary, context_type); // Avoid unused variable warnings
                    CliResponse::error(
                        "context-create",
                        "Vector search feature not enabled. Context creation requires vector storage.",
                    )
                }
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

// Helper function to parse datetime strings
fn parse_datetime(
    date_str: &str,
) -> std::result::Result<chrono::DateTime<chrono::Utc>, &'static str> {
    use chrono::{DateTime, TimeZone};

    // Try different formats
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        return Ok(dt.with_timezone(&chrono::Utc));
    }

    // Try ISO format without timezone
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S") {
        return Ok(chrono::Utc.from_utc_datetime(&dt));
    }

    // Try date only
    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        if let Some(datetime) = date.and_hms_opt(0, 0, 0) {
            return Ok(chrono::Utc.from_utc_datetime(&datetime));
        }
    }

    Err("Invalid date format")
}

#[cfg(test)]
mod storage_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_commands_name() {
        let cmd = StorageCommands::Database {
            action: DatabaseAction::Init,
        };
        assert_eq!(cmd.name(), "database");

        let cmd = StorageCommands::Search {
            action: SearchAction::Text {
                query: "test".to_string(),
                chat_id: None,
                limit: 10,
            },
        };
        assert_eq!(cmd.name(), "search");
    }

    #[test]
    fn test_context_type_enum() {
        let context_type = ContextType::Recent;
        assert!(matches!(context_type, ContextType::Recent));
    }

    #[test]
    fn test_parse_datetime() {
        // Test RFC3339 format
        assert!(parse_datetime("2023-01-01T00:00:00Z").is_ok());

        // Test ISO format without timezone
        assert!(parse_datetime("2023-01-01T00:00:00").is_ok());

        // Test date only
        assert!(parse_datetime("2023-01-01").is_ok());

        // Test invalid format
        assert!(parse_datetime("invalid-date").is_err());
    }

    #[test]
    fn test_context_action_variants() {
        // Test that ContextAction variants are defined correctly
        let get_action = ContextAction::Get {
            chat_id: Some("test_chat".to_string()),
            context_type: ContextType::Recent,
            timeframe: None,
        };

        let create_action = ContextAction::Create {
            chat_id: "test_chat".to_string(),
            summary: "Test summary".to_string(),
            context_type: "recent".to_string(),
        };

        // These should match without errors
        match get_action {
            ContextAction::Get { .. } => {}
            _ => panic!("Expected ContextAction::Get"),
        }

        match create_action {
            ContextAction::Create { .. } => {}
            _ => panic!("Expected ContextAction::Create"),
        }
    }

    #[test]
    fn test_database_action_variants() {
        // Test DatabaseAction variants
        let init_action = DatabaseAction::Init;
        let stats_action = DatabaseAction::Stats {
            chat_id: Some("test_chat".to_string()),
            since: Some("2023-01-01".to_string()),
        };
        let cleanup_action = DatabaseAction::Cleanup {
            older_than_days: 30,
        };
        let vector_metrics_action = DatabaseAction::VectorMetrics;
        let vector_maintenance_action = DatabaseAction::VectorMaintenance;

        match init_action {
            DatabaseAction::Init => {}
            _ => panic!("Expected DatabaseAction::Init"),
        }

        match stats_action {
            DatabaseAction::Stats { .. } => {}
            _ => panic!("Expected DatabaseAction::Stats"),
        }

        match cleanup_action {
            DatabaseAction::Cleanup { older_than_days } => {
                assert_eq!(older_than_days, 30);
            }
            _ => panic!("Expected DatabaseAction::Cleanup"),
        }

        match vector_metrics_action {
            DatabaseAction::VectorMetrics => {}
            _ => panic!("Expected DatabaseAction::VectorMetrics"),
        }

        match vector_maintenance_action {
            DatabaseAction::VectorMaintenance => {}
            _ => panic!("Expected DatabaseAction::VectorMaintenance"),
        }
    }

    #[test]
    fn test_search_action_variants() {
        // Test SearchAction variants
        let semantic_action = SearchAction::Semantic {
            query: "test query".to_string(),
            chat_id: Some("test_chat".to_string()),
            limit: 5,
        };

        let text_action = SearchAction::Text {
            query: "search text".to_string(),
            chat_id: None,
            limit: 20,
        };

        let advanced_action = SearchAction::Advanced {
            user_id: Some("user123".to_string()),
            event_type: Some("NewMessage".to_string()),
            since: Some("2023-01-01".to_string()),
            until: Some("2023-12-31".to_string()),
            limit: 50,
        };

        match semantic_action {
            SearchAction::Semantic { query, chat_id, limit } => {
                assert_eq!(query, "test query");
                assert_eq!(chat_id, Some("test_chat".to_string()));
                assert_eq!(limit, 5);
            }
            _ => panic!("Expected SearchAction::Semantic"),
        }

        match text_action {
            SearchAction::Text { query, chat_id, limit } => {
                assert_eq!(query, "search text");
                assert_eq!(chat_id, None);
                assert_eq!(limit, 20);
            }
            _ => panic!("Expected SearchAction::Text"),
        }

        match advanced_action {
            SearchAction::Advanced { user_id, event_type, since, until, limit } => {
                assert_eq!(user_id, Some("user123".to_string()));
                assert_eq!(event_type, Some("NewMessage".to_string()));
                assert_eq!(since, Some("2023-01-01".to_string()));
                assert_eq!(until, Some("2023-12-31".to_string()));
                assert_eq!(limit, 50);
            }
            _ => panic!("Expected SearchAction::Advanced"),
        }
    }

    #[test]
    fn test_context_type_enum_values() {
        // Test all ContextType variants
        let recent = ContextType::Recent;
        let topic = ContextType::Topic;
        let user_profile = ContextType::UserProfile;

        match recent {
            ContextType::Recent => {}
            _ => panic!("Expected ContextType::Recent"),
        }

        match topic {
            ContextType::Topic => {}
            _ => panic!("Expected ContextType::Topic"),
        }

        match user_profile {
            ContextType::UserProfile => {}
            _ => panic!("Expected ContextType::UserProfile"),
        }
    }

    #[test]
    fn test_storage_commands_validation() {
        // Test that storage commands validate correctly
        let database_cmd = StorageCommands::Database {
            action: DatabaseAction::Init,
        };
        assert!(database_cmd.validate().is_ok());

        let search_cmd = StorageCommands::Search {
            action: SearchAction::Text {
                query: "test".to_string(),
                chat_id: None,
                limit: 10,
            },
        };
        assert!(search_cmd.validate().is_ok());

        let context_cmd = StorageCommands::Context {
            action: ContextAction::Get {
                chat_id: Some("test".to_string()),
                context_type: ContextType::Recent,
                timeframe: None,
            },
        };
        assert!(context_cmd.validate().is_ok());
    }

    #[test]
    fn test_storage_commands_debug_and_clone() {
        let cmd = StorageCommands::Database {
            action: DatabaseAction::Init,
        };

        // Test Debug trait
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Database"));
        assert!(debug_str.contains("Init"));

        // Test Clone trait
        let cloned_cmd = cmd.clone();
        assert_eq!(cloned_cmd.name(), cmd.name());
    }

    #[test]
    fn test_parse_datetime_edge_cases() {
        // Test more date format edge cases
        assert!(parse_datetime("2023-12-31T23:59:59Z").is_ok());
        assert!(parse_datetime("2023-01-01T00:00:00+00:00").is_ok());
        assert!(parse_datetime("2023-06-15T12:30:45").is_ok());
        assert!(parse_datetime("2023-02-28").is_ok());
        assert!(parse_datetime("2024-02-29").is_ok()); // Leap year
        
        // Invalid formats
        assert!(parse_datetime("").is_err());
        assert!(parse_datetime("not-a-date").is_err());
        assert!(parse_datetime("2023-13-01").is_err()); // Invalid month
        assert!(parse_datetime("2023-02-30").is_err()); // Invalid day
    }

    #[tokio::test]
    async fn test_storage_config_fallback() {
        let storage_cmd = StorageCommands::Database {
            action: DatabaseAction::Init,
        };

        // Test fallback configuration loading
        let config_result = storage_cmd.load_storage_config().await;
        assert!(config_result.is_ok());

        let config = config_result.unwrap();
        assert_eq!(config.database.max_connections, 20);
        assert_eq!(config.database.connection_timeout, 30);
        assert!(config.database.auto_migrate);
        assert_eq!(config.settings.event_retention_days, 365);
        assert_eq!(config.settings.cleanup_interval_hours, 24);
        assert_eq!(config.settings.batch_size, 100);
        assert_eq!(config.settings.max_memory_events, 10000);
    }

    #[test]
    fn test_context_action_get_with_different_types() {
        let recent_action = ContextAction::Get {
            chat_id: Some("chat1".to_string()),
            context_type: ContextType::Recent,
            timeframe: Some("1d".to_string()),
        };

        let topic_action = ContextAction::Get {
            chat_id: Some("chat2".to_string()),
            context_type: ContextType::Topic,
            timeframe: Some("1w".to_string()),
        };

        let user_profile_action = ContextAction::Get {
            chat_id: Some("chat3".to_string()),
            context_type: ContextType::UserProfile,
            timeframe: None,
        };

        match recent_action {
            ContextAction::Get { context_type: ContextType::Recent, .. } => {}
            _ => panic!("Expected Recent context type"),
        }

        match topic_action {
            ContextAction::Get { context_type: ContextType::Topic, .. } => {}
            _ => panic!("Expected Topic context type"),
        }

        match user_profile_action {
            ContextAction::Get { context_type: ContextType::UserProfile, .. } => {}
            _ => panic!("Expected UserProfile context type"),
        }
    }

    #[test]
    fn test_storage_commands_name_variants() {
        let database_cmd = StorageCommands::Database {
            action: DatabaseAction::Stats {
                chat_id: None,
                since: None,
            },
        };
        assert_eq!(database_cmd.name(), "database");

        let search_cmd = StorageCommands::Search {
            action: SearchAction::Semantic {
                query: "test".to_string(),
                chat_id: None,
                limit: 10,
            },
        };
        assert_eq!(search_cmd.name(), "search");

        let context_cmd = StorageCommands::Context {
            action: ContextAction::Create {
                chat_id: "test".to_string(),
                summary: "summary".to_string(),
                context_type: "topic".to_string(),
            },
        };
        assert_eq!(context_cmd.name(), "context");
    }
}
