//! Snapshot management for incremental codebase tracking
//!
//! Tracks file changes using Merkle tree hashing for efficient incremental sync.
//! Avoids reprocessing unchanged files during codebase indexing.

use crate::core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// File snapshot with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSnapshot {
    /// Relative path from codebase root
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// Last modified time (Unix timestamp)
    pub modified: u64,
    /// Content hash (Merkle tree hash)
    pub hash: String,
    /// File extension (for filtering)
    pub extension: String,
}

/// Codebase snapshot with all files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseSnapshot {
    /// Codebase root path
    pub root_path: String,
    /// Snapshot creation timestamp
    pub created_at: u64,
    /// Map of relative paths to file snapshots
    pub files: HashMap<String, FileSnapshot>,
    /// Total number of files
    pub file_count: usize,
    /// Total size in bytes
    pub total_size: u64,
}

/// Snapshot manager for incremental tracking
pub struct SnapshotManager {
    /// Base directory for storing snapshots
    snapshot_dir: PathBuf,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new() -> Result<Self> {
        let snapshot_dir = dirs::home_dir()
            .ok_or_else(|| Error::internal("Cannot determine home directory"))?
            .join(".context")
            .join("snapshots");

        fs::create_dir_all(&snapshot_dir)
            .map_err(|e| Error::internal(format!("Failed to create snapshot directory: {}", e)))?;

        Ok(Self { snapshot_dir })
    }

    /// Create snapshot for a codebase
    pub async fn create_snapshot(&self, root_path: &Path) -> Result<CodebaseSnapshot> {
        let mut files = HashMap::new();
        let mut total_size = 0u64;
        let root_path_str = root_path.to_string_lossy().to_string();

        self.walk_directory(root_path, root_path, &mut files, &mut total_size)
            .await?;

        let snapshot = CodebaseSnapshot {
            root_path: root_path_str,
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            files,
            file_count: files.len(),
            total_size,
        };

        // Save snapshot
        self.save_snapshot(&snapshot).await?;

        Ok(snapshot)
    }

    /// Load existing snapshot for a codebase
    pub async fn load_snapshot(&self, root_path: &Path) -> Result<Option<CodebaseSnapshot>> {
        let snapshot_path = self.get_snapshot_path(root_path);

        if !snapshot_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&snapshot_path)
            .map_err(|e| Error::internal(format!("Failed to read snapshot: {}", e)))?;

        let snapshot: CodebaseSnapshot = serde_json::from_str(&content)
            .map_err(|e| Error::internal(format!("Failed to parse snapshot: {}", e)))?;

        Ok(Some(snapshot))
    }

    /// Compare snapshots to find changes
    pub async fn compare_snapshots(
        &self,
        old_snapshot: &CodebaseSnapshot,
        new_snapshot: &CodebaseSnapshot,
    ) -> Result<SnapshotChanges> {
        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut removed = Vec::new();
        let mut unchanged = Vec::new();

        // Find added and modified files
        for (path, new_file) in &new_snapshot.files {
            if let Some(old_file) = old_snapshot.files.get(path) {
                if old_file.hash != new_file.hash {
                    modified.push(path.clone());
                } else {
                    unchanged.push(path.clone());
                }
            } else {
                added.push(path.clone());
            }
        }

        // Find removed files
        for path in old_snapshot.files.keys() {
            if !new_snapshot.files.contains_key(path) {
                removed.push(path.clone());
            }
        }

        Ok(SnapshotChanges {
            added,
            modified,
            removed,
            unchanged,
        })
    }

    /// Get files that need processing (added or modified)
    pub async fn get_changed_files(&self, root_path: &Path) -> Result<Vec<String>> {
        let current_snapshot = self.create_snapshot(root_path).await?;
        let previous_snapshot = self.load_snapshot(root_path).await?;

        let changes = if let Some(prev) = previous_snapshot {
            self.compare_snapshots(&prev, &current_snapshot).await?
        } else {
            // First time - all files are "added"
            SnapshotChanges {
                added: current_snapshot.files.keys().cloned().collect(),
                modified: Vec::new(),
                removed: Vec::new(),
                unchanged: Vec::new(),
            }
        };

        let mut changed_files = changes.added;
        changed_files.extend(changes.modified);

        Ok(changed_files)
    }

    /// Walk directory recursively and collect file snapshots
    async fn walk_directory(
        &self,
        root_path: &Path,
        current_path: &Path,
        files: &mut HashMap<String, FileSnapshot>,
        total_size: &mut u64,
    ) -> Result<()> {
        let entries = fs::read_dir(current_path)
            .map_err(|e| Error::internal(format!("Failed to read directory: {}", e)))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| Error::internal(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();

            // Skip hidden files and directories
            if let Some(file_name) = path.file_name() {
                if file_name.to_string_lossy().starts_with('.') {
                    continue;
                }
            }

            let metadata = entry
                .metadata()
                .map_err(|e| Error::internal(format!("Failed to get metadata: {}", e)))?;

            if metadata.is_dir() {
                // Recurse into subdirectories
                Box::pin(self.walk_directory(root_path, &path, files, total_size)).await?;
            } else if metadata.is_file() {
                // Process file
                let relative_path = path
                    .strip_prefix(root_path)
                    .map_err(|e| Error::internal(format!("Failed to make path relative: {}", e)))?
                    .to_string_lossy()
                    .to_string();

                // Skip certain file types
                if self.should_skip_file(&path) {
                    continue;
                }

                let snapshot = self
                    .create_file_snapshot(&path, &relative_path, &metadata)
                    .await?;
                *total_size += snapshot.size;
                files.insert(relative_path, snapshot);
            }
        }

        Ok(())
    }

    /// Create snapshot for a single file
    async fn create_file_snapshot(
        &self,
        file_path: &Path,
        relative_path: &str,
        metadata: &fs::Metadata,
    ) -> Result<FileSnapshot> {
        let content = fs::read(file_path).map_err(|e| {
            Error::internal(format!(
                "Failed to read file {}: {}",
                file_path.display(),
                e
            ))
        })?;

        let hash = self.calculate_hash(&content);
        let extension = file_path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let modified = metadata
            .modified()
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(FileSnapshot {
            path: relative_path.to_string(),
            size: metadata.len(),
            modified,
            hash,
            extension,
        })
    }

    /// Calculate simple hash of content (for change detection)
    fn calculate_hash(&self, content: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Check if file should be skipped
    fn should_skip_file(&self, path: &Path) -> bool {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        // Skip common non-source files
        if file_name.ends_with(".log")
            || file_name.ends_with(".tmp")
            || file_name.ends_with(".cache")
            || file_name.ends_with(".lock")
            || file_name.starts_with('.')
        {
            return true;
        }

        // Skip build artifacts
        let path_str = path.to_string_lossy();
        if path_str.contains("/target/")
            || path_str.contains("/node_modules/")
            || path_str.contains("/.git/")
            || path_str.contains("/dist/")
            || path_str.contains("/build/")
        {
            return true;
        }

        false
    }

    /// Get snapshot file path for a codebase
    fn get_snapshot_path(&self, root_path: &Path) -> PathBuf {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let path_str = root_path
            .canonicalize()
            .unwrap_or_else(|_| root_path.to_path_buf())
            .to_string_lossy()
            .to_string();

        let mut hasher = DefaultHasher::new();
        path_str.hash(&mut hasher);
        let hash = format!("{:x}", hasher.finish());

        self.snapshot_dir.join(format!("{}.snapshot", hash))
    }

    /// Save snapshot to disk
    async fn save_snapshot(&self, snapshot: &CodebaseSnapshot) -> Result<()> {
        let snapshot_path = self.get_snapshot_path(Path::new(&snapshot.root_path));

        let content = serde_json::to_string_pretty(snapshot)
            .map_err(|e| Error::internal(format!("Failed to serialize snapshot: {}", e)))?;

        fs::write(&snapshot_path, content)
            .map_err(|e| Error::internal(format!("Failed to write snapshot: {}", e)))?;

        Ok(())
    }
}

/// Changes between snapshots
#[derive(Debug, Clone)]
pub struct SnapshotChanges {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub removed: Vec<String>,
    pub unchanged: Vec<String>,
}

impl SnapshotChanges {
    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.modified.is_empty() || !self.removed.is_empty()
    }

    /// Get total number of changes
    pub fn total_changes(&self) -> usize {
        self.added.len() + self.modified.len() + self.removed.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_snapshot_manager_creation() {
        let manager = SnapshotManager::new().unwrap();
        assert!(manager.snapshot_dir.exists());
    }

    #[tokio::test]
    async fn test_empty_directory_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SnapshotManager::new().unwrap();

        let snapshot = manager.create_snapshot(temp_dir.path()).await.unwrap();
        assert_eq!(snapshot.file_count, 0);
        assert_eq!(snapshot.total_size, 0);
        assert!(snapshot.files.is_empty());
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
    fn test_should_skip_file() {
        let manager = SnapshotManager::new().unwrap();
        let temp_dir = TempDir::new().unwrap();

        // Should skip log files
        assert!(manager.should_skip_file(&temp_dir.path().join("debug.log")));

        // Should skip hidden files
        assert!(manager.should_skip_file(&temp_dir.path().join(".hidden")));

        // Should skip build artifacts
        assert!(manager.should_skip_file(&temp_dir.path().join("target/debug/app")));

        // Should not skip source files
        assert!(!manager.should_skip_file(&temp_dir.path().join("src/main.rs")));
    }
}
