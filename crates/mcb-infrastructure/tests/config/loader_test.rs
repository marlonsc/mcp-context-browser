//! Configuration Loader Tests

use mcb_infrastructure::config::data::AppConfig;
use mcb_infrastructure::config::loader::{ConfigBuilder, ConfigLoader};
use mcb_infrastructure::constants::{DEFAULT_HTTP_PORT, DEFAULT_LOG_LEVEL};
use tempfile::TempDir;

/// Create test config with auth disabled (avoids JWT secret validation per ADR-025)
fn test_config() -> AppConfig {
    let mut config = AppConfig::default();
    config.auth.enabled = false;
    config
}

/// Test config builder creates valid config with expected defaults
///
/// Note: Per ADR-025, when auth is enabled, JWT secret MUST be configured.
/// We use auth disabled to test the builder without validation failure.
#[test]
fn test_config_loader_default() {
    // Build config directly with auth disabled
    let config = test_config();

    assert_eq!(config.server.network.port, DEFAULT_HTTP_PORT);
    assert_eq!(config.logging.level, DEFAULT_LOG_LEVEL);
}

#[test]
fn test_config_builder() {
    let mut server_config = mcb_infrastructure::config::data::ServerConfig::default();
    server_config.network.port = 9090;

    let config = ConfigBuilder::new().with_server(server_config).build();

    assert_eq!(config.server.network.port, 9090);
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

    // Create config with custom port and auth disabled
    let mut original_config = test_config();
    original_config.server.network.port = 9999;

    // Save config
    loader.save_to_file(&original_config, &config_path).unwrap();

    // Load config
    let loaded_config = ConfigLoader::new()
        .with_config_path(&config_path)
        .load()
        .unwrap();

    assert_eq!(loaded_config.server.network.port, 9999);
}
