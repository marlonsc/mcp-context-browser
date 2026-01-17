# Services Module

**Source**: `crates/mcb-application/src/use_cases/`
**Traits**: `crates/mcb-application/src/ports/`
**Crate**: `mcb-application`

Orchestrates the semantic code search workflow - from codebase ingestion to search results.

## Overview

The services module contains core business logic that powers the semantic code search platform. Each service encapsulates specific capabilities that work together to deliver code intelligence.

All services implement interface traits defined in `crates/mcb-application/src/ports/` for DI compatibility.

## Service Interface Traits

All service interfaces extend `shaku::Interface` (defined in mcb-domain):

```rust
pub trait ContextServiceInterface: Interface + Send + Sync {
    fn initialize(&self) -> impl Future<Output = Result<()>> + Send;
    fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> impl Future<Output = Result<()>> + Send;
    fn search_similar(&self, collection: &str, query: &str, limit: usize) -> impl Future<Output = Result<Vec<SearchResult>>> + Send;
    fn embed_text(&self, text: &str) -> impl Future<Output = Result<Embedding>> + Send;
    fn clear_collection(&self, collection: &str) -> impl Future<Output = Result<()>> + Send;
    fn embedding_dimensions(&self) -> usize;
}

pub trait SearchServiceInterface: Interface + Send + Sync {
    fn search(&self, collection: &str, query: &str, limit: usize) -> impl Future<Output = Result<Vec<SearchResult>>> + Send;
}

pub trait IndexingServiceInterface: Interface + Send + Sync {
    fn index_codebase(&self, path: &Path, collection: &str) -> impl Future<Output = Result<IndexingResult>> + Send;
    fn get_status(&self) -> IndexingStatus;
    fn clear_collection(&self, collection: &str) -> impl Future<Output = Result<()>> + Send;
}

pub trait ChunkingOrchestratorInterface: Interface + Send + Sync {
    fn process_files(&self, files: &[PathBuf], collection: &str) -> impl Future<Output = Result<Vec<CodeChunk>>> + Send;
    fn process_file(&self, path: &Path, collection: &str) -> impl Future<Output = Result<Vec<CodeChunk>>> + Send;
}
```

## Services

### ContextService

Coordinates embedding generation and vector storage operations.

**Location**: `crates/mcb-application/src/use_cases/context_service.rs`

**Constructor**:

```rust
pub fn new_with_providers(
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
) -> Self
```

**Responsibilities**:

1.  Generate embeddings via AI providers
2.  Store and retrieve vectors
3.  Handle batch processing
4.  Collect performance metrics

**Related**: [providers.md](./providers.md), [domain.md](./domain.md)

### IndexingService

Processes codebases and creates searchable vector indexes.

**Location**: `crates/mcb-application/src/use_cases/indexing_service.rs`

**Responsibilities**:

1.  Repository scanning and file discovery
2.  Language detection and AST parsing
3.  Incremental indexing with change detection
4.  Chunk generation and metadata extraction

**Related**: [chunking.md](./chunking.md), [domain.md](./domain.md)

### SearchService

Executes semantic similarity searches across indexed codebases.

**Location**: `crates/mcb-application/src/use_cases/search_service.rs`

**Responsibilities**:

1.  Query processing and embedding generation
2.  Vector similarity search execution
3.  Result ranking and filtering
4.  Response caching and optimization

**Related**: [providers.md](./providers.md)

### ChunkingOrchestrator

Coordinates batch chunking operations across files.

**Location**: `crates/mcb-application/src/domain_services/chunking.rs`

**Responsibilities**:

1.  Process multiple files in parallel
2.  Coordinate with language processors
3.  Handle file batching and error recovery

## Integration Points

### AI Providers

1.  OpenAI, Ollama, Gemini, VoyageAI, FastEmbed
2.  Intelligent routing with failover
3.  See [providers.md](./providers.md)

### Vector Storage

1.  InMemory (development), Encrypted (sensitive data)
2.  See [providers.md](./providers.md)

### MCP Protocol

1.  Standardized interface with AI assistants
2.  See [server.md](./server.md)

## Key Exports

```rust
pub use use_cases::context_service::ContextServiceImpl;
pub use use_cases::indexing_service::IndexingServiceImpl;
pub use use_cases::search_service::SearchServiceImpl;
pub use domain_services::chunking::ChunkingOrchestrator;
```

## File Structure

```text
crates/mcb-application/src/
├── use_cases/
│   ├── context_service.rs      # Embedding and vector operations
│   ├── indexing_service.rs     # Codebase ingestion and processing
│   ├── search_service.rs       # Query processing and ranking
│   └── mod.rs
├── domain_services/
│   ├── chunking.rs             # Batch chunking coordination
│   └── search.rs               # Search domain logic
└── mod.rs

crates/mcb-application/src/ports/    # Service interface traits
```

## Testing

See `crates/mcb-application/tests/` for service-specific tests.

## Cross-References

-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
-   **Domain Ports**: [domain.md](./domain.md)
-   **Providers**: [providers.md](./providers.md)
-   **Server**: [server.md](./server.md)

---

*Updated 2026-01-17 - Reflects modular crate architecture (v0.1.1)*
