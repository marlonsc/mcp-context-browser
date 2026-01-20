//! Tests for sync infrastructure

use mcb_application::ports::infrastructure::SyncProvider;
use mcb_infrastructure::infrastructure::NullSyncProvider;
use std::path::Path;
use std::time::Duration;

#[test]
fn test_null_sync_provider_creation() {
    let provider = NullSyncProvider::new();
    let _provider: Box<dyn SyncProvider> = Box::new(provider);
}

#[tokio::test]
async fn test_null_sync_provider_should_debounce() {
    let provider = NullSyncProvider::new();
    let result = provider.should_debounce(Path::new("/test")).await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_null_sync_provider_acquire_sync_slot() {
    let provider = NullSyncProvider::new();
    let result = provider.acquire_sync_slot(Path::new("/test")).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_null_sync_provider_get_changed_files() {
    let provider = NullSyncProvider::new();
    let result = provider.get_changed_files(Path::new("/test")).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_null_sync_provider_intervals() {
    let provider = NullSyncProvider::new();
    assert_eq!(provider.sync_interval(), Duration::from_secs(60));
    assert_eq!(provider.debounce_interval(), Duration::from_secs(5));
}
