//! Binary watcher tests
//!
//! Tests migrated from src/infrastructure/binary_watcher.rs

use mcp_context_browser::infrastructure::binary_watcher::BinaryWatcherConfig;
use mcp_context_browser::infrastructure::events::EventBus;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_binary_watcher_config_defaults() {
    let config = BinaryWatcherConfig::default();
    assert!(config.binary_path.is_none());
    assert_eq!(config.debounce_duration, Duration::from_secs(3));
    assert!(config.auto_respawn);
}

#[tokio::test]
async fn test_binary_watcher_creation() -> Result<(), Box<dyn std::error::Error>> {
    use mcp_context_browser::infrastructure::binary_watcher::BinaryWatcher;

    let event_bus = Arc::new(EventBus::default());
    let dir = tempdir()?;
    let binary_path = dir.path().join("test_binary");
    std::fs::write(&binary_path, "test")?;

    let config = BinaryWatcherConfig {
        binary_path: Some(binary_path.clone()),
        ..Default::default()
    };

    let watcher = BinaryWatcher::new(event_bus, config)?;
    assert_eq!(watcher.binary_path(), &binary_path);
    assert!(!watcher.is_running());
    Ok(())
}
