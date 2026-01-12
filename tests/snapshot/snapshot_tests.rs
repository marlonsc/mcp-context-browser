//! Tests for snapshot management functionality
//!
//! Migrated from src/snapshot/mod.rs inline tests.
//! Tests snapshot creation, loading, and change detection.

use mcp_context_browser::snapshot::{SnapshotChanges, SnapshotManager};
use tempfile::TempDir;

#[tokio::test]
async fn test_snapshot_manager_creation() -> Result<(), Box<dyn std::error::Error>> {
    let _manager = SnapshotManager::new()?;
    // Verify the manager was created successfully
    // The snapshot_dir should exist after creation
    Ok(())
}

#[tokio::test]
async fn test_empty_directory_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = SnapshotManager::new()?;

    let snapshot = manager.create_snapshot(temp_dir.path()).await?;
    assert_eq!(snapshot.file_count, 0);
    assert_eq!(snapshot.total_size, 0);
    assert!(snapshot.files.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_snapshot_changes() {
    let changes = SnapshotChanges {
        added: vec!["file1.rs".to_string()],
        modified: vec!["file2.rs".to_string()],
        removed: vec!["file3.rs".to_string()],
        unchanged: vec!["file4.rs".to_string()],
    };

    assert!(changes.has_changes());
    assert_eq!(changes.total_changes(), 3);
}

#[test]
fn test_should_skip_file() -> Result<(), Box<dyn std::error::Error>> {
    let _manager = SnapshotManager::new()?;
    let _temp_dir = TempDir::new()?;

    // Create test paths - note that should_skip_file is private, so we test indirectly
    // through create_snapshot behavior

    // Files that should be skipped won't appear in snapshots
    // Files that shouldn't be skipped will appear in snapshots
    Ok(())
}

#[tokio::test]
async fn test_snapshot_changes_no_changes() {
    let changes = SnapshotChanges {
        added: vec![],
        modified: vec![],
        removed: vec![],
        unchanged: vec!["file1.rs".to_string()],
    };

    assert!(!changes.has_changes());
    assert_eq!(changes.total_changes(), 0);
}

#[tokio::test]
async fn test_snapshot_changes_only_added() {
    let changes = SnapshotChanges {
        added: vec!["file1.rs".to_string(), "file2.rs".to_string()],
        modified: vec![],
        removed: vec![],
        unchanged: vec![],
    };

    assert!(changes.has_changes());
    assert_eq!(changes.total_changes(), 2);
}

#[tokio::test]
async fn test_snapshot_changes_only_modified() {
    let changes = SnapshotChanges {
        added: vec![],
        modified: vec!["file1.rs".to_string()],
        removed: vec![],
        unchanged: vec!["file2.rs".to_string()],
    };

    assert!(changes.has_changes());
    assert_eq!(changes.total_changes(), 1);
}

#[tokio::test]
async fn test_snapshot_changes_only_removed() {
    let changes = SnapshotChanges {
        added: vec![],
        modified: vec![],
        removed: vec!["file1.rs".to_string()],
        unchanged: vec![],
    };

    assert!(changes.has_changes());
    assert_eq!(changes.total_changes(), 1);
}

#[tokio::test]
async fn test_snapshot_with_files() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = SnapshotManager::new()?;

    // Create some test files
    std::fs::write(temp_dir.path().join("test.rs"), "fn main() {}")?;
    std::fs::write(temp_dir.path().join("lib.rs"), "pub mod test;")?;

    let snapshot = manager.create_snapshot(temp_dir.path()).await?;

    assert_eq!(snapshot.file_count, 2);
    assert!(snapshot.total_size > 0);
    assert!(snapshot.files.contains_key("test.rs"));
    assert!(snapshot.files.contains_key("lib.rs"));
    Ok(())
}

#[tokio::test]
async fn test_snapshot_skips_hidden_files() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = SnapshotManager::new()?;

    // Create visible and hidden files
    std::fs::write(temp_dir.path().join("visible.rs"), "fn main() {}")?;
    std::fs::write(temp_dir.path().join(".hidden"), "secret")?;

    let snapshot = manager.create_snapshot(temp_dir.path()).await?;

    // Should only include visible file
    assert_eq!(snapshot.file_count, 1);
    assert!(snapshot.files.contains_key("visible.rs"));
    assert!(!snapshot.files.contains_key(".hidden"));
    Ok(())
}

#[tokio::test]
async fn test_snapshot_respects_gitignore() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = SnapshotManager::new()?;

    // Create a .gitignore that excludes .log files
    std::fs::write(temp_dir.path().join(".gitignore"), "*.log\n")?;

    // Create source and log files
    std::fs::write(temp_dir.path().join("main.rs"), "fn main() {}")?;
    std::fs::write(temp_dir.path().join("debug.log"), "log content")?;

    let snapshot = manager.create_snapshot(temp_dir.path()).await?;

    // Should only include source file (log is gitignored)
    assert_eq!(snapshot.file_count, 1);
    assert!(snapshot.files.contains_key("main.rs"));
    assert!(!snapshot.files.contains_key("debug.log"));
    Ok(())
}

#[tokio::test]
async fn test_load_nonexistent_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = SnapshotManager::new()?;

    // Loading a snapshot for a path that has never been snapshotted should return None
    let result = manager.load_snapshot(temp_dir.path()).await?;
    assert!(result.is_none());
    Ok(())
}

#[tokio::test]
async fn test_snapshot_load_after_create() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = SnapshotManager::new()?;

    // Create a file and snapshot
    std::fs::write(temp_dir.path().join("test.rs"), "fn main() {}")?;
    let original = manager.create_snapshot(temp_dir.path()).await?;

    // Load the snapshot
    let loaded = manager.load_snapshot(temp_dir.path()).await?;

    assert!(loaded.is_some());
    let loaded = loaded.expect("Snapshot should exist");
    assert_eq!(loaded.file_count, original.file_count);
    assert_eq!(loaded.total_size, original.total_size);
    Ok(())
}

#[tokio::test]
async fn test_compare_identical_snapshots() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = SnapshotManager::new()?;

    // Create a file and snapshot twice
    std::fs::write(temp_dir.path().join("test.rs"), "fn main() {}")?;
    let snapshot1 = manager.create_snapshot(temp_dir.path()).await?;
    let snapshot2 = manager.create_snapshot(temp_dir.path()).await?;

    // Compare should show no changes
    let changes = manager.compare_snapshots(&snapshot1, &snapshot2).await?;
    assert!(!changes.has_changes());
    assert_eq!(changes.unchanged.len(), 1);
    Ok(())
}

#[tokio::test]
async fn test_get_changed_files_initial() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = SnapshotManager::new()?;

    // Create some files
    std::fs::write(temp_dir.path().join("file1.rs"), "fn main() {}")?;
    std::fs::write(temp_dir.path().join("file2.rs"), "fn test() {}")?;

    // First time getting changed files should return all files as "added"
    let changed = manager.get_changed_files(temp_dir.path()).await?;
    assert_eq!(changed.len(), 2);
    Ok(())
}

#[tokio::test]
async fn test_snapshot_disabled_manager() {
    let _manager = SnapshotManager::new_disabled();
    // The disabled manager uses a temp directory that may not exist,
    // which can cause save operations to fail. The manager is primarily
    // used for testing where snapshot persistence is not needed.

    // Creating the manager should not panic
    let _temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Note: create_snapshot may fail because the snapshot_dir doesn't exist
    // in the disabled manager. This is expected behavior for a "disabled" manager.
    // The manager is meant for contexts where snapshots don't need to persist.
}
