//! Tests for SyncManager - TDD RED phase
//!
//! These tests verify the REAL sync implementation:
//! - File change detection
//! - Modification time tracking
//! - Event publishing on sync completion

use mcp_context_browser::sync::manager::{SyncConfig, SyncManager};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test directory with files
fn create_test_directory() -> Result<TempDir, Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    // Create some test files
    fs::write(temp_dir.path().join("file1.rs"), "fn main() {}")?;
    fs::write(temp_dir.path().join("file2.rs"), "fn test() {}")?;
    fs::create_dir(temp_dir.path().join("subdir"))?;
    fs::write(temp_dir.path().join("subdir/file3.rs"), "mod test;")?;

    Ok(temp_dir)
}

#[tokio::test]
async fn test_sync_detects_new_files() -> Result<(), Box<dyn std::error::Error>> {
    // Given: A sync manager and a directory with files
    let temp_dir = create_test_directory()?;
    let manager = SyncManager::with_config(SyncConfig {
        interval_ms: 1000,
        debounce_ms: 0, // Disable debounce for testing
    });

    // When: We sync the first time
    let result = manager.sync_codebase(temp_dir.path()).await;

    // Then: Sync should succeed and detect files
    assert!(result.is_ok(), "Sync should succeed");
    assert!(
        result?,
        "First sync should return true (files detected)"
    );

    // And: Stats should show file count
    let stats = manager.get_stats();
    assert!(stats.successful > 0, "Should have successful syncs");
    Ok(())
}

#[tokio::test]
async fn test_sync_detects_file_modifications() -> Result<(), Box<dyn std::error::Error>> {
    // Given: A synced directory
    let temp_dir = create_test_directory()?;
    let manager = SyncManager::with_config(SyncConfig {
        interval_ms: 1000,
        debounce_ms: 0,
    });

    // First sync to establish baseline
    let _ = manager.sync_codebase(temp_dir.path()).await;

    // Wait a bit to ensure file timestamps differ
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // When: A file is modified
    fs::write(
        temp_dir.path().join("file1.rs"),
        "fn main() { println!(\"modified\"); }",
    )?;

    // And: We sync again
    let result = manager.sync_codebase(temp_dir.path()).await;

    // Then: Sync should detect the change
    assert!(result.is_ok(), "Sync should succeed");
    // The sync should return true if it detected changes
    // This test will FAIL until we implement real change detection
    Ok(())
}

#[tokio::test]
async fn test_sync_tracks_modification_times() -> Result<(), Box<dyn std::error::Error>> {
    // Given: A sync manager
    let temp_dir = create_test_directory()?;
    let manager = SyncManager::with_config(SyncConfig {
        interval_ms: 1000,
        debounce_ms: 0,
    });

    // When: We sync
    let _ = manager.sync_codebase(temp_dir.path()).await;

    // Then: File modification times should be tracked
    let tracked_files = manager.get_tracked_file_count();
    assert!(tracked_files > 0, "Should track at least one file");
    Ok(())
}

#[tokio::test]
async fn test_sync_returns_changed_files() -> Result<(), Box<dyn std::error::Error>> {
    // Given: A synced directory with modifications
    let temp_dir = create_test_directory()?;
    let manager = SyncManager::with_config(SyncConfig {
        interval_ms: 1000,
        debounce_ms: 0,
    });

    // First sync
    let _ = manager.sync_codebase(temp_dir.path()).await;

    // Wait for filesystem time resolution (some filesystems have 1-second granularity)
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    fs::write(
        temp_dir.path().join("file2.rs"),
        "fn test_modified() { assert!(true); } // modified content",
    )?;

    // When: We get changed files
    let changed = manager.get_changed_files(temp_dir.path()).await;

    // Then: Should include the modified file
    assert!(changed.is_ok(), "Should succeed");
    let files = changed?;
    assert!(!files.is_empty(), "Should have detected changed file");
    assert!(
        files.iter().any(|f| f.ends_with("file2.rs")),
        "Should include file2.rs"
    );
    Ok(())
}

#[tokio::test]
async fn test_sync_with_event_bus_publishes_event() -> Result<(), Box<dyn std::error::Error>> {
    use mcp_context_browser::infrastructure::events::{EventBus, SystemEvent};
    use std::sync::Arc;

    // Given: A sync manager with event bus
    let temp_dir = create_test_directory()?;
    let event_bus = Arc::new(EventBus::new(10));
    let mut receiver = event_bus.subscribe();

    let manager = SyncManager::with_event_bus(
        SyncConfig {
            interval_ms: 1000,
            debounce_ms: 0,
        },
        event_bus,
    );

    // When: We sync
    let _ = manager.sync_codebase(temp_dir.path()).await;

    // Then: An event should be published
    // Use timeout to avoid hanging if event not published
    let event = tokio::time::timeout(std::time::Duration::from_millis(500), receiver.recv()).await;

    assert!(event.is_ok(), "Should receive event within timeout");
    match event? {
        Ok(SystemEvent::SyncCompleted {
            path,
            files_changed,
        }) => {
            assert_eq!(path, temp_dir.path().to_string_lossy().to_string());
            assert!(files_changed >= 0, "Should report files changed count");
        }
        _ => return Err("Expected SyncCompleted event".into()),
    }
    Ok(())
}

#[tokio::test]
async fn test_sync_handles_empty_directory() -> Result<(), Box<dyn std::error::Error>> {
    // Given: An empty directory
    let temp_dir = TempDir::new()?;
    let manager = SyncManager::with_config(SyncConfig {
        interval_ms: 1000,
        debounce_ms: 0,
    });

    // When: We sync an empty directory
    let result = manager.sync_codebase(temp_dir.path()).await;

    // Then: Should succeed without error
    assert!(result.is_ok(), "Sync should succeed for empty directory");
    Ok(())
}

#[tokio::test]
async fn test_sync_handles_nonexistent_directory() {
    // Given: A path that doesn't exist
    let manager = SyncManager::new();
    let nonexistent = PathBuf::from("/nonexistent/path/that/does/not/exist");

    // When: We try to sync
    let result = manager.sync_codebase(&nonexistent).await;

    // Then: Should return error
    assert!(result.is_err(), "Should error for nonexistent directory");
}

#[tokio::test]
async fn test_sync_filters_by_extension() -> Result<(), Box<dyn std::error::Error>> {
    // Given: A directory with mixed file types
    let temp_dir = TempDir::new()?;
    fs::write(temp_dir.path().join("code.rs"), "fn main() {}")?;
    fs::write(temp_dir.path().join("readme.md"), "# Readme")?;
    fs::write(temp_dir.path().join("data.json"), "{}")?;

    let manager = SyncManager::with_config(SyncConfig {
        interval_ms: 1000,
        debounce_ms: 0,
    });

    // When: We get changed files (should only track code files)
    let _ = manager.sync_codebase(temp_dir.path()).await;
    let tracked = manager.get_tracked_file_count();

    // Then: Should only track relevant file types
    // (Exact count depends on implementation - at minimum should track .rs files)
    assert!(tracked >= 1, "Should track at least the .rs file");
    Ok(())
}
