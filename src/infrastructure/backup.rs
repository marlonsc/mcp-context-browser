//! Backup Manager for creating and restoring backups
//!
//! Uses tar and flate2 for standard, portable stream-based compression.
//! Listens to BackupCreate events from the Event Bus.

use crate::domain::error::{Error, Result};
use crate::infrastructure::events::{SharedEventBusProvider, SystemEvent};
use crate::infrastructure::ErrorContext;
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
    pub fn new(
        backup_dir: &str,
        data_dir: &str,
        event_bus: Option<SharedEventBusProvider>,
    ) -> Self {
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
    fn start_event_listener(&self, event_bus: SharedEventBusProvider) {
        let manager = self.clone();

        tokio::spawn(async move {
            if let Ok(mut receiver) = event_bus.subscribe().await {
                while let Ok(event) = receiver.recv().await {
                    if let SystemEvent::BackupCreate { path } = event {
                        tracing::info!("[BACKUP] Creating backup at: {}", path);
                        let m = manager.clone();
                        let p = path.clone();
                        // Run backup in a blocking task to avoid stalling the async runtime
                        let result =
                            tokio::task::spawn_blocking(move || m.create_backup_internal(&p)).await;

                        match result {
                            Ok(Ok(info)) => {
                                tracing::info!(
                                    "[BACKUP] Created backup: {} ({} bytes)",
                                    info.path,
                                    info.size_bytes
                                );
                            }
                            Ok(Err(e)) => {
                                tracing::error!(
                                    "[BACKUP] Failed to create backup at {}: {}",
                                    path,
                                    e
                                );
                            }
                            Err(e) => {
                                tracing::error!("[BACKUP] Backup task panicked: {}", e);
                            }
                        }
                    }
                }
            }
        });
    }

    /// Internal backup creation logic (synchronous)
    fn create_backup_internal(&self, target_path: &str) -> Result<BackupInfo> {
        BackupActor::perform_backup(target_path, &self.backup_dir, &self.data_dir)
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
        let backup_dir = self.backup_dir.clone();
        let data_dir = self.data_dir.clone();

        while let Some(msg) = self.receiver.recv().await {
            match msg {
                BackupMessage::CreateBackup { path, response } => {
                    let b_dir = backup_dir.clone();
                    let d_dir = data_dir.clone();
                    let p = path.clone();

                    // Run the actual backup in a blocking task
                    let result = tokio::task::spawn_blocking(move || {
                        Self::perform_backup(&p, &b_dir, &d_dir)
                    })
                    .await
                    .map_err(|_| Error::internal("Backup task panicked"))
                    .and_then(|res| res);

                    let _ = response.send(result);
                }
                BackupMessage::ListBackups { response } => {
                    let b_dir = backup_dir.clone();
                    let result =
                        tokio::task::spawn_blocking(move || Self::perform_list_backups(&b_dir))
                            .await
                            .map_err(|_| Error::internal("List backups task panicked"))
                            .and_then(|res| res);

                    let _ = response.send(result);
                }
            }
        }
    }

    fn perform_backup(
        target_path: &str,
        backup_dir_path: &str,
        data_dir_path: &str,
    ) -> Result<BackupInfo> {
        // Ensure backup directory exists
        let backup_dir = Path::new(backup_dir_path);
        if !backup_dir.exists() {
            fs::create_dir_all(backup_dir)
                .internal_context("Failed to create backup dir")?;
        }

        // Create the tar.gz file
        let file = File::create(target_path)
            .internal_context("Failed to create backup file")?;

        let encoder = GzEncoder::new(file, Compression::default());
        let mut archive = Builder::new(encoder);

        // Add the data directory to the archive
        let data_path = Path::new(data_dir_path);
        if data_path.exists() && data_path.is_dir() {
            archive
                .append_dir_all("data", data_path)
                .internal_context("Failed to add data to backup")?;
        }

        // Finish writing the archive
        let encoder = archive
            .into_inner()
            .internal_context("Failed to finalize archive")?;
        encoder
            .finish()
            .internal_context("Failed to finish compression")?;

        // Get file size
        let metadata = fs::metadata(target_path)
            .internal_context("Failed to get backup metadata")?;

        Ok(BackupInfo {
            path: target_path.to_string(),
            size_bytes: metadata.len(),
            created_at: chrono::Utc::now(),
        })
    }

    fn perform_list_backups(backup_dir_path: &str) -> Result<Vec<BackupInfo>> {
        let backup_dir = Path::new(backup_dir_path);
        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        let entries = fs::read_dir(backup_dir)
            .internal_context("Failed to read backup dir")?;

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
    event_bus: Option<SharedEventBusProvider>,
) -> SharedBackupManager {
    Arc::new(BackupManager::new(backup_dir, data_dir, event_bus))
}
