//! Application DI Module Implementation
//!
//! Contains business logic services (ContextService, SearchService, IndexingService).

//!
//! ## Service Hierarchy
//!
//! Services depend on repositories from AdaptersModule:
//! - ContextService injects ChunkRepository, SearchRepository, EmbeddingProvider
//! - SearchService injects ContextServiceInterface
//! - IndexingService injects ContextServiceInterface, ChunkingOrchestratorInterface
//!
//! ## Module Dependencies
//!
//! ApplicationModule uses AdaptersModule as a submodule to provide:
//! - ChunkRepository (for ContextService)
//! - SearchRepository (for ContextService)
//! - EmbeddingProvider (for ContextService)

use shaku::module;

use super::traits::{AdaptersModule, ApplicationModule};
use crate::application::context::ContextService;
use crate::application::indexing::{ChunkingOrchestrator, IndexingService};
use crate::application::search::SearchService;
use crate::domain::chunking::IntelligentChunker;
use crate::domain::ports::{ChunkRepository, EmbeddingProvider, SearchRepository};

// Implementation of the ApplicationModule trait providing business logic services.
// This module provides the core application services with dependencies on adapters.
//
// Generated components:
// - `ContextService`: Main intelligence service combining embeddings and search
// - `SearchService`: Semantic code search functionality
// - `IndexingService`: Codebase indexing orchestration
// - `ChunkingOrchestrator`: AST-based code chunking coordination
// - `IntelligentChunker`: Tree-sitter based code chunking engine
//
// Dependencies (from AdaptersModule):
// - `ChunkRepository`: Storage and retrieval of code chunks
// - `SearchRepository`: Semantic search operations
// - `EmbeddingProvider`: Text-to-vector embedding generation
module! {
    pub ApplicationModuleImpl: ApplicationModule {
        components = [
            ContextService,
            SearchService,
            IndexingService,
            ChunkingOrchestrator,
            IntelligentChunker
        ],
        providers = [],

        use dyn AdaptersModule {
            components = [dyn ChunkRepository, dyn SearchRepository, dyn EmbeddingProvider],
            providers = []
        }
    }
}
