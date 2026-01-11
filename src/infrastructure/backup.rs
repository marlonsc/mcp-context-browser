//! Backup Manager for creating and restoring backups
//!
//! Uses tar and flate2 for standard, portable stream-based compression.
//! Listens to BackupCreate events from the Event Bus.

use crate::domain::error::{Error, Result};
use crate::infrastructure::events::{SharedEventBus, SystemEvent};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::path::Path;
use std::sync::Arc;
use tar::Builder;
use tokio::sync::mpsc;

/// Messages for the backup actor
pub enum BackupMessage {
    CreateBackup {
        path: String,
        response: tokio::sync::oneshot::Sender<Result<BackupInfo>>,
    },
    ListBackups {
        response: tokio::sync::oneshot::Sender<Result<Vec<BackupInfo>>>,
    },
}

/// Information about a backup
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub path: String,
    pub size_bytes: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Backup Manager using Actor pattern (no locks)
#[derive(Clone)]
pub struct BackupManager {
    sender: mpsc::Sender<BackupMessage>,
    backup_dir: String,
    data_dir: String,
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new(backup_dir: &str, data_dir: &str, event_bus: Option<SharedEventBus>) -> Self {
        let (tx, rx) = mpsc::channel(100);
        let backup_dir_owned = backup_dir.to_string();
        let data_dir_owned = data_dir.to_string();

        // Start the actor
        let mut actor = BackupActor::new(rx, backup_dir_owned.clone(), data_dir_owned.clone());
        tokio::spawn(async move {
            actor.run().await;
        });

        let manager = Self {
            sender: tx,
            backup_dir: backup_dir_owned,
            data_dir: data_dir_owned,
        };

        // Start event listener if event bus provided
        if let Some(bus) = event_bus {
            manager.start_event_listener(bus);
        }

        manager
    }

    /// Start listening for backup events
    fn start_event_listener(&self, event_bus: SharedEventBus) {
        let mut receiver = event_bus.subscribe();
        let manager = self.clone();

        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                if let SystemEvent::BackupCreate { path } = event {
                    tracing::info!("[BACKUP] Creating backup at: {}", path);
                    match manager.create_backup_sync(&path) {
                        Ok(info) => {
                            tracing::info!(
                                "[BACKUP] Created backup: {} ({} bytes)",
                                info.path,
                                info.size_bytes
                            );
                        }
                        Err(e) => {
                            tracing::error!("[BACKUP] Failed to create backup at {}: {}", path, e);
                        }
                    }
                }
            }
        });
    }

    /// Create a backup synchronously (for event handler)
    fn create_backup_sync(&self, target_path: &str) -> Result<BackupInfo> {
        // Ensure backup directory exists
        let backup_dir = Path::new(&self.backup_dir);
        if !backup_dir.exists() {
            fs::create_dir_all(backup_dir)
                .map_err(|e| Error::internal(format!("Failed to create backup dir: {}", e)))?;
        }

        // Create the tar.gz file
        let file = File::create(target_path)
            .map_err(|e| Error::internal(format!("Failed to create backup file: {}", e)))?;

        let encoder = GzEncoder::new(file, Compression::default());
        let mut archive = Builder::new(encoder);

        // Add the data directory to the archive
        let data_path = Path::new(&self.data_dir);
        if data_path.exists() && data_path.is_dir() {
            archive
                .append_dir_all("data", data_path)
                .map_err(|e| Error::internal(format!("Failed to add data to backup: {}", e)))?;
        }

        // Finish writing the archive
        let encoder = archive
            .into_inner()
            .map_err(|e| Error::internal(format!("Failed to finalize archive: {}", e)))?;
        encoder
            .finish()
            .map_err(|e| Error::internal(format!("Failed to finish compression: {}", e)))?;

        // Get file size
        let metadata = fs::metadata(target_path)
            .map_err(|e| Error::internal(format!("Failed to get backup metadata: {}", e)))?;

        Ok(BackupInfo {
            path: target_path.to_string(),
            size_bytes: metadata.len(),
            created_at: chrono::Utc::now(),
        })
    }

    /// Create a backup asynchronously via actor
    pub async fn create_backup(&self, path: &str) -> Result<BackupInfo> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender
            .send(BackupMessage::CreateBackup {
                path: path.to_string(),
                response: tx,
            })
            .await
            .map_err(|_| Error::internal("Backup actor closed"))?;

        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Backup response channel closed")))
    }

    /// List all backups
    pub async fn list_backups(&self) -> Result<Vec<BackupInfo>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender
            .send(BackupMessage::ListBackups { response: tx })
            .await
            .map_err(|_| Error::internal("Backup actor closed"))?;

        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Backup response channel closed")))
    }

    /// Get the backup directory path
    pub fn backup_dir(&self) -> &str {
        &self.backup_dir
    }
}

/// Actor that handles backup operations
struct BackupActor {
    receiver: mpsc::Receiver<BackupMessage>,
    backup_dir: String,
    data_dir: String,
}

impl BackupActor {
    fn new(receiver: mpsc::Receiver<BackupMessage>, backup_dir: String, data_dir: String) -> Self {
        Self {
            receiver,
            backup_dir,
            data_dir,
        }
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                BackupMessage::CreateBackup { path, response } => {
                    let result = self.do_create_backup(&path);
                    let _ = response.send(result);
                }
                BackupMessage::ListBackups { response } => {
                    let result = self.do_list_backups();
                    let _ = response.send(result);
                }
            }
        }
    }

    fn do_create_backup(&self, target_path: &str) -> Result<BackupInfo> {
        // Ensure backup directory exists
        let backup_dir = Path::new(&self.backup_dir);
        if !backup_dir.exists() {
            fs::create_dir_all(backup_dir)
                .map_err(|e| Error::internal(format!("Failed to create backup dir: {}", e)))?;
        }

        // Create the tar.gz file
        let file = File::create(target_path)
            .map_err(|e| Error::internal(format!("Failed to create backup file: {}", e)))?;

        let encoder = GzEncoder::new(file, Compression::default());
        let mut archive = Builder::new(encoder);

        // Add the data directory to the archive
        let data_path = Path::new(&self.data_dir);
        if data_path.exists() && data_path.is_dir() {
            archive
                .append_dir_all("data", data_path)
                .map_err(|e| Error::internal(format!("Failed to add data to backup: {}", e)))?;
        }

        // Finish writing the archive
        let encoder = archive
            .into_inner()
            .map_err(|e| Error::internal(format!("Failed to finalize archive: {}", e)))?;
        encoder
            .finish()
            .map_err(|e| Error::internal(format!("Failed to finish compression: {}", e)))?;

        // Get file size
        let metadata = fs::metadata(target_path)
            .map_err(|e| Error::internal(format!("Failed to get backup metadata: {}", e)))?;

        Ok(BackupInfo {
            path: target_path.to_string(),
            size_bytes: metadata.len(),
            created_at: chrono::Utc::now(),
        })
    }

    fn do_list_backups(&self) -> Result<Vec<BackupInfo>> {
        let backup_dir = Path::new(&self.backup_dir);
        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        let entries = fs::read_dir(backup_dir)
            .map_err(|e| Error::internal(format!("Failed to read backup dir: {}", e)))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("gz") {
                if let Ok(metadata) = fs::metadata(&path) {
                    let created_at = metadata
                        .created()
                        .ok()
                        .map(chrono::DateTime::<chrono::Utc>::from)
                        .unwrap_or_else(chrono::Utc::now);

                    backups.push(BackupInfo {
                        path: path.to_string_lossy().to_string(),
                        size_bytes: metadata.len(),
                        created_at,
                    });
                }
            }
        }

        Ok(backups)
    }
}

/// Shared backup manager type
pub type SharedBackupManager = Arc<BackupManager>;

/// Create a shared backup manager
pub fn create_shared_backup_manager(
    backup_dir: &str,
    data_dir: &str,
    event_bus: Option<SharedEventBus>,
) -> SharedBackupManager {
    Arc::new(BackupManager::new(backup_dir, data_dir, event_bus))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_backup_creation() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let backup_dir = TempDir::new()?;
        let data_dir = TempDir::new()?;

        // Create some test data
        let test_file = data_dir.path().join("test.txt");
        fs::write(&test_file, "test data")?;

        let manager = BackupManager::new(
            backup_dir.path().to_str().ok_or("Invalid backup dir path")?,
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
            backup_dir.path().to_str().ok_or("Invalid backup dir path")?,
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
}
