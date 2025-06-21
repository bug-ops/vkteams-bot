//! Integration tests for storage functionality using testcontainers

#[cfg(all(test, feature = "storage-full"))]
mod tests {
    use std::time::Duration;
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::postgres::Postgres;
    use vkteams_bot::storage::{
        config::{DatabaseConfig, StorageSettings},
        StorageConfig, StorageManager,
    };

    #[cfg(feature = "vector-search")]
    use vkteams_bot::storage::vector::{SearchQuery, VectorDocument};

    /// Setup a test PostgreSQL container with pgvector extension
    async fn setup_postgres_container() -> testcontainers::ContainerAsync<Postgres> {
        let postgres_container = Postgres::default()
            .start()
            .await
            .expect("Failed to start PostgreSQL container");

        // Additional wait for PostgreSQL to be fully ready
        tokio::time::sleep(Duration::from_secs(2)).await;

        postgres_container
    }

    /// Create storage configuration for tests
    fn create_test_storage_config(database_url: String) -> StorageConfig {
        StorageConfig {
            database: DatabaseConfig {
                url: database_url.clone(),
                max_connections: 5,
                connection_timeout: 10,
                auto_migrate: true,
            },
            #[cfg(feature = "vector-search")]
            vector: None, // Disable vector search for CI tests since pgvector extension may not be available
            #[cfg(feature = "ai-embeddings")]
            embedding: None, // Skip embeddings for integration tests
            settings: StorageSettings {
                event_retention_days: 7,
                cleanup_interval_hours: 1,
                batch_size: 10,
                max_memory_events: 100,
            },
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_storage_manager_initialization() {
        let container = setup_postgres_container().await;
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let database_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        let config = create_test_storage_config(database_url);
        let storage_manager = StorageManager::new(&config).await;

        assert!(storage_manager.is_ok(), "Failed to create storage manager");

        let manager = storage_manager.unwrap();
        
        // Test initialization
        let init_result = manager.initialize().await;
        assert!(init_result.is_ok(), "Failed to initialize storage");

        // Test health check
        let health_result = manager.health_check().await;
        assert!(health_result.is_ok(), "Storage health check failed");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_storage_stats() {
        let container = setup_postgres_container().await;
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let database_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        let config = create_test_storage_config(database_url);
        let storage_manager = StorageManager::new(&config).await.unwrap();
        storage_manager.initialize().await.unwrap();

        // Get initial stats
        let stats = storage_manager.get_stats(None).await;
        assert!(stats.is_ok(), "Failed to get storage stats");

        let stats = stats.unwrap();
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.total_messages, 0);
        assert_eq!(stats.unique_chats, 0);
        assert_eq!(stats.unique_users, 0);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_message_search() {
        let container = setup_postgres_container().await;
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let database_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        let config = create_test_storage_config(database_url);
        let storage_manager = StorageManager::new(&config).await.unwrap();
        storage_manager.initialize().await.unwrap();

        // Test empty search
        let search_result = storage_manager.search_messages("test query", None, 10).await;
        assert!(search_result.is_ok(), "Failed to search messages");

        let messages = search_result.unwrap();
        assert_eq!(messages.len(), 0, "Expected no messages in empty database");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_recent_events() {
        let container = setup_postgres_container().await;
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let database_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        let config = create_test_storage_config(database_url);
        let storage_manager = StorageManager::new(&config).await.unwrap();
        storage_manager.initialize().await.unwrap();

        // Test recent events with empty database
        let events_result = storage_manager.get_recent_events(None, None, 10).await;
        assert!(events_result.is_ok(), "Failed to get recent events");

        let events = events_result.unwrap();
        assert_eq!(events.len(), 0, "Expected no events in empty database");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_cleanup_old_data() {
        let container = setup_postgres_container().await;
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let database_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        let config = create_test_storage_config(database_url);
        let storage_manager = StorageManager::new(&config).await.unwrap();
        storage_manager.initialize().await.unwrap();

        // Test cleanup with empty database
        let cleanup_result = storage_manager.cleanup_old_data(30).await;
        assert!(cleanup_result.is_ok(), "Failed to cleanup old data");

        let deleted_count = cleanup_result.unwrap();
        assert_eq!(deleted_count, 0, "Expected no records to delete in empty database");
    }

    #[cfg(all(feature = "vector-search", feature = "pgvector-tests"))]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_vector_storage_initialization() {
        let container = setup_postgres_container().await;
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let database_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        // Test vector store creation
        let vector_store = vkteams_bot::storage::vector::create_vector_store(
            "pgvector",
            &database_url,
            Some("test_vectors".to_string()),
        ).await;

        assert!(vector_store.is_ok(), "Failed to create vector store");

        let store = vector_store.unwrap();
        
        // Test health check
        let health_result = store.health_check().await;
        assert!(health_result.is_ok(), "Vector store health check failed");
    }

    #[cfg(all(feature = "vector-search", feature = "pgvector-tests"))]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_vector_document_operations() {
        use chrono::Utc;
        use serde_json::json;

        let container = setup_postgres_container().await;
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let database_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        let vector_store = vkteams_bot::storage::vector::create_vector_store(
            "pgvector",
            &database_url,
            Some("test_docs".to_string()),
        ).await.unwrap();

        // Create test document
        let test_doc = VectorDocument {
            id: "test_doc_1".to_string(),
            content: "This is a test document for vector search".to_string(),
            metadata: json!({
                "source": "test",
                "category": "example"
            }),
            embedding: pgvector::Vector::from(vec![0.1; 128]), // 128 dimensions
            created_at: Utc::now(),
        };

        // Test storing document
        let store_result = vector_store.store_document(test_doc.clone()).await;
        assert!(store_result.is_ok(), "Failed to store vector document");

        // Test retrieving document
        let get_result = vector_store.get_document("test_doc_1").await;
        assert!(get_result.is_ok(), "Failed to get vector document");

        let retrieved_doc = get_result.unwrap();
        assert!(retrieved_doc.is_some(), "Document not found");

        let doc = retrieved_doc.unwrap();
        assert_eq!(doc.id, "test_doc_1");
        assert_eq!(doc.content, "This is a test document for vector search");

        // Test similarity search
        let search_query = SearchQuery {
            embedding: pgvector::Vector::from(vec![0.1; 128]),
            limit: 5,
            score_threshold: Some(0.5),
            metadata_filter: None,
            include_content: true,
        };

        let search_result = vector_store.search_similar(search_query).await;
        assert!(search_result.is_ok(), "Failed to perform similarity search");

        let results = search_result.unwrap();
        assert!(!results.is_empty(), "Expected to find similar documents");
        assert_eq!(results[0].id, "test_doc_1");

        // Test deleting document
        let delete_result = vector_store.delete_document("test_doc_1").await;
        assert!(delete_result.is_ok(), "Failed to delete vector document");

        let deleted = delete_result.unwrap();
        assert!(deleted, "Document should have been deleted");

        // Verify document was deleted
        let get_after_delete = vector_store.get_document("test_doc_1").await;
        assert!(get_after_delete.is_ok(), "Failed to check deleted document");
        assert!(get_after_delete.unwrap().is_none(), "Document should not exist after deletion");
    }

    #[cfg(all(feature = "vector-search", feature = "pgvector-tests"))]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_vector_batch_operations() {
        use chrono::Utc;
        use serde_json::json;

        let container = setup_postgres_container().await;
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let database_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        let vector_store = vkteams_bot::storage::vector::create_vector_store(
            "pgvector",
            &database_url,
            Some("test_batch".to_string()),
        ).await.unwrap();

        // Create multiple test documents
        let docs: Vec<VectorDocument> = (0..5).map(|i| {
            VectorDocument {
                id: format!("batch_doc_{}", i),
                content: format!("Batch test document number {}", i),
                metadata: json!({
                    "batch": true,
                    "index": i
                }),
                embedding: pgvector::Vector::from(vec![0.1 + (i as f32 * 0.1); 128]),
                created_at: Utc::now(),
            }
        }).collect();

        // Test batch storing
        let batch_store_result = vector_store.store_documents(docs).await;
        assert!(batch_store_result.is_ok(), "Failed to store documents in batch");

        // Test searching with metadata filter
        let search_query = SearchQuery {
            embedding: pgvector::Vector::from(vec![0.2; 128]),
            limit: 10,
            score_threshold: Some(0.0),
            metadata_filter: Some(json!({"batch": true})),
            include_content: true,
        };

        let search_result = vector_store.search_similar(search_query).await;
        assert!(search_result.is_ok(), "Failed to perform batch search");

        let results = search_result.unwrap();
        assert_eq!(results.len(), 5, "Expected to find all batch documents");

        // Test cleanup
        let cleanup_date = Utc::now() + chrono::Duration::minutes(1);
        let cleanup_result = vector_store.cleanup_old_documents(cleanup_date).await;
        assert!(cleanup_result.is_ok(), "Failed to cleanup documents");

        let deleted_count = cleanup_result.unwrap();
        assert_eq!(deleted_count, 5, "Expected to delete all batch documents");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_storage_config_validation() {
        let container = setup_postgres_container().await;
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        
        // Test invalid database URL
        let invalid_config = StorageConfig {
            database: DatabaseConfig {
                url: "invalid://url".to_string(),
                max_connections: 5,
                connection_timeout: 10,
                auto_migrate: true,
            },
            #[cfg(feature = "vector-search")]
            vector: None,
            #[cfg(feature = "ai-embeddings")]
            embedding: None,
            settings: StorageSettings {
                event_retention_days: 7,
                cleanup_interval_hours: 1,
                batch_size: 10,
                max_memory_events: 100,
            },
        };

        let storage_result = StorageManager::new(&invalid_config).await;
        assert!(storage_result.is_err(), "Expected error for invalid database URL");

        // Test valid configuration
        let valid_database_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            host_port
        );
        let valid_config = create_test_storage_config(valid_database_url);

        let storage_result = StorageManager::new(&valid_config).await;
        assert!(storage_result.is_ok(), "Valid configuration should work");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_concurrent_storage_operations() {
        let container = setup_postgres_container().await;
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let database_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        let config = create_test_storage_config(database_url);
        let storage_manager = StorageManager::new(&config).await.unwrap();
        storage_manager.initialize().await.unwrap();

        // Run multiple operations concurrently
        let (stats_result, search_result, events_result) = tokio::join!(
            storage_manager.get_stats(None),
            storage_manager.search_messages("test", None, 10),
            storage_manager.get_recent_events(None, None, 10)
        );

        assert!(stats_result.is_ok(), "Stats operation failed");
        assert!(search_result.is_ok(), "Search operation failed");
        assert!(events_result.is_ok(), "Events operation failed");
    }
}