//! Configuration Loader Tests

use mcb_infrastructure::config::loader::{ConfigBuilder, ConfigLoader};
use mcb_infrastructure::constants::{DEFAULT_HTTP_PORT, DEFAULT_LOG_LEVEL};
use tempfile::TempDir;

#[test]
fn test_config_loader_default() {
    let loader = ConfigLoader::new();
    let config = loader.load().unwrap();

    assert_eq!(config.server.port, DEFAULT_HTTP_PORT);
    assert_eq!(config.logging.level, DEFAULT_LOG_LEVEL);
}

#[test]
fn test_config_builder() {
    let config = ConfigBuilder::new()
        .with_server(mcb_infrastructure::config::data::ServerConfig {
            port: 9090,
            ..Default::default()
        })
        .build();

    assert_eq!(config.server.port, 9090);
}

// Note: validate_config is private, so we test the public API instead
#[test]
fn test_config_loader_exists() {
    // Test that ConfigLoader type exists and can be created
    let _ = ConfigLoader::new();
}

#[test]
fn test_config_save_load() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");

    let loader = ConfigLoader::new();
    let original_config = ConfigBuilder::new()
        .with_server(mcb_infrastructure::config::data::ServerConfig {
            port: 9999,
            ..Default::default()
        })
        .build();

    // Save config
    loader.save_to_file(&original_config, &config_path).unwrap();

    // Load config
    let loaded_config = ConfigLoader::new()
        .with_config_path(&config_path)
        .load()
        .unwrap();

    assert_eq!(loaded_config.server.port, 9999);
}
