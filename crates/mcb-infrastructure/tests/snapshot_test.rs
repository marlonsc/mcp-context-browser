//! Tests for snapshot infrastructure

use mcb_application::ports::infrastructure::SnapshotProvider;
use mcb_infrastructure::infrastructure::snapshot::NullSnapshotProvider;
use std::path::Path;

#[test]
fn test_null_snapshot_provider_creation() {
    let provider = NullSnapshotProvider::new();
    // Test that provider can be created without panicking
    let _provider: Box<dyn SnapshotProvider> = Box::new(provider);
}

#[tokio::test]
async fn test_null_snapshot_provider_create_snapshot() {
    let provider = NullSnapshotProvider::new();

    // Null implementation creates an empty snapshot
    let result = provider.create_snapshot(Path::new("/test/project")).await;
    assert!(result.is_ok());
    let snapshot = result.unwrap();
    assert_eq!(snapshot.id, "null-snapshot");
    assert_eq!(snapshot.total_files, 0);
}

#[tokio::test]
async fn test_null_snapshot_provider_load_snapshot() {
    let provider = NullSnapshotProvider::new();

    // Null implementation always returns None
    let result = provider.load_snapshot(Path::new("/test/project")).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_null_snapshot_provider_get_changed_files() {
    let provider = NullSnapshotProvider::new();

    // Null implementation returns empty vec
    let result = provider.get_changed_files(Path::new("/test/project")).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_null_snapshot_provider_new() {
    let provider = NullSnapshotProvider::new();
    // Test that new implementation can be created
    assert!(std::any::type_name_of_val(&provider).contains("NullSnapshotProvider"));
}
