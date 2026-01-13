//! Respawn manager tests
//!
//! Tests migrated from src/infrastructure/respawn.rs

use mcp_context_browser::infrastructure::respawn::{
    can_respawn, get_binary_path, RespawnConfig, RespawnManager,
};

#[test]
fn test_respawn_config_defaults() {
    let config = RespawnConfig::default();
    assert!(config.binary_path.is_none());
    assert!(config.use_exec);
    assert_eq!(config.restart_exit_code, 71);
}

#[test]
fn test_respawn_manager_creation() -> Result<(), Box<dyn std::error::Error>> {
    // This test may fail in some environments without /proc
    if can_respawn() {
        let manager = RespawnManager::with_defaults()?;
        let path_str = manager.binary_path().to_string_lossy();
        assert!(manager.binary_path().exists() || path_str.contains("target"));
    }
    Ok(())
}

#[test]
fn test_can_respawn() {
    // On Linux with /proc, this should work
    // On other platforms or containers without /proc, it may not
    let result = can_respawn();
    // Just verify it doesn't panic
    let _ = result;
}

#[test]
fn test_get_binary_path() -> Result<(), Box<dyn std::error::Error>> {
    if can_respawn() {
        let path = get_binary_path()?;
        // The path should exist or at least be a valid path
        assert!(!path.to_string_lossy().is_empty());
    }
    Ok(())
}
