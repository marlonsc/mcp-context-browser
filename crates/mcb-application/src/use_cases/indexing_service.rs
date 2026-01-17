//! Indexing Service Use Case
//!
//! Application service for code indexing and ingestion operations.
//! Orchestrates file discovery, chunking, and storage of code embeddings.

use crate::domain_services::search::{
    ContextServiceInterface, IndexingResult, IndexingServiceInterface,
};
use crate::ports::providers::LanguageChunkingProvider;
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use std::path::Path;
use std::sync::Arc;

/// Directories to skip during indexing
const SKIP_DIRS: &[&str] = &[".git", "node_modules", "target", "__pycache__"];

/// Supported file extensions for indexing
const SUPPORTED_EXTENSIONS: &[&str] = &["rs", "py", "js", "ts", "java", "cpp", "c", "go"];

/// Accumulator for indexing progress and errors
struct IndexingProgress {
    files_processed: usize,
    chunks_created: usize,
    files_skipped: usize,
    errors: Vec<String>,
}

impl IndexingProgress {
    fn new() -> Self {
        Self {
            files_processed: 0,
            chunks_created: 0,
            files_skipped: 0,
            errors: Vec::new(),
        }
    }

    fn record_error(&mut self, context: &str, path: &Path, error: impl std::fmt::Display) {
        self.errors
            .push(format!("{} {}: {}", context, path.display(), error));
    }

    fn into_result(self) -> IndexingResult {
        IndexingResult {
            files_processed: self.files_processed,
            chunks_created: self.chunks_created,
            files_skipped: self.files_skipped,
            errors: self.errors,
        }
    }
}

/// Indexing service implementation - orchestrates file discovery and chunking
#[derive(shaku::Component)]
#[shaku(interface = crate::domain_services::search::IndexingServiceInterface)]
pub struct IndexingServiceImpl {
    #[shaku(inject)]
    context_service: Arc<dyn ContextServiceInterface>,

    #[shaku(inject)]
    language_chunker: Arc<dyn LanguageChunkingProvider>,
}

impl IndexingServiceImpl {
    /// Create new indexing service with injected dependencies
    pub fn new(
        context_service: Arc<dyn ContextServiceInterface>,
        language_chunker: Arc<dyn LanguageChunkingProvider>,
    ) -> Self {
        Self {
            context_service,
            language_chunker,
        }
    }

    /// Discover files recursively from a path
    async fn discover_files(
        &self,
        path: &Path,
        progress: &mut IndexingProgress,
    ) -> Vec<std::path::PathBuf> {
        use tokio::fs;

        let mut files = Vec::new();
        let mut dirs_to_visit = vec![path.to_path_buf()];

        while let Some(dir_path) = dirs_to_visit.pop() {
            let mut entries = match fs::read_dir(&dir_path).await {
                Ok(entries) => entries,
                Err(e) => {
                    progress.record_error("Failed to read directory", &dir_path, e);
                    continue;
                }
            };

            while let Ok(Some(entry)) = entries.next_entry().await {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    if Self::should_visit_dir(&entry_path) {
                        dirs_to_visit.push(entry_path);
                    }
                } else if Self::is_supported_file(&entry_path) {
                    files.push(entry_path);
                }
            }
        }
        files
    }

    /// Check if directory should be visited during indexing
    fn should_visit_dir(path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| !SKIP_DIRS.contains(&name))
            .unwrap_or(true)
    }

    /// Check if file has a supported extension
    fn is_supported_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    /// Chunk file content using intelligent AST-based chunking
    fn chunk_file_content(&self, content: &str, path: &Path) -> Vec<CodeChunk> {
        self.language_chunker
            .chunk(content, &path.to_string_lossy())
    }
}

#[async_trait::async_trait]
impl IndexingServiceInterface for IndexingServiceImpl {
    async fn index_codebase(&self, path: &Path, collection: &str) -> Result<IndexingResult> {
        use tokio::fs;

        self.context_service.initialize(collection).await?;
        let mut progress = IndexingProgress::new();

        // Discover and process files
        let files = self.discover_files(path, &mut progress).await;

        for file_path in files {
            let content = match fs::read_to_string(&file_path).await {
                Ok(c) => c,
                Err(e) => {
                    progress.record_error("Failed to read", &file_path, e);
                    progress.files_skipped += 1;
                    continue;
                }
            };

            let chunks = self.chunk_file_content(&content, &file_path);
            if let Err(e) = self.context_service.store_chunks(collection, &chunks).await {
                progress.record_error("Failed to store chunks for", &file_path, e);
                continue;
            }
            progress.files_processed += 1;
            progress.chunks_created += chunks.len();
        }

        Ok(progress.into_result())
    }

    fn get_status(&self) -> crate::domain_services::search::IndexingStatus {
        crate::domain_services::search::IndexingStatus::default()
    }

    async fn clear_collection(&self, collection: &str) -> Result<()> {
        self.context_service.clear_collection(collection).await
    }
}
