//! File Discovery Service - Handles file scanning and filtering for indexing
//!
//! Single Responsibility: Discover and filter files eligible for indexing.

use crate::domain::error::{Error, Result};
use crate::domain::types::Language;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Configuration for file discovery
#[derive(Debug, Clone)]
pub struct DiscoveryOptions {
    /// File extensions to include (empty = all supported)
    pub extensions: Vec<String>,
    /// Patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Maximum file size in bytes
    pub max_file_size: Option<u64>,
    /// Follow symbolic links
    pub follow_symlinks: bool,
}

impl Default for DiscoveryOptions {
    fn default() -> Self {
        Self {
            extensions: Vec::new(),
            exclude_patterns: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "target".to_string(),
                "__pycache__".to_string(),
                ".venv".to_string(),
            ],
            max_file_size: Some(1024 * 1024), // 1MB default
            follow_symlinks: false,
        }
    }
}

/// Result of file discovery
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    /// Files discovered
    pub files: Vec<PathBuf>,
    /// Files skipped with reasons
    pub skipped: Vec<(PathBuf, String)>,
    /// Total bytes discovered
    pub total_bytes: u64,
}

/// Service for discovering files to index
pub struct FileDiscoveryService {
    options: DiscoveryOptions,
}

impl Default for FileDiscoveryService {
    fn default() -> Self {
        Self::new(DiscoveryOptions::default())
    }
}

impl FileDiscoveryService {
    /// Create a new file discovery service
    pub fn new(options: DiscoveryOptions) -> Self {
        Self { options }
    }

    /// Discover all indexable files in a directory
    pub async fn discover(&self, root: &Path) -> Result<DiscoveryResult> {
        if !root.exists() || !root.is_dir() {
            return Err(Error::not_found(format!(
                "Directory not found: {}",
                root.display()
            )));
        }

        let mut result = DiscoveryResult {
            files: Vec::new(),
            skipped: Vec::new(),
            total_bytes: 0,
        };

        self.discover_recursive(root, root, &mut result).await?;

        Ok(result)
    }

    /// Recursively discover files
    async fn discover_recursive(
        &self,
        root: &Path,
        current: &Path,
        result: &mut DiscoveryResult,
    ) -> Result<()> {
        let mut entries = fs::read_dir(current)
            .await
            .map_err(|e| Error::io(format!("Failed to read directory: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| Error::io(format!("Failed to read entry: {}", e)))?
        {
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip hidden files and excluded patterns
            if file_name.starts_with('.') || self.is_excluded(&path) {
                continue;
            }

            let metadata = match entry.metadata().await {
                Ok(m) => m,
                Err(e) => {
                    result
                        .skipped
                        .push((path.clone(), format!("metadata error: {}", e)));
                    continue;
                }
            };

            if metadata.is_dir() {
                // Recurse into subdirectories
                Box::pin(self.discover_recursive(root, &path, result)).await?;
            } else if metadata.is_file() {
                // Check file eligibility
                if let Some(reason) = self.check_file_eligibility(&path, &metadata) {
                    result.skipped.push((path, reason));
                } else {
                    result.total_bytes += metadata.len();
                    result.files.push(path);
                }
            } else if metadata.is_symlink() && self.options.follow_symlinks {
                // Handle symlinks if configured
                if let Ok(resolved) = fs::canonicalize(&path).await {
                    if resolved.is_file() {
                        if let Ok(meta) = fs::metadata(&resolved).await {
                            if self.check_file_eligibility(&resolved, &meta).is_none() {
                                result.total_bytes += meta.len();
                                result.files.push(path);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if a file is eligible for indexing, returns None if eligible or reason if not
    fn check_file_eligibility(&self, path: &Path, metadata: &std::fs::Metadata) -> Option<String> {
        // Check file size
        if let Some(max_size) = self.options.max_file_size {
            if metadata.len() > max_size {
                return Some(format!("file too large: {} bytes", metadata.len()));
            }
        }

        // Check extension
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let language = Language::from_extension(ext);

        if matches!(language, Language::Unknown) {
            return Some(format!("unsupported extension: {}", ext));
        }

        // Check if extension is in allowed list (if specified)
        if !self.options.extensions.is_empty() && !self.options.extensions.iter().any(|e| e == ext)
        {
            return Some(format!("extension not in allowed list: {}", ext));
        }

        None
    }

    /// Check if a path matches exclusion patterns
    fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.options
            .exclude_patterns
            .iter()
            .any(|pattern| path_str.contains(pattern))
    }

    /// Check if a file type is supported
    pub fn is_supported(&self, ext: &str) -> bool {
        let language = Language::from_extension(ext);
        !matches!(language, Language::Unknown)
    }

    /// Detect language from file path
    pub fn detect_language(&self, path: &Path) -> Language {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        Language::from_extension(ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let options = DiscoveryOptions::default();
        assert!(!options.exclude_patterns.is_empty());
        assert!(options.max_file_size.is_some());
    }

    #[test]
    fn test_is_supported() {
        let service = FileDiscoveryService::default();
        assert!(service.is_supported("rs"));
        assert!(service.is_supported("py"));
        assert!(service.is_supported("js"));
        assert!(!service.is_supported("xyz"));
    }

    #[test]
    fn test_detect_language() {
        let service = FileDiscoveryService::default();
        assert_eq!(
            service.detect_language(Path::new("test.rs")),
            Language::Rust
        );
        assert_eq!(
            service.detect_language(Path::new("test.py")),
            Language::Python
        );
    }
}
