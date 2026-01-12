//! Snapshot manager for incremental codebase tracking
//!
//! Uses the `ignore` crate (same as ripgrep) for proper git-aware file traversal,
//! automatically respecting:
//! - `.gitignore` files (including nested ones in subdirectories)
//! - `.git/info/exclude`
//! - Global gitignore from git config
//! - Hidden files and directories

use super::{CodebaseSnapshot, FileSnapshot, SnapshotChanges};
use crate::domain::error::{Error, Result};
use ignore::WalkBuilder;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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

    /// Create a disabled snapshot manager (uses temporary directory)
    pub fn new_disabled() -> Self {
        Self {
            snapshot_dir: std::env::temp_dir().join("mcp_disabled_snapshots"),
        }
    }

    /// Create snapshot for a codebase
    pub async fn create_snapshot(&self, root_path: &Path) -> Result<CodebaseSnapshot> {
        let root_path_buf = root_path.to_path_buf();
        let root_path_str = root_path.to_string_lossy().to_string();
        let snapshot_manager = Self {
            snapshot_dir: self.snapshot_dir.clone(),
        };

        let (files, total_size) = tokio::task::spawn_blocking(move || {
            snapshot_manager.walk_directory_sync(&root_path_buf)
        })
        .await
        .map_err(|e| Error::internal(format!("Blocking task failed: {}", e)))??;

        let file_count = files.len();
        let snapshot = CodebaseSnapshot {
            root_path: root_path_str,
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            files,
            file_count,
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
        // Load previous snapshot FIRST, before creating new one
        // (create_snapshot saves to disk, which would overwrite the previous snapshot)
        let previous_snapshot = self.load_snapshot(root_path).await?;
        let current_snapshot = self.create_snapshot(root_path).await?;

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

    /// Walk directory using git-aware traversal (respects .gitignore)
    ///
    /// Uses the `ignore` crate which automatically handles:
    /// - All `.gitignore` files (root and nested in subdirectories)
    /// - `.git/info/exclude`
    /// - Global gitignore from `core.excludesFile` git config
    /// - Hidden files (dotfiles)
    fn walk_directory_sync(
        &self,
        root_path: &Path,
    ) -> Result<(HashMap<String, FileSnapshot>, u64)> {
        let mut files = HashMap::new();
        let mut total_size = 0u64;

        // Build a git-aware walker using the ignore crate
        // This is the same approach ripgrep uses
        let walker = WalkBuilder::new(root_path)
            // Respect .gitignore files
            .git_ignore(true)
            // Respect .git/info/exclude
            .git_exclude(true)
            // Respect global gitignore from git config
            .git_global(true)
            // Skip hidden files and directories (dotfiles)
            .hidden(true)
            // Don't follow symlinks (safer default)
            .follow_links(false)
            // Use reasonable defaults for parallel walking
            .threads(1) // Single-threaded for deterministic ordering
            .build();

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    // Log warning but continue - don't fail on permission errors etc.
                    tracing::warn!("Error walking directory: {}", e);
                    continue;
                }
            };

            // Skip directories (we only care about files)
            let file_type = match entry.file_type() {
                Some(ft) => ft,
                None => continue,
            };

            if !file_type.is_file() {
                continue;
            }

            let path = entry.path();

            // Get relative path from root
            let relative_path = match path.strip_prefix(root_path) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(_) => continue,
            };

            // Get file metadata
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!("Failed to get metadata for {}: {}", path.display(), e);
                    continue;
                }
            };

            // Create file snapshot
            match self.create_file_snapshot_sync(path, &relative_path, &metadata) {
                Ok(snapshot) => {
                    total_size += snapshot.size;
                    files.insert(relative_path, snapshot);
                }
                Err(e) => {
                    tracing::warn!("Failed to create snapshot for {}: {}", path.display(), e);
                    continue;
                }
            }
        }

        Ok((files, total_size))
    }

    /// Create snapshot for a single file
    fn create_file_snapshot_sync(
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

    /// Calculate SHA256 hash of content (for change detection)
    fn calculate_hash(&self, content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let result = hasher.finalize();
        format!("{:x}", result)
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
