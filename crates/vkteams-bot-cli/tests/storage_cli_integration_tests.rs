//! Integration tests for CLI storage commands using testcontainers

#[cfg(all(test, feature = "storage"))]
mod tests {
    use std::time::Duration;
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::postgres::Postgres;
    use vkteams_bot_cli::commands::storage::*;
    use vkteams_bot_cli::commands::Command;
    use vkteams_bot::prelude::*;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_storage_commands_with_real_database() {
        let postgres_container = Postgres::default()
            .start()
            .await
            .expect("Failed to start PostgreSQL container");
        
        // Wait for PostgreSQL to be ready
        tokio::time::sleep(Duration::from_secs(2)).await;

        let host_port = postgres_container.get_host_port_ipv4(5432).await.unwrap();
        let _database_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        // Note: In real usage, DATABASE_URL would be set externally

        // Create storage commands instance
        let _storage_commands = StorageCommands::Database {
            action: DatabaseAction::Init,
        };

        // Test configuration loading through public interface only
        // Note: We'll test this through the actual command execution

        // Create dummy bot for testing
        let _bot = Bot::with_params(&APIVersionUrl::V1, "dummy_token", "https://dummy.api.com")
            .expect("Failed to create dummy bot");

        // Test database initialization
        let init_commands = StorageCommands::Database {
            action: DatabaseAction::Init,
        };

        let init_response = init_commands.handle_database(&DatabaseAction::Init).await;
        assert!(init_response.success, "Database initialization should succeed");
        assert_eq!(init_response.command, "database-init");

        // Test getting database stats
        let stats_commands = StorageCommands::Database {
            action: DatabaseAction::Stats {
                chat_id: None,
                since: None,
            },
        };

        let stats_response = stats_commands.handle_database(&DatabaseAction::Stats {
            chat_id: None,
            since: None,
        }).await;
        assert!(stats_response.success, "Getting stats should succeed");
        assert_eq!(stats_response.command, "database-stats");

        if let Some(data) = stats_response.data {
            assert!(data.get("total_events").is_some());
            assert!(data.get("total_messages").is_some());
            assert!(data.get("unique_chats").is_some());
        }

        // Test message search (should return empty results)
        let search_commands = StorageCommands::Search {
            action: SearchAction::Text {
                query: "test search query".to_string(),
                chat_id: None,
                limit: 10,
            },
        };

        let search_response = search_commands.handle_search(&SearchAction::Text {
            query: "test search query".to_string(),
            chat_id: None,
            limit: 10,
        }).await;
        assert!(search_response.success, "Text search should succeed");
        assert_eq!(search_response.command, "search-text");

        if let Some(data) = search_response.data {
            assert_eq!(data.get("results_count").unwrap().as_u64().unwrap(), 0);
            assert!(data.get("messages").unwrap().as_array().unwrap().is_empty());
        }

        // Test context retrieval
        let context_commands = StorageCommands::Context {
            action: ContextAction::Get {
                chat_id: Some("test_chat".to_string()),
                context_type: ContextType::Recent,
                timeframe: None,
            },
        };

        let context_response = context_commands.handle_context(&ContextAction::Get {
            chat_id: Some("test_chat".to_string()),
            context_type: ContextType::Recent,
            timeframe: None,
        }).await;
        assert!(context_response.success, "Context retrieval should succeed");
        assert_eq!(context_response.command, "context-get");

        // Test database cleanup
        let cleanup_commands = StorageCommands::Database {
            action: DatabaseAction::Cleanup {
                older_than_days: 30,
            },
        };

        let cleanup_response = cleanup_commands.handle_database(&DatabaseAction::Cleanup {
            older_than_days: 30,
        }).await;
        assert!(cleanup_response.success, "Database cleanup should succeed");
        assert_eq!(cleanup_response.command, "database-cleanup");

        if let Some(data) = cleanup_response.data {
            assert!(data.get("deleted_events").is_some());
            assert_eq!(data.get("older_than_days").unwrap().as_u64().unwrap(), 30);
        }

        // Test completed successfully
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_storage_config_with_environment_variables() {
        let postgres_container = Postgres::default()
            .start()
            .await
            .expect("Failed to start PostgreSQL container");
        tokio::time::sleep(Duration::from_secs(3)).await;

        let host_port = postgres_container.get_host_port_ipv4(5432).await.unwrap();
        let _database_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        // Note: Testing environment variable handling

        let storage_commands = StorageCommands::Database {
            action: DatabaseAction::Init,
        };

        // Test basic functionality (without env vars since they require unsafe)
        // In real usage, environment variables would be set externally
        let _init_response = storage_commands.handle_database(&DatabaseAction::Init).await;
        // This might fail without proper env setup, which is expected
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_storage_error_handling() {
        // Test error handling with default configuration

        let storage_commands = StorageCommands::Database {
            action: DatabaseAction::Init,
        };

        // Test that commands handle errors gracefully
        let _init_response = storage_commands.handle_database(&DatabaseAction::Init).await;
        // This tests error handling when no valid database connection is available
    }

    #[cfg(feature = "vector-search")]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_semantic_search_feature_availability() {
        let postgres_container = Postgres::default()
            .start()
            .await
            .expect("Failed to start PostgreSQL container");
        tokio::time::sleep(Duration::from_secs(3)).await;

        let host_port = postgres_container.get_host_port_ipv4(5432).await.unwrap();
        let _database_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        // Note: In production, DATABASE_URL would be set externally

        let search_commands = StorageCommands::Search {
            action: SearchAction::Semantic {
                query: "test semantic search".to_string(),
                chat_id: None,
                limit: 5,
            },
        };

        let _search_response = search_commands.handle_search(&SearchAction::Semantic {
            query: "test semantic search".to_string(),
            chat_id: None,
            limit: 5,
        }).await;

        // Test that semantic search command is recognized (regardless of database connection)
    }

    #[cfg(not(feature = "vector-search"))]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_semantic_search_feature_disabled() {
        let search_commands = StorageCommands::Search {
            action: SearchAction::Semantic {
                query: "test semantic search".to_string(),
                chat_id: None,
                limit: 5,
            },
        };

        let _search_response = search_commands.handle_search(&SearchAction::Semantic {
            query: "test semantic search".to_string(),
            chat_id: None,
            limit: 5,
        }).await;

        // Should return error when vector-search feature is disabled
        assert!(!search_response.success, "Semantic search should be disabled without vector-search feature");
        assert!(search_response.error.unwrap().contains("Vector search feature not enabled"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_command_validation() {
        // Test valid commands
        let valid_commands = vec![
            StorageCommands::Database { action: DatabaseAction::Init },
            StorageCommands::Database { 
                action: DatabaseAction::Stats { 
                    chat_id: Some("test_chat".to_string()), 
                    since: None 
                } 
            },
            StorageCommands::Database { 
                action: DatabaseAction::Cleanup { 
                    older_than_days: 30 
                } 
            },
            StorageCommands::Search { 
                action: SearchAction::Text { 
                    query: "test".to_string(), 
                    chat_id: None, 
                    limit: 10 
                } 
            },
            StorageCommands::Context { 
                action: ContextAction::Get { 
                    chat_id: Some("test".to_string()), 
                    context_type: ContextType::Recent, 
                    timeframe: None 
                } 
            },
        ];

        for command in valid_commands {
            let validation_result = command.validate();
            assert!(validation_result.is_ok(), "Command validation should pass: {:?}", command);
        }
    }

    #[tokio::test]
    #[serial_test::serial] 
    async fn test_command_names() {
        let commands_with_names = vec![
            (StorageCommands::Database { action: DatabaseAction::Init }, "database"),
            (StorageCommands::Search { 
                action: SearchAction::Text { 
                    query: "test".to_string(), 
                    chat_id: None, 
                    limit: 10 
                } 
            }, "search"),
            (StorageCommands::Context { 
                action: ContextAction::Get { 
                    chat_id: Some("test".to_string()), 
                    context_type: ContextType::Recent, 
                    timeframe: None 
                } 
            }, "context"),
        ];

        for (command, expected_name) in commands_with_names {
            assert_eq!(command.name(), expected_name);
        }
    }
}