//! Tests for snapshot infrastructure

use mcb_application::ports::infrastructure::SnapshotProvider;
use mcb_infrastructure::infrastructure::NullSnapshotProvider;
use std::path::Path;

#[test]
fn test_null_snapshot_provider_creation() {
    let provider = NullSnapshotProvider::new();
    let _provider: Box<dyn SnapshotProvider> = Box::new(provider);
}

#[tokio::test]
async fn test_null_snapshot_provider_create_snapshot() {
    let provider = NullSnapshotProvider::new();
    let result = provider.create_snapshot(Path::new("/test/project")).await;
    assert!(result.is_ok());
    let snapshot = result.unwrap();
    assert_eq!(snapshot.id, "null-snapshot");
    assert_eq!(snapshot.total_files, 0);
}

#[tokio::test]
async fn test_null_snapshot_provider_load_snapshot() {
    let provider = NullSnapshotProvider::new();
    let result = provider.load_snapshot(Path::new("/test/project")).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_null_snapshot_provider_get_changed_files() {
    let provider = NullSnapshotProvider::new();
    let result = provider.get_changed_files(Path::new("/test/project")).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}
