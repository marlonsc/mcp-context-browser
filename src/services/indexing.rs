//! Indexing service for processing codebases

use crate::chunking::IntelligentChunker;
use crate::core::error::{Error, Result};
use crate::core::types::CodeChunk;
use crate::services::context::ContextService;
use crate::snapshot::SnapshotManager;
use crate::sync::SyncManager;
use futures::future::join_all;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;

/// Advanced indexing service with snapshot-based incremental processing
pub struct IndexingService {
    context_service: Arc<ContextService>,
    snapshot_manager: SnapshotManager,
    sync_manager: Option<Arc<SyncManager>>,
    chunker: IntelligentChunker,
}

impl IndexingService {
    /// Create a new indexing service
    pub fn new(context_service: Arc<ContextService>) -> Result<Self> {
        Ok(Self {
            context_service,
            snapshot_manager: SnapshotManager::new()?,
            sync_manager: None,
            chunker: IntelligentChunker::new(),
        })
    }

    /// Create indexing service with sync coordination
    pub fn with_sync_manager(
        context_service: Arc<ContextService>,
        sync_manager: Arc<SyncManager>,
    ) -> Result<Self> {
        Ok(Self {
            context_service,
            snapshot_manager: SnapshotManager::new()?,
            sync_manager: Some(sync_manager),
            chunker: IntelligentChunker::new(),
        })
    }

    /// Index a directory with incremental processing and sync coordination
    pub async fn index_directory(&self, path: &Path, collection: &str) -> Result<usize> {
        if !path.exists() || !path.is_dir() {
            return Err(Error::not_found("Directory not found"));
        }

        // Canonicalize path for consistent snapshots
        let canonical_path = path
            .canonicalize()
            .map_err(|e| Error::io(format!("Failed to canonicalize path: {}", e)))?;

        // Check if sync is needed (if sync manager is available)
        if let Some(sync_mgr) = &self.sync_manager {
            if sync_mgr.should_debounce(&canonical_path).await? {
                println!("[INDEX] Skipping {} - debounced", canonical_path.display());
                return Ok(0);
            }
        }

        // Get changed files using snapshots
        let changed_files = self
            .snapshot_manager
            .get_changed_files(&canonical_path)
            .await?;
        println!(
            "[INDEX] Found {} changed files in {}",
            changed_files.len(),
            canonical_path.display()
        );

        if changed_files.is_empty() {
            return Ok(0);
        }

        // Process changed files in parallel batches
        let total_chunks = self
            .process_files_parallel(&canonical_path, &changed_files, collection)
            .await;

        // Update sync timestamp if sync manager is available
        // TODO: Add public method to update sync timestamp when available
        // if let Some(sync_mgr) = &self.sync_manager {
        //     sync_mgr.update_last_sync(&canonical_path).await;
        // }

        println!(
            "[INDEX] Completed indexing {} files with {} total chunks",
            changed_files.len(),
            total_chunks
        );
        Ok(total_chunks)
    }

    /// Process a single file into intelligent chunks using tree-sitter
    async fn process_file(&self, path: &Path) -> Result<Vec<CodeChunk>> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| Error::io(format!("Failed to read file {}: {}", path.display(), e)))?;

        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        let file_name = path.display().to_string();
        let language = self.detect_language(path)?;

        // Use intelligent tree-sitter based chunking
        let mut chunks = self.chunker.chunk_code(&content, &file_name, language);

        // Filter out chunks that are too small
        chunks.retain(|chunk| chunk.content.len() >= 25 && chunk.content.lines().count() >= 2);

        // Limit chunks per file to avoid explosion
        if chunks.len() > 50 {
            chunks.truncate(50);
        }

        Ok(chunks)
    }

    /// Detect programming language from file extension
    fn detect_language(&self, path: &Path) -> Result<crate::core::types::Language> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "rs" => Ok(crate::core::types::Language::Rust),
            "py" => Ok(crate::core::types::Language::Python),
            "js" => Ok(crate::core::types::Language::JavaScript),
            "ts" => Ok(crate::core::types::Language::TypeScript),
            "java" => Ok(crate::core::types::Language::Java),
            "cpp" | "cc" | "cxx" => Ok(crate::core::types::Language::Cpp),
            "c" => Ok(crate::core::types::Language::C),
            "go" => Ok(crate::core::types::Language::Go),
            "php" => Ok(crate::core::types::Language::Php),
            "rb" => Ok(crate::core::types::Language::Ruby),
            _ => Ok(crate::core::types::Language::Unknown),
        }
    }

    /// Process files in parallel batches for better performance
    async fn process_files_parallel(
        &self,
        canonical_path: &Path,
        changed_files: &[String],
        collection: &str,
    ) -> usize {
        let batch_size = 10; // Process 10 files concurrently
        let mut total_chunks = 0;

        // Process files in batches
        for batch in changed_files.chunks(batch_size) {
            let futures: Vec<_> = batch
                .iter()
                .filter_map(|file_path| {
                    let full_path = canonical_path.join(file_path);

                    // Only process supported file types
                    if let Some(ext) = full_path.extension().and_then(|e| e.to_str()) {
                        if self.is_supported_file_type(ext) {
                            let file_path_clone = file_path.clone();
                            let full_path_clone = full_path.clone();
                            Some(async move {
                                match self.process_file(&full_path_clone).await {
                                    Ok(file_chunks) => {
                                        if !file_chunks.is_empty() {
                                            println!(
                                                "[INDEX] Processed {} chunks from {}",
                                                file_chunks.len(),
                                                file_path_clone
                                            );
                                            Some(file_chunks)
                                        } else {
                                            None
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "[INDEX] Failed to process {}: {}",
                                            file_path_clone, e
                                        );
                                        None
                                    }
                                }
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            // Execute batch concurrently
            let batch_results = join_all(futures).await;

            // Store chunks sequentially to avoid concurrent access issues
            for chunks in batch_results.into_iter().flatten() {
                match self.context_service.store_chunks(collection, &chunks).await {
                    Ok(()) => {
                        total_chunks += chunks.len();
                    }
                    Err(e) => {
                        eprintln!("[INDEX] Failed to store batch of chunks: {}", e);
                        // Continue with other batches
                    }
                }
            }
        }

        total_chunks
    }

    /// Check if file type is supported for indexing
    fn is_supported_file_type(&self, ext: &str) -> bool {
        matches!(
            ext.to_lowercase().as_str(),
            "rs" | "py"
                | "js"
                | "ts"
                | "java"
                | "cpp"
                | "cc"
                | "cxx"
                | "c"
                | "go"
                | "php"
                | "rb"
                | "scala"
                | "kt"
                | "swift"
                | "cs"
                | "fs"
                | "vb"
                | "pl"
                | "pm"
                | "sh"
                | "bash"
                | "zsh"
                | "fish"
                | "ps1"
                | "sql"
                | "html"
                | "xml"
                | "json"
                | "yaml"
                | "yml"
                | "toml"
                | "ini"
                | "cfg"
                | "md"
                | "txt"
                | "rst"
        )
    }
}
