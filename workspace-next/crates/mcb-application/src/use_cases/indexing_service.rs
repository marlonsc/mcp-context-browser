//! Indexing Service Use Case
//!
//! Application service for code indexing and ingestion operations.
//! Orchestrates file discovery, chunking, and storage of code embeddings.

use mcb_domain::domain_services::search::{ContextServiceInterface, IndexingServiceInterface};
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use mcb_domain::ports::providers::LanguageChunkingProvider;
use shaku::Component;
use std::path::Path;
use std::sync::Arc;

/// Indexing service implementation - orchestrates file discovery and chunking
#[derive(Component)]
#[shaku(interface = IndexingServiceInterface)]
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
    async fn discover_files(&self, path: &Path, errors: &mut Vec<String>) -> Vec<std::path::PathBuf> {
        use tokio::fs;

        let mut files = Vec::new();
        let mut dirs_to_visit = vec![path.to_path_buf()];

        while let Some(dir_path) = dirs_to_visit.pop() {
            let mut entries = match fs::read_dir(&dir_path).await {
                Ok(entries) => entries,
                Err(e) => {
                    errors.push(format!("Failed to read directory {}: {}", dir_path.display(), e));
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
        !path.ends_with(".git")
            && !path.ends_with("node_modules")
            && !path.ends_with("target")
            && !path.ends_with("__pycache__")
    }

    /// Check if file has a supported extension
    fn is_supported_file(path: &Path) -> bool {
        path.extension()
            .map(|ext| {
                let ext_str = ext.to_string_lossy().to_lowercase();
                matches!(ext_str.as_str(), "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c" | "go")
            })
            .unwrap_or(false)
    }

    /// Chunk file content using intelligent AST-based chunking
    fn chunk_file_content(&self, content: &str, path: &Path) -> Vec<CodeChunk> {
        let file_name = path.to_string_lossy().to_string();
        self.language_chunker.chunk(content, &file_name)
    }
}

#[async_trait::async_trait]
impl IndexingServiceInterface for IndexingServiceImpl {
    async fn index_codebase(
        &self,
        path: &Path,
        collection: &str,
    ) -> Result<mcb_domain::domain_services::search::IndexingResult> {
        use tokio::fs;

        self.context_service.initialize(collection).await?;
        let mut errors = Vec::new();

        // Discover files recursively
        let files_to_process = self.discover_files(path, &mut errors).await;

        // Process files and collect results
        let mut files_processed = 0;
        let mut chunks_created = 0;
        let mut files_skipped = 0;

        for file_path in files_to_process {
            let content = match fs::read_to_string(&file_path).await {
                Ok(c) => c,
                Err(e) => {
                    errors.push(format!("Failed to read {}: {}", file_path.display(), e));
                    files_skipped += 1;
                    continue;
                }
            };

            let chunks = self.chunk_file_content(&content, &file_path);
            if let Err(e) = self.context_service.store_chunks(collection, &chunks).await {
                errors.push(format!("Failed to store chunks for {}: {}", file_path.display(), e));
                continue;
            }
            files_processed += 1;
            chunks_created += chunks.len();
        }

        Ok(mcb_domain::domain_services::search::IndexingResult {
            files_processed,
            chunks_created,
            files_skipped,
            errors,
        })
    }

    fn get_status(&self) -> mcb_domain::domain_services::search::IndexingStatus {
        mcb_domain::domain_services::search::IndexingStatus {
            is_indexing: false,
            progress: 0.0,
            current_file: None,
            total_files: 0,
            processed_files: 0,
        }
    }

    async fn clear_collection(&self, collection: &str) -> Result<()> {
        self.context_service.clear_collection(collection).await
    }
}