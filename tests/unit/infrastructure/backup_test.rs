//! Backup manager tests
//!
//! Tests migrated from src/infrastructure/backup.rs

use mcp_context_browser::infrastructure::backup::BackupManager;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[tokio::test]
async fn test_backup_creation() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let backup_dir = TempDir::new()?;
    let data_dir = TempDir::new()?;

    // Create some test data
    let test_file = data_dir.path().join("test.txt");
    fs::write(&test_file, "test data")?;

    let manager = BackupManager::new(
        backup_dir
            .path()
            .to_str()
            .ok_or("Invalid backup dir path")?,
        data_dir.path().to_str().ok_or("Invalid data dir path")?,
        None,
    );

    let backup_path = backup_dir.path().join("test_backup.tar.gz");
    let info = manager
        .create_backup(backup_path.to_str().ok_or("Invalid backup path")?)
        .await?;

    assert!(info.size_bytes > 0);
    assert!(Path::new(&info.path).exists());
    Ok(())
}

#[tokio::test]
async fn test_list_backups() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let backup_dir = TempDir::new()?;
    let data_dir = TempDir::new()?;

    let manager = BackupManager::new(
        backup_dir
            .path()
            .to_str()
            .ok_or("Invalid backup dir path")?,
        data_dir.path().to_str().ok_or("Invalid data dir path")?,
        None,
    );

    // Initially empty
    let backups = manager.list_backups().await?;
    assert!(backups.is_empty());

    // Create a backup
    let backup_path = backup_dir.path().join("test.tar.gz");
    manager
        .create_backup(backup_path.to_str().ok_or("Invalid backup path")?)
        .await?;

    // Should list one backup
    let backups = manager.list_backups().await?;
    assert_eq!(backups.len(), 1);
    Ok(())
}
