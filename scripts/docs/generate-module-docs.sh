#!/bin/bash

# MCP Context Browser - Automatic Module Documentation Generation
# Generates documentation from source code analysis

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_header() {
    echo -e "${PURPLE}[DOCS]${NC} $1"
}

# Extract module documentation from source files
extract_module_docs() {
    local module_path="$1"
    local module_name="$2"

    log_info "Extracting documentation for $module_name..."

    # Create output directory
    local output_dir="$PROJECT_ROOT/docs/modules"
    mkdir -p "$output_dir"

    # Extract main module doc comment
    local main_doc=""
    if [ -f "$module_path/mod.rs" ]; then
        main_doc=$(grep -A 10 "^//! " "$module_path/mod.rs" | sed 's|^//! ||' | sed '/^$/q' | head -10)
    fi

    local rs_files total_lines traits structs enums functions exports
    rs_files=$(find "$module_path" -name "*.rs" -type f | grep -c . || true)
    total_lines=$(find "$module_path" -name "*.rs" -type f -exec wc -l {} \; | awk '{sum += $1} END {print sum}')
    traits=$(grep -h "^pub trait " "$module_path"/*.rs 2>/dev/null | grep -c . || true)
    structs=$(grep -h "^pub struct " "$module_path"/*.rs 2>/dev/null | grep -c . || true)
    enums=$(grep -h "^pub enum " "$module_path"/*.rs 2>/dev/null | grep -c . || true)
    functions=$(grep -h "^pub fn " "$module_path"/*.rs 2>/dev/null | grep -c . || true)
    exports=""
    if [ -f "$module_path/mod.rs" ]; then
        exports=$(grep "^pub use " "$module_path/mod.rs" | sed 's/pub use //' | sed 's/;//' | tr '\n' ', ' | sed 's/, $//')
    fi

    # Generate markdown
    cat > "$output_dir/$module_name.md" << EOF
# $module_name Module

**Source**: \`src/$module_name/\`
**Files**: $rs_files
**Lines of Code**: $total_lines
**Traits**: $traits
**Structs**: $structs
**Enums**: $enums
**Functions**: $functions

## Overview

$main_doc

## Key Exports

\`$exports\`

## File Structure

\`\`\`text
$(find "$module_path" -name "*.rs" -type f | sed "s|$PROJECT_ROOT/src/$module_name/||")
\`\`\`

---

*Auto-generated from source code on $(date)*
EOF

    log_success "Generated docs for $module_name"
}

# Generate API reference
generate_api_reference() {
    log_header "Generating API Reference"

    local output_file="$PROJECT_ROOT/docs/api-reference.md"

    cat > "$output_file" << 'EOF'
# API Reference

This document provides a comprehensive reference of the MCP Context Browser public API.

## Table of Contents

- [Core Types](#core-types)
- [Providers](#providers)
- [Services](#services)
- [Utilities](#utilities)

## Core Types

### Embedding

```rust
pub struct Embedding {
    pub vector: Vec<f32>,
    pub dimensions: usize,
    pub model: String,
    pub provider: String,
}
```

Vector representation of text with metadata.

### SearchResult

```rust
pub struct SearchResult {
    pub content: String,
    pub score: f32,
    pub metadata: HashMap<String, serde_json::Value>,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
}
```

Search result with relevance score and source location.

### CodeChunk

```rust
pub struct CodeChunk {
    pub content: String,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub language: Language,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

Parsed code chunk with location and language information.

## Providers

### EmbeddingProvider Trait

```rust
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;
    async fn health_check(&self) -> Result<()>;
}
```

Interface for text-to-vector conversion providers.

### VectorStoreProvider Trait

```rust
#[async_trait]
pub trait VectorStoreProvider: Send + Sync {
    async fn create_collection(&self, name: &str, dimensions: usize) -> Result<()>;
    async fn delete_collection(&self, name: &str) -> Result<()>;
    async fn collection_exists(&self, name: &str) -> Result<bool>;
    async fn insert_vectors(&self, collection: &str, vectors: &[Embedding], metadata: Vec<HashMap<String, serde_json::Value>>) -> Result<Vec<String>>;
    async fn search_similar(&self, collection: &str, query_vector: &[f32], limit: usize, filter: Option<&str>) -> Result<Vec<SearchResult>>;
    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()>;
    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, serde_json::Value>>;
    async fn flush(&self, collection: &str) -> Result<()>;
    fn provider_name(&self) -> &str;
    async fn health_check(&self) -> Result<()>;
}
```

Interface for vector storage and retrieval.

## Services

### ContextService

```rust
pub struct ContextService {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
}

impl ContextService {
    pub async fn embed_text(&self, text: &str) -> Result<Embedding>;
    pub async fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()>;
    pub async fn search_similar(&self, collection: &str, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
}
```

Orchestrates embedding generation and vector operations.

### IndexingService

```rust
pub struct IndexingService {
    context_service: Arc<ContextService>,
}

impl IndexingService {
    pub fn new(context_service: Arc<ContextService>) -> Self;
    pub async fn index_directory(&self, path: &Path, collection: &str) -> Result<usize>;
}
```

Handles codebase indexing and chunking.

### SearchService

```rust
pub struct SearchService {
    context_service: Arc<ContextService>,
}

impl SearchService {
    pub fn new(context_service: Arc<ContextService>) -> Self;
    pub async fn search(&self, collection: &str, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
}
```

Provides semantic search capabilities.

## Utilities

### Metrics

```rust
pub struct SystemMetricsCollector {
    pub fn collect_cpu_metrics(&mut self) -> CpuMetrics;
    pub fn collect_memory_metrics(&mut self) -> MemoryMetrics;
}

pub struct MetricsApiServer {
    pub async fn start(&self, addr: SocketAddr) -> Result<()>;
}
```

System monitoring and metrics collection.

### Sync

```rust
pub struct CodebaseLockManager;
impl CodebaseLockManager {
    pub async fn acquire_lock(&self, codebase_path: &str) -> Result<LockMetadata>;
    pub async fn release_lock(&self, lock_id: &str) -> Result<()>;
    pub async fn cleanup_stale_locks(&self) -> Result<usize>;
}

pub struct SyncManager {
    pub async fn sync_operation<F, Fut>(&self, operation: F) -> Result<()>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<()>>;
}
```

Cross-process synchronization utilities.

### Routing

```rust
pub struct ProviderRouter {
    pub async fn select_embedding_provider(&self, context: &ProviderContext) -> Result<String>;
    pub async fn get_embedding_provider(&self, context: &ProviderContext) -> Result<Arc<dyn EmbeddingProvider>>;
}

pub struct CircuitBreaker {
    pub async fn call<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>;
}
```

Intelligent provider routing with resilience.

---

*Auto-generated API reference*
EOF

    log_success "Generated API reference"
}

# Generate implementation status
generate_implementation_status() {
    log_header "Generating Implementation Status"

    local output_file="$PROJECT_ROOT/docs/implementation-status.md"
    local embedding_providers vector_providers routing_modules core_modules

    embedding_providers=$(find "$PROJECT_ROOT/src/adapters/providers/embedding" -maxdepth 1 -name "*.rs" ! -name "mod.rs" 2>/dev/null | grep -c . || true)
    vector_providers=$(find "$PROJECT_ROOT/src/adapters/providers/vector_store" -maxdepth 1 -name "*.rs" ! -name "mod.rs" 2>/dev/null | grep -c . || true)
    routing_modules=$(find "$PROJECT_ROOT/src/adapters/providers/routing" -maxdepth 1 -type f 2>/dev/null | grep -c . || true)
    core_modules=$(find "$PROJECT_ROOT/src/core" -maxdepth 1 -type f 2>/dev/null | grep -c . || true)

    cat > "$output_file" << EOF
# Implementation Status

**Last Updated**: $(date)
**Version**: $(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/')

## 📊 Implementation Metrics

- **Core Modules**: $core_modules
- **Embedding Providers**: $embedding_providers
- **Vector Store Providers**: $vector_providers
- **Routing Modules**: $routing_modules
- **Total Source Files**: $(find "$PROJECT_ROOT/src" -name "*.rs" -type f | wc -l)
- **Lines of Code**: $(find "$PROJECT_ROOT/src" -name "*.rs" -type f -exec wc -l {} \; | awk '{sum += $1} END {print sum}')

## ✅ Fully Implemented

### Core Infrastructure
- [x] Error handling system
- [x] Configuration management
- [x] Logging and tracing
- [x] HTTP client utilities
- [x] Resource limits
- [x] Rate limiting
- [x] Caching system
- [x] Database connection pooling

### Provider System
- [x] Provider trait abstractions
- [x] Registry system
- [x] Factory pattern
- [x] Health checking
- [x] Circuit breaker protection
- [x] Cost tracking
- [x] Failover management

### Services Layer
- [x] Context service orchestration
- [x] Indexing service
- [x] Search service
- [x] MCP protocol handlers

### Advanced Features
- [x] Hybrid search (BM25 + semantic)
- [x] Intelligent chunking
- [x] Cross-process synchronization
- [x] Background daemon
- [x] Metrics collection
- [x] System monitoring

## 🚧 Partially Implemented

### Providers
- [x] OpenAI embeddings (complete)
- [x] Ollama embeddings (complete)
- [x] Gemini embeddings (complete)
- [x] VoyageAI embeddings (complete)
- [x] Milvus vector store (complete)
- [x] In-memory vector store (complete)
- [x] Filesystem vector store (basic)
- [x] Encrypted vector store (basic)

### Server Components
- [x] MCP stdio transport (complete)
- [x] HTTP API server (basic)
- [x] Metrics HTTP endpoint (complete)
- [x] WebSocket support (planned)

## 📋 Planned Features

### Provider Expansions
- [ ] Anthropic embeddings
- [ ] Pinecone vector store
- [ ] Qdrant vector store
- [ ] Redis vector store

### Enterprise Features
- [ ] Multi-tenant isolation
- [ ] Advanced authentication
- [ ] Audit logging
- [ ] Backup and recovery

### Performance Optimizations
- [ ] Query result caching
- [ ] Batch processing improvements
- [ ] Memory optimization
- [ ] Concurrent indexing

---

*Auto-generated implementation status*
EOF

    log_success "Generated implementation status"
}

# Main execution
main() {
    log_header "🔧 MCP Context Browser - Module Documentation Generation"
    echo "======================================================="

    # Create output directory
    mkdir -p "$PROJECT_ROOT/docs/modules"

    local module_name
    for module_path in "$PROJECT_ROOT/src"/*; do
        if [ -d "$module_path" ]; then
            module_name=$(basename "$module_path")
            extract_module_docs "$module_path" "$module_name"
        fi
    done

    # Generate API reference
    generate_api_reference

    # Generate implementation status
    generate_implementation_status

    log_success "Module documentation generation completed"
    log_info "Generated files in docs/modules/"
    log_info "Generated API reference at docs/api-reference.md"
    log_info "Generated implementation status at docs/implementation-status.md"
}

# Run main function
main "$@"