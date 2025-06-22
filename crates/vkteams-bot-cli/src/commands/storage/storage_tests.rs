//! Tests for storage CLI commands

#[cfg(test)]
mod tests {
    use crate::commands::storage::*;
    use crate::output::CliResponse;
    use serde_json::{Value, json};

    #[test]
    fn test_database_action_variants() {
        let init_action = DatabaseAction::Init;
        let stats_action = DatabaseAction::Stats {
            chat_id: Some("test_chat".to_string()),
            since: None,
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
            DatabaseAction::Stats { chat_id, since: _ } => {
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
            since: None,
            until: None,
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
                since: _,
                until: _,
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

    #[test]
    fn test_context_action_variants() {
        let get_action = ContextAction::Get {
            chat_id: Some("test_chat".to_string()),
            context_type: ContextType::Recent,
            timeframe: None,
        };

        let create_action = ContextAction::Create {
            chat_id: "new_chat".to_string(),
            summary: "Test summary".to_string(),
            context_type: "conversation".to_string(),
        };

        match get_action {
            ContextAction::Get {
                chat_id,
                context_type,
                timeframe: _,
            } => {
                assert_eq!(chat_id, Some("test_chat".to_string()));
                assert!(matches!(context_type, ContextType::Recent));
            }
            _ => panic!("Expected Get variant"),
        }

        match create_action {
            ContextAction::Create {
                chat_id,
                summary,
                context_type,
            } => {
                assert_eq!(chat_id, "new_chat");
                assert_eq!(summary, "Test summary");
                assert_eq!(context_type, "conversation");
            }
            _ => panic!("Expected Create variant"),
        }
    }

    #[test]
    fn test_cli_response_creation() {
        let success_response = CliResponse::success("test-command", json!({"result": "ok"}));
        let error_response: CliResponse<Value> =
            CliResponse::error("test-command", "Something went wrong".to_string());

        assert!(success_response.success);
        assert_eq!(success_response.command, "test-command");
        assert!(success_response.data.is_some());
        assert!(success_response.error.is_none());

        assert!(!error_response.success);
        assert_eq!(error_response.command, "test-command");
        assert!(error_response.data.is_none());
        assert!(error_response.error.is_some());
        assert_eq!(error_response.error.unwrap(), "Something went wrong");
    }

    #[tokio::test]
    async fn test_load_storage_config() {
        let storage_commands = StorageCommands::Database {
            action: DatabaseAction::Init,
        };

        // Test loading configuration (should work without database)
        let result = storage_commands.load_storage_config().await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(!config.database.url.is_empty());
        assert_eq!(config.database.max_connections, 20);
        assert_eq!(config.settings.event_retention_days, 365);
    }
}
