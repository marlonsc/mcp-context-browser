# Providers Module

**Source**: `crates/mcb-providers/src/`
**Crate**: `mcb-providers`

Extensible provider system for AI embedding, vector storage, caching, and language processing services.

## Overview

The providers module implements a trait-based abstraction layer for AI and storage services. This enables flexible deployment with multiple providers, intelligent routing, and automatic failover.

## Provider Categories

### Embedding Providers (`embedding/`)

Transform text into vector embeddings.

| Provider | Model | Dimensions | Use Case |
|----------|-------|------------|----------|
| OpenAI | text-embedding-3-small | 1536 | Production |
| Ollama | nomic-embed-text | 768 | Self-hosted |
| Gemini | text-embedding-004 | 768 | Alternative |
| VoyageAI | voyage-3-lite | 512 | Specialized |
| FastEmbed | local models | varies | Privacy-first |
| Null | fixed vectors | 128 | Testing |

**Trait** (defined in `mcb-domain`):

```rust
#[async_trait]
pub trait EmbeddingProvider: Send + Sync + shaku::Interface {
    async fn embed(&self, text: &str) -> Result<Embedding>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;
}
```

### Vector Store Providers (`vector_store/`)

Store and search vector embeddings.

| Provider | Storage | Use Case |
|----------|---------|----------|
| InMemory | RAM | Development/testing |
| Encrypted | Encrypted files | Sensitive data |
| Null | No-op | Testing |

**Trait** (defined in `mcb-domain`):

```rust
#[async_trait]
pub trait VectorStoreProvider: Send + Sync + shaku::Interface {
    async fn store(&self, collection: &str, embeddings: &[Embedding]) -> Result<()>;
    async fn search(&self, collection: &str, query: &[f32], limit: usize) -> Result<Vec<SearchResult>>;
    async fn delete_collection(&self, collection: &str) -> Result<()>;
    fn provider_name(&self) -> &str;
}
```

### Cache Providers (`cache/`)

Caching layer implementations.

| Provider | Backend | Use Case |
|----------|---------|----------|
| Moka | In-memory LRU | Default, high-performance |
| Redis | Redis server | Distributed caching |

### Language Processors (`language/`)

AST-based code chunking for 12 programming languages.

| Language | Parser | Status |
|----------|--------|--------|
| Rust | tree-sitter-rust | Production |
| Python | tree-sitter-python | Production |
| JavaScript | tree-sitter-javascript | Production |
| TypeScript | tree-sitter-typescript | Production |
| Go | tree-sitter-go | Production |
| Java | tree-sitter-java | Production |
| C | tree-sitter-c | Production |
| C++ | tree-sitter-cpp | Production |
| C# | tree-sitter-c-sharp | Production |
| Ruby | tree-sitter-ruby | Production |
| PHP | tree-sitter-php | Production |
| Swift | tree-sitter-swift | Production |
| Kotlin | tree-sitter-kotlin-ng | Production |

### Routing System (`routing/`)

Intelligent provider selection and management.

-   **CircuitBreaker** - Failure detection and recovery
-   **HealthMonitor** - Continuous health checking

## File Structure

```text
crates/mcb-providers/src/
├── embedding/
│   ├── fastembed.rs    # Local embeddings (feature-gated)
│   ├── gemini.rs       # Google Gemini
│   ├── mod.rs          # Module exports
│   ├── null.rs         # Mock provider
│   ├── ollama.rs       # Self-hosted
│   ├── openai.rs       # OpenAI API
│   └── voyageai.rs     # VoyageAI
├── vector_store/
│   ├── encrypted.rs    # AES-GCM encrypted (feature-gated)
│   ├── memory.rs       # In-memory store
│   ├── mod.rs          # Module exports
│   └── null.rs         # No-op provider
├── cache/
│   ├── moka.rs         # Moka cache (feature-gated)
│   ├── redis.rs        # Redis cache (feature-gated)
│   └── mod.rs          # Module exports
├── language/
│   ├── rust.rs         # Rust processor
│   ├── python.rs       # Python processor
│   ├── javascript.rs   # JavaScript processor
│   ├── typescript.rs   # TypeScript processor
│   ├── go.rs           # Go processor
│   ├── java.rs         # Java processor
│   ├── c.rs            # C processor
│   ├── cpp.rs          # C++ processor
│   ├── csharp.rs       # C# processor
│   ├── ruby.rs         # Ruby processor
│   ├── php.rs          # PHP processor
│   ├── swift.rs        # Swift processor
│   ├── kotlin.rs       # Kotlin processor
│   └── mod.rs          # Module exports
├── routing/
│   ├── circuit_breaker.rs
│   ├── health.rs
│   └── mod.rs
├── admin/
│   └── metrics.rs      # AtomicPerformanceMetrics
└── lib.rs              # Crate root
```

## Feature Flags

Providers are controlled via Cargo.toml features:

| Feature | Default | Description |
|---------|---------|-------------|
| `embedding-ollama` | Yes | Ollama embedding provider |
| `embedding-openai` | No | OpenAI embedding provider |
| `embedding-fastembed` | No | FastEmbed local embeddings |
| `vectorstore-memory` | Yes | In-memory vector store |
| `vectorstore-encrypted` | No | AES-GCM encrypted store |
| `cache-moka` | Yes | Moka cache provider |
| `cache-redis` | No | Redis cache provider |
| `lang-all` | Yes | All 12 language processors |

## Testing

Provider tests are located in `crates/mcb-providers/tests/`.

## Cross-References

-   **Domain Ports**: [domain.md](./domain.md) (trait definitions)
-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
-   **Module Structure**: [module-structure.md](./module-structure.md)

---

*Updated 2026-01-17 - Reflects modular crate architecture (v0.1.1)*
