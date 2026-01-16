//! Filesystem Snapshot Provider
//!
//! Snapshot provider implementation using filesystem storage with JSON serialization.

use crate::adapters::chunking::language_helpers::language_from_extension;
use async_trait::async_trait;
use mcb_domain::entities::codebase::{CodebaseSnapshot, FileSnapshot, SnapshotChanges};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::snapshot::SnapshotProvider;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use uuid::Uuid;

/// Filesystem-based snapshot provider
///
/// Stores snapshots as JSON files in a configured directory.
/// Uses SHA-256 for content hashing and supports incremental updates.
pub struct FilesystemSnapshotProvider {
    /// Directory to store snapshot files
    snapshot_dir: PathBuf,
}

impl FilesystemSnapshotProvider {
    /// Create a new filesystem snapshot provider
    pub fn new(snapshot_dir: PathBuf) -> Self {
        Self { snapshot_dir }
    }

    /// Create with default snapshot directory
    pub fn with_default_dir() -> Self {
        Self {
            snapshot_dir: Self::default_snapshot_dir(),
        }
    }

    /// Create as Arc for sharing
    pub fn new_shared(snapshot_dir: PathBuf) -> Arc<Self> {
        Arc::new(Self::new(snapshot_dir))
    }

    fn default_snapshot_dir() -> PathBuf {
        PathBuf::from(".mcb/snapshots")
    }

    /// Get the snapshot file path for a codebase
    fn snapshot_path(&self, root_path: &Path) -> PathBuf {
        let hash = Self::path_hash(root_path);
        self.snapshot_dir.join(format!("{}.json", hash))
    }

    /// Create a hash from a path for filename
    fn path_hash(path: &Path) -> String {
        let mut hasher = Sha256::new();
        hasher.update(path.to_string_lossy().as_bytes());
        let result = hasher.finalize();
        hex::encode(&result[..8]) // Use first 8 bytes for shorter filenames
    }

    /// Compute content hash for a file
    async fn compute_file_hash(path: &Path) -> Result<String> {
        let content = fs::read(path)
            .await
            .map_err(|e| Error::io(format!("Failed to read file for hashing: {}", e)))?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(hex::encode(hasher.finalize()))
    }

    /// Get current timestamp
    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }

    /// Detect language from file extension
    fn detect_language(path: &Path) -> String {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(language_from_extension)
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Check if file should be included in snapshot
    fn should_include_directory(path: &Path) -> bool {
        let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        !matches!(dir_name, ".git" | "node_modules" | "target" | "__pycache__")
    }

    fn should_include_file(path: &Path) -> bool {
        // Skip hidden files and common non-code directories
        let path_str = path.to_string_lossy();
        if path_str.contains("/.") || path_str.contains("\\.") {
            return false;
        }
        if path_str.contains("/node_modules/")
            || path_str.contains("/target/")
            || path_str.contains("/__pycache__/")
            || path_str.contains("/.git/")
        {
            return false;
        }

        // Only include code files
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                matches!(
                    ext.to_lowercase().as_str(),
                    "rs" | "py"
                        | "js"
                        | "ts"
                        | "jsx"
                        | "tsx"
                        | "go"
                        | "java"
                        | "c"
                        | "cpp"
                        | "cc"
                        | "cxx"
                        | "cs"
                        | "rb"
                        | "php"
                        | "swift"
                        | "kt"
                )
            })
            .unwrap_or(false)
    }
}

impl Default for FilesystemSnapshotProvider {
    fn default() -> Self {
        Self::with_default_dir()
    }
}

#[async_trait]
impl SnapshotProvider for FilesystemSnapshotProvider {
    async fn create_snapshot(&self, root_path: &Path) -> Result<CodebaseSnapshot> {
        // Ensure snapshot directory exists
        fs::create_dir_all(&self.snapshot_dir)
            .await
            .map_err(|e| Error::io(format!("Failed to create snapshot directory: {}", e)))?;

        let mut files = HashMap::new();
        let mut total_size: u64 = 0;
        let collection = root_path.to_string_lossy().to_string();

        // Walk directory tree
        let mut dirs_to_visit = vec![root_path.to_path_buf()];
        while let Some(dir) = dirs_to_visit.pop() {
            let mut entries = match fs::read_dir(&dir).await {
                Ok(entries) => entries,
                Err(_) => continue,
            };

            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();

                if path.is_dir() && Self::should_include_directory(&path) {
                    dirs_to_visit.push(path);
                } else if Self::should_include_file(&path) {
                    let metadata = match fs::metadata(&path).await {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    let size = metadata.len();
                    total_size += size;

                    let modified_at = metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);

                    let hash = Self::compute_file_hash(&path).await.unwrap_or_default();
                    let rel_path = path
                        .strip_prefix(root_path)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();

                    let file_snapshot = FileSnapshot {
                        path: rel_path.clone(),
                        modified_at,
                        size,
                        hash,
                        language: Self::detect_language(&path),
                    };

                    files.insert(rel_path, file_snapshot);
                }
            }
        }

        let total_files = files.len();
        let snapshot = CodebaseSnapshot {
            id: Uuid::new_v4().to_string(),
            created_at: Self::current_timestamp(),
            collection,
            files,
            total_files,
            total_size,
        };

        // Save snapshot to file
        let snapshot_path = self.snapshot_path(root_path);
        let json = serde_json::to_string_pretty(&snapshot)
            .map_err(|e| Error::internal(format!("Failed to serialize snapshot: {}", e)))?;
        fs::write(&snapshot_path, json)
            .await
            .map_err(|e| Error::io(format!("Failed to write snapshot file: {}", e)))?;

        Ok(snapshot)
    }

    async fn load_snapshot(&self, root_path: &Path) -> Result<Option<CodebaseSnapshot>> {
        let snapshot_path = self.snapshot_path(root_path);

        if !snapshot_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&snapshot_path)
            .await
            .map_err(|e| Error::io(format!("Failed to read snapshot file: {}", e)))?;

        let snapshot: CodebaseSnapshot = serde_json::from_str(&content)
            .map_err(|e| Error::internal(format!("Failed to parse snapshot: {}", e)))?;

        Ok(Some(snapshot))
    }

    async fn compare_snapshots(
        &self,
        old_snapshot: &CodebaseSnapshot,
        new_snapshot: &CodebaseSnapshot,
    ) -> Result<SnapshotChanges> {
        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut removed = Vec::new();

        // Find added and modified files
        for (path, new_file) in &new_snapshot.files {
            match old_snapshot.files.get(path) {
                Some(old_file) => {
                    // File exists in both - check if modified
                    if old_file.hash != new_file.hash {
                        modified.push(path.clone());
                    }
                }
                None => {
                    // File only in new snapshot - added
                    added.push(path.clone());
                }
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
        })
    }

    async fn get_changed_files(&self, root_path: &Path) -> Result<Vec<String>> {
        let old_snapshot = self.load_snapshot(root_path).await?;
        let new_snapshot = self.create_snapshot(root_path).await?;

        match old_snapshot {
            Some(old) => {
                let changes = self.compare_snapshots(&old, &new_snapshot).await?;
                let mut changed = changes.added;
                changed.extend(changes.modified);
                Ok(changed)
            }
            None => {
                // No previous snapshot - all files are "changed"
                Ok(new_snapshot.files.keys().cloned().collect())
            }
        }
    }
}
