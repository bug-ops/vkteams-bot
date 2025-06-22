//! Tests for storage functionality

#[cfg(test)]
mod storage_tests {
    use crate::storage::StorageConfig;
    use crate::storage::config::{DatabaseConfig, StorageSettings};

    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        assert_eq!(config.database.url, "postgresql://localhost/vkteams_bot");
        assert_eq!(config.database.max_connections, 20);
        assert!(config.database.auto_migrate);
        assert_eq!(config.settings.event_retention_days, 365);
        assert_eq!(config.settings.batch_size, 100);
    }

    #[test]
    fn test_storage_config_creation() {
        let config = StorageConfig {
            database: DatabaseConfig {
                url: "postgresql://test/db".to_string(),
                max_connections: 10,
                connection_timeout: 15,
                auto_migrate: false,
            },
            #[cfg(feature = "vector-search")]
            vector: None,
            #[cfg(feature = "ai-embeddings")]
            embedding: None,
            settings: StorageSettings {
                event_retention_days: 30,
                cleanup_interval_hours: 12,
                batch_size: 50,
                max_memory_events: 5000,
            },
        };

        assert_eq!(config.database.url, "postgresql://test/db");
        assert_eq!(config.database.max_connections, 10);
        assert!(!config.database.auto_migrate);
        assert_eq!(config.settings.event_retention_days, 30);
    }

    #[cfg(feature = "vector-search")]
    #[test]
    fn test_vector_document_creation() {
        use crate::storage::vector::VectorDocument;
        use chrono::Utc;
        use serde_json::json;

        let doc = VectorDocument {
            id: "test_doc_1".to_string(),
            content: "This is a test document".to_string(),
            metadata: json!({
                "source": "test",
                "category": "example"
            }),
            embedding: pgvector::Vector::from(vec![0.1, 0.2, 0.3]),
            created_at: Utc::now(),
        };

        assert_eq!(doc.id, "test_doc_1");
        assert_eq!(doc.content, "This is a test document");
        assert_eq!(doc.embedding.as_slice(), &[0.1, 0.2, 0.3]);
    }

    #[cfg(feature = "vector-search")]
    #[test]
    fn test_search_query_creation() {
        use crate::storage::vector::SearchQuery;
        use serde_json::json;

        let query = SearchQuery {
            embedding: pgvector::Vector::from(vec![0.1, 0.2, 0.3]),
            limit: 10,
            score_threshold: Some(0.8),
            metadata_filter: Some(json!({"category": "test"})),
            include_content: true,
        };

        assert_eq!(query.limit, 10);
        assert_eq!(query.score_threshold, Some(0.8));
        assert!(query.include_content);
    }

    #[cfg(feature = "ai-embeddings")]
    #[test]
    fn test_embedding_provider_config() {
        use crate::storage::embedding::EmbeddingProviderConfig;

        let openai_config = EmbeddingProviderConfig::OpenAI {
            api_key: "test_key".to_string(),
            model: "text-embedding-ada-002".to_string(),
        };

        let ollama_config = EmbeddingProviderConfig::Ollama {
            host: "localhost".to_string(),
            port: 11434,
            model: "llama2".to_string(),
            dimensions: Some(1536),
        };

        match openai_config {
            EmbeddingProviderConfig::OpenAI { api_key, model } => {
                assert_eq!(api_key, "test_key");
                assert_eq!(model, "text-embedding-ada-002");
            }
            _ => panic!("Expected OpenAI config"),
        }

        match ollama_config {
            EmbeddingProviderConfig::Ollama {
                host,
                port,
                model,
                dimensions,
            } => {
                assert_eq!(host, "localhost");
                assert_eq!(port, 11434);
                assert_eq!(model, "llama2");
                assert_eq!(dimensions, Some(1536));
            }
            _ => panic!("Expected Ollama config"),
        }
    }

    #[test]
    fn test_storage_error_display() {
        use crate::storage::StorageError;

        let connection_error = StorageError::Connection("Failed to connect".to_string());
        assert_eq!(
            format!("{}", connection_error),
            "Database connection error: Failed to connect"
        );

        let query_error = StorageError::Query("Invalid SQL".to_string());
        assert_eq!(
            format!("{}", query_error),
            "Database query error: Invalid SQL"
        );

        let config_error = StorageError::Configuration("Missing setting".to_string());
        assert_eq!(
            format!("{}", config_error),
            "Configuration error: Missing setting"
        );
    }

    #[test]
    fn test_storage_error_from_io() {
        use crate::storage::StorageError;
        use std::io::{Error, ErrorKind};

        let io_error = Error::new(ErrorKind::NotFound, "File not found");
        let storage_error = StorageError::from(io_error);

        match storage_error {
            StorageError::Io(_) => {} // Expected
            _ => panic!("Expected IO error"),
        }
    }

    #[cfg(feature = "database")]
    #[test]
    fn test_storage_error_from_sqlx() {
        use crate::storage::StorageError;

        let sqlx_error = sqlx::Error::RowNotFound;
        let storage_error = StorageError::from(sqlx_error);

        match storage_error {
            StorageError::NotFound(_) => {} // Expected
            _ => panic!("Expected NotFound error"),
        }
    }

    // Integration tests would go here, but they require a real database
    // For now, we'll focus on unit tests for data structures and configuration
}

#[cfg(test)]
mod cli_tests {
    // Note: CLI tests would normally be in the CLI crate
    // These are simplified storage-related enum tests

    #[derive(Debug, Clone)]
    pub enum DatabaseAction {
        Init,
        Stats {
            chat_id: Option<String>,
            _since: Option<String>,
        },
        Cleanup {
            older_than_days: u32,
        },
    }

    #[derive(Debug, Clone)]
    pub enum SearchAction {
        Semantic {
            query: String,
            chat_id: Option<String>,
            limit: usize,
        },
        Text {
            query: String,
            chat_id: Option<String>,
            limit: i64,
        },
        Advanced {
            user_id: Option<String>,
            event_type: Option<String>,
            _since: Option<String>,
            _until: Option<String>,
            limit: i64,
        },
    }

    #[derive(Debug, Clone)]
    pub enum ContextType {
        Recent,
        Topic,
        UserProfile,
    }

    #[test]
    fn test_database_action_variants() {
        let init_action = DatabaseAction::Init;
        let stats_action = DatabaseAction::Stats {
            chat_id: Some("test_chat".to_string()),
            _since: None,
        };
        let cleanup_action = DatabaseAction::Cleanup {
            older_than_days: 30,
        };

        // Test that all variants can be created
        match init_action {
            DatabaseAction::Init => {}
            _ => panic!("Expected Init variant"),
        }

        match stats_action {
            DatabaseAction::Stats { chat_id, _since: _ } => {
                assert_eq!(chat_id, Some("test_chat".to_string()));
            }
            _ => panic!("Expected Stats variant"),
        }

        match cleanup_action {
            DatabaseAction::Cleanup { older_than_days } => {
                assert_eq!(older_than_days, 30);
            }
            _ => panic!("Expected Cleanup variant"),
        }
    }

    #[test]
    fn test_search_action_variants() {
        let semantic_action = SearchAction::Semantic {
            query: "test query".to_string(),
            chat_id: None,
            limit: 10,
        };

        let text_action = SearchAction::Text {
            query: "search text".to_string(),
            chat_id: Some("chat123".to_string()),
            limit: 20,
        };

        let advanced_action = SearchAction::Advanced {
            user_id: Some("user123".to_string()),
            event_type: Some("message".to_string()),
            _since: None,
            _until: None,
            limit: 15,
        };

        // Test that all variants work correctly
        match semantic_action {
            SearchAction::Semantic {
                query,
                chat_id,
                limit,
            } => {
                assert_eq!(query, "test query");
                assert_eq!(chat_id, None);
                assert_eq!(limit, 10);
            }
            _ => panic!("Expected Semantic variant"),
        }

        match text_action {
            SearchAction::Text {
                query,
                chat_id,
                limit,
            } => {
                assert_eq!(query, "search text");
                assert_eq!(chat_id, Some("chat123".to_string()));
                assert_eq!(limit, 20);
            }
            _ => panic!("Expected Text variant"),
        }

        match advanced_action {
            SearchAction::Advanced {
                user_id,
                event_type,
                _since: _,
                _until: _,
                limit,
            } => {
                assert_eq!(user_id, Some("user123".to_string()));
                assert_eq!(event_type, Some("message".to_string()));
                assert_eq!(limit, 15);
            }
            _ => panic!("Expected Advanced variant"),
        }
    }

    #[test]
    fn test_context_type_enum() {
        let recent = ContextType::Recent;
        let topic = ContextType::Topic;
        let user_profile = ContextType::UserProfile;

        // Test that all variants exist and are distinct
        assert!(matches!(recent, ContextType::Recent));
        assert!(matches!(topic, ContextType::Topic));
        assert!(matches!(user_profile, ContextType::UserProfile));
    }
}
