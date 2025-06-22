//! SSL/TLS configuration tests

#[cfg(test)]
mod tests {
    use crate::storage::config::{DatabaseConfig, SslConfig, StorageConfig};
    
    #[test]
    fn test_ssl_config_default() {
        let ssl_config = SslConfig::default();
        assert!(!ssl_config.enabled);
        assert_eq!(ssl_config.mode, "prefer");
        assert!(ssl_config.root_cert.is_none());
        assert!(ssl_config.client_cert.is_none());
        assert!(ssl_config.client_key.is_none());
    }
    
    #[test]
    fn test_database_config_with_ssl() {
        let db_config = DatabaseConfig {
            url: "postgresql://localhost/test".to_string(),
            max_connections: 10,
            connection_timeout: 30,
            auto_migrate: true,
            ssl: SslConfig {
                enabled: true,
                mode: "require".to_string(),
                root_cert: Some("/path/to/ca.pem".to_string()),
                client_cert: None,
                client_key: None,
            },
        };
        
        assert!(db_config.ssl.enabled);
        assert_eq!(db_config.ssl.mode, "require");
        assert_eq!(db_config.ssl.root_cert, Some("/path/to/ca.pem".to_string()));
    }
    
    #[test]
    fn test_storage_config_includes_ssl() {
        let config = StorageConfig::default();
        assert!(!config.database.ssl.enabled);
        assert_eq!(config.database.ssl.mode, "prefer");
    }
    
    #[test]
    fn test_ssl_config_serialization() {
        let ssl_config = SslConfig {
            enabled: true,
            mode: "verify-full".to_string(),
            root_cert: Some("/etc/ssl/ca.pem".to_string()),
            client_cert: Some("/etc/ssl/client.pem".to_string()),
            client_key: Some("/etc/ssl/client.key".to_string()),
        };
        
        let serialized = serde_json::to_string(&ssl_config).unwrap();
        let deserialized: SslConfig = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(ssl_config, deserialized);
    }
    
    #[test]
    fn test_ssl_mode_values() {
        let valid_modes = vec!["disable", "prefer", "require", "verify-ca", "verify-full"];
        
        for mode in valid_modes {
            let ssl_config = SslConfig {
                enabled: true,
                mode: mode.to_string(),
                root_cert: None,
                client_cert: None,
                client_key: None,
            };
            
            // Just verify the mode is accepted
            assert_eq!(ssl_config.mode, mode);
        }
    }
}