//! Snapshot manager for incremental codebase tracking
//!
//! Uses the `ignore` crate (same as ripgrep) for proper git-aware file traversal,
//! automatically respecting:
//! - `.gitignore` files (including nested ones in subdirectories)
//! - `.git/info/exclude`
//! - Global gitignore from git config
//! - Hidden files and directories

use super::{CodebaseSnapshot, FileSnapshot, HashCalculator, SnapshotChanges, SnapshotComparator};
use crate::domain::error::{Error, Result};
use crate::domain::ports::SnapshotProvider;
use crate::infrastructure::utils::FileUtils;
use async_trait::async_trait;
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

/// Snapshot manager for incremental tracking
pub struct SnapshotManager {
    /// Base directory for storing snapshots
    snapshot_dir: PathBuf,
    /// Hash calculator service
    hash_calculator: Arc<HashCalculator>,
    /// Snapshot comparator service
    comparator: Arc<SnapshotComparator>,
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

        Ok(Self {
            snapshot_dir,
            hash_calculator: Arc::new(HashCalculator::new()),
            comparator: Arc::new(SnapshotComparator::new()),
        })
    }

    /// Create a disabled snapshot manager (uses temporary directory)
    pub fn new_disabled() -> Self {
        Self {
            snapshot_dir: std::env::temp_dir().join("mcp_disabled_snapshots"),
            hash_calculator: Arc::new(HashCalculator::new()),
            comparator: Arc::new(SnapshotComparator::new()),
        }
    }

    /// Create snapshot for a codebase
    pub async fn create_snapshot(&self, root_path: &Path) -> Result<CodebaseSnapshot> {
        let root_path_buf = root_path.to_path_buf();
        let root_path_str = root_path.to_string_lossy().to_string();
        let snapshot_manager = Self {
            snapshot_dir: self.snapshot_dir.clone(),
            hash_calculator: Arc::clone(&self.hash_calculator),
            comparator: Arc::clone(&self.comparator),
        };

        let (files, total_size) = tokio::task::spawn_blocking(move || {
            snapshot_manager.walk_directory_sync(&root_path_buf)
        })
        .await
        .map_err(|e| Error::internal(format!("Blocking task failed: {}", e)))??;

        let file_count = files.len();
        let snapshot = CodebaseSnapshot {
            root_path: root_path_str,
            created_at: crate::infrastructure::utils::TimeUtils::now_unix_secs(),
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

        let snapshot: CodebaseSnapshot = FileUtils::read_json(&snapshot_path, "snapshot").await?;

        Ok(Some(snapshot))
    }

    /// Compare snapshots to find changes
    ///
    /// Delegates to `SnapshotComparator` service for the actual comparison logic.
    pub async fn compare_snapshots(
        &self,
        old_snapshot: &CodebaseSnapshot,
        new_snapshot: &CodebaseSnapshot,
    ) -> Result<SnapshotChanges> {
        // Delegate to focused SnapshotComparator service
        Ok(self.comparator.compare(old_snapshot, new_snapshot))
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
    ///
    /// Delegates to `HashCalculator` service for the actual hashing.
    fn calculate_hash(&self, content: &[u8]) -> String {
        self.hash_calculator.hash_bytes(content)
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
        FileUtils::write_json(&snapshot_path, snapshot, "snapshot").await
    }
}

#[async_trait]
impl SnapshotProvider for SnapshotManager {
    async fn get_changed_files(&self, root_path: &Path) -> Result<Vec<String>> {
        self.get_changed_files(root_path).await
    }
}
