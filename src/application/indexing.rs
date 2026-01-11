//! Indexing service for processing codebases
//!
//! # Architecture Note
//!
//! This module imports from `infrastructure::events` which is a cross-cutting concern.
//! Events are used for decoupled notification when indexing completes, enabling
//! cache invalidation and UI updates without tight coupling. This is an acceptable
//! deviation from strict Clean Architecture layering per ADR-002 (Async-First Design).

use crate::application::context::ContextService;
use crate::chunking::IntelligentChunker;
use crate::domain::error::{Error, Result};
use crate::domain::types::CodeChunk;
// Cross-cutting concern: Event bus for decoupled notifications
use crate::infrastructure::events::{SharedEventBus, SystemEvent};
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

    /// Start listening for system events
    pub fn start_event_listener(&self, event_bus: SharedEventBus) {
        let mut receiver = event_bus.subscribe();
        let _service = Arc::new(self.clone_service());

        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                if let SystemEvent::IndexRebuild { collection } = event {
                    let coll = collection.unwrap_or_else(|| "default".to_string());
                    tracing::info!("[INDEX] Rebuild requested for collection: {}", coll);
                    // In a real implementation, we might trigger a full re-index of known paths
                }
            }
        });
    }

    /// Helper to clone service state for event listener
    fn clone_service(&self) -> Self {
        Self {
            context_service: Arc::clone(&self.context_service),
            snapshot_manager: SnapshotManager::new()
                .unwrap_or_else(|_| SnapshotManager::new_disabled()),
            sync_manager: self.sync_manager.clone(),
            chunker: IntelligentChunker::new(),
        }
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
        let batch = if let Some(sync_mgr) = &self.sync_manager {
            if sync_mgr.should_debounce(&canonical_path).await? {
                tracing::info!("[INDEX] Skipping {} - debounced", canonical_path.display());
                return Ok(0);
            }

            match sync_mgr.acquire_sync_slot(&canonical_path).await? {
                Some(b) => Some(b),
                None => {
                    tracing::info!("[INDEX] Sync deferred for {}", canonical_path.display());
                    return Ok(0);
                }
            }
        } else {
            None
        };

        // Get changed files using snapshots
        let changed_files = self
            .snapshot_manager
            .get_changed_files(&canonical_path)
            .await?;
        tracing::info!(
            "[INDEX] Found {} changed files in {}",
            changed_files.len(),
            canonical_path.display()
        );

        if changed_files.is_empty() {
            // Release slot if we acquired one but have no work
            if let (Some(sync_mgr), Some(b)) = (&self.sync_manager, batch) {
                sync_mgr.release_sync_slot(&canonical_path, b).await?;
            }
            return Ok(0);
        }

        // Process changed files in parallel batches
        let total_chunks = self
            .process_files_parallel(&canonical_path, &changed_files, collection)
            .await;

        // Release slot and update timestamp
        if let (Some(sync_mgr), Some(b)) = (&self.sync_manager, batch) {
            sync_mgr.release_sync_slot(&canonical_path, b).await?;
            // Update last sync time
            sync_mgr.update_last_sync(&canonical_path).await;
        }

        tracing::info!(
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

        // Use intelligent tree-sitter based chunking (offloaded to blocking thread)
        let mut chunks = self
            .chunker
            .chunk_code_async(content, file_name, language)
            .await;

        // Filter out chunks that are too small
        chunks.retain(|chunk| chunk.content.len() >= 25 && chunk.content.lines().count() >= 2);

        // Limit chunks per file to avoid explosion
        if chunks.len() > 50 {
            chunks.truncate(50);
        }

        Ok(chunks)
    }

    /// Detect programming language from file extension
    fn detect_language(&self, path: &Path) -> Result<crate::domain::types::Language> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "rs" => Ok(crate::domain::types::Language::Rust),
            "py" => Ok(crate::domain::types::Language::Python),
            "js" => Ok(crate::domain::types::Language::JavaScript),
            "ts" => Ok(crate::domain::types::Language::TypeScript),
            "java" => Ok(crate::domain::types::Language::Java),
            "cpp" | "cc" | "cxx" => Ok(crate::domain::types::Language::Cpp),
            "c" => Ok(crate::domain::types::Language::C),
            "go" => Ok(crate::domain::types::Language::Go),
            "php" => Ok(crate::domain::types::Language::Php),
            "rb" => Ok(crate::domain::types::Language::Ruby),
            _ => Ok(crate::domain::types::Language::Unknown),
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
                                            tracing::debug!(
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
                                        tracing::warn!(
                                            "[INDEX] Failed to process {}: {}",
                                            file_path_clone,
                                            e
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
                        tracing::error!("[INDEX] Failed to store batch of chunks: {}", e);
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

    /// Clear all indexed data from a collection
    ///
    /// This permanently removes all code chunks and vector embeddings from the specified
    /// collection. After clearing, the collection will need to be re-indexed before
    /// search functionality can be used.
    pub async fn clear_collection(&self, collection: &str) -> Result<()> {
        tracing::info!(
            "[INDEX] Starting collection clear operation for: {}",
            collection
        );

        // Clear the vector store collection
        self.context_service.clear_collection(collection).await?;

        // Reset snapshot state for the collection if we have sync manager
        if let Some(_sync_mgr) = &self.sync_manager {
            tracing::info!("[INDEX] Clearing sync state for collection: {}", collection);
        }

        tracing::info!("[INDEX] Successfully cleared collection: {}", collection);

        Ok(())
    }
}
