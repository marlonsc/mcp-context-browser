//! Configuration Types Tests
//!
//! Note: ConfigKey, ConfigProfile, and ValidationResult types were removed
//! during the configuration refactoring. These tests are retained as a
//! placeholder for future configuration type testing.

use mcb_infrastructure::config::data::{ServerConfig, ServerNetworkConfig, ServerSslConfig};

#[test]
fn test_server_config_defaults() {
    let config = ServerConfig::default();

    // Network defaults
    assert!(!config.network.host.is_empty());
    assert!(config.network.port > 0);

    // SSL defaults to disabled
    assert!(!config.ssl.https);
    assert!(config.ssl.ssl_cert_path.is_none());
    assert!(config.ssl.ssl_key_path.is_none());

    // CORS defaults
    assert!(config.cors.cors_enabled);
    assert!(!config.cors.cors_origins.is_empty());
}

#[test]
fn test_server_network_config() {
    let network = ServerNetworkConfig {
        host: "0.0.0.0".to_string(),
        port: 9000,
        admin_port: 9001,
    };

    assert_eq!(network.host, "0.0.0.0");
    assert_eq!(network.port, 9000);
    assert_eq!(network.admin_port, 9001);
}

#[test]
fn test_server_ssl_config() {
    let ssl = ServerSslConfig {
        https: true,
        ssl_cert_path: Some(std::path::PathBuf::from("/path/to/cert.pem")),
        ssl_key_path: Some(std::path::PathBuf::from("/path/to/key.pem")),
    };

    assert!(ssl.https);
    assert!(ssl.ssl_cert_path.is_some());
    assert!(ssl.ssl_key_path.is_some());
}
