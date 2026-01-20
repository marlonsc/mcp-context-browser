# ADR 029: Hexagonal Architecture with dill IoC

## Status

**Implemented** (v0.1.2)

> Evolution of [ADR 024: Simplified Dependency Injection](024-simplified-dependency-injection.md), adding dill Catalog as IoC container while maintaining the handle-based pattern.

## Context

The previous architecture (ADR-024) used a handle-based DI pattern with linkme registry for compile-time provider discovery. While effective, this approach had coupling issues:

1.  **Infrastructure imported concrete types from Application**
    -   `domain_services.rs` imported `ContextServiceImpl`, `SearchServiceImpl`

2.  **Application ports were duplicated**
    -   `mcb-domain/src/ports/providers/` (correct location)
    -   `mcb-application/src/ports/providers/` (duplication)

3.  **No IoC container for service lifecycle management**
    -   Manual wiring in bootstrap.rs
    -   No dependency graph validation

## Decision

We implement a proper hexagonal architecture with dill IoC container:

### 1. Ports in mcb-domain (Single Source of Truth)

All provider ports are defined in `mcb-domain/src/ports/providers/`:

```rust
// mcb-domain/src/ports/providers/embedding.rs
pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Result<Embedding>;
}

// mcb-domain/src/ports/providers/vector_store.rs
pub trait VectorStoreProvider: Send + Sync {
    fn store(&self, embedding: &Embedding) -> Result<()>;
    fn search(&self, query: &Embedding) -> Result<Vec<SearchResult>>;
}
```

Application layer re-exports for backward compatibility:

```rust
// mcb-application/src/ports/providers/mod.rs
pub use mcb_domain::ports::providers::*;
```

### 2. dill Catalog as IoC Container

The dill `Catalog` manages service registration and resolution:

```rust
// mcb-infrastructure/src/di/catalog.rs
pub async fn build_catalog(config: AppConfig) -> Result<Catalog> {
    CatalogBuilder::new()
        // Configuration
        .add_value(config)
        // Providers (from linkme registry)
        .add_value(embedding_provider)
        .add_value(vector_store_provider)
        // Handles (for runtime switching)
        .add_value(embedding_handle)
        .add_value(vector_store_handle)
        // Admin services
        .add_value(embedding_admin)
        .add_value(vector_store_admin)
        .build()
}

// Service retrieval
pub fn get_embedding_provider(catalog: &Catalog) -> Result<Arc<dyn EmbeddingProvider>> {
    catalog.get_one::<dyn EmbeddingProvider>()
        .map_err(|e| Error::configuration(format!("Service not found: {e:?}")))
}
```

### 3. Architecture Layers

```text
┌──────────────────────────────────────────────────────────────┐
│                        mcb-domain                             │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ PORTS (trait definitions)                              │  │
│  │   - EmbeddingProvider                                  │  │
│  │   - VectorStoreProvider                                │  │
│  │   - CacheProvider                                      │  │
│  │   - LanguageChunkingProvider                           │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                              ↑
┌──────────────────────────────────────────────────────────────┐
│                      mcb-application                          │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ USE CASES (import ports from mcb-domain)               │  │
│  │   - ContextServiceImpl                                 │  │
│  │   - SearchServiceImpl                                  │  │
│  │   - IndexingServiceImpl                                │  │
│  └────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ REGISTRY (linkme distributed slices)                   │  │
│  │   - EMBEDDING_PROVIDERS                                │  │
│  │   - VECTOR_STORE_PROVIDERS                             │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                              ↑
┌──────────────────────────────────────────────────────────────┐
│                    mcb-infrastructure                         │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ COMPOSITION ROOT (dill Catalog)                        │  │
│  │   - build_catalog() creates IoC container              │  │
│  │   - Provider Handles for runtime switching             │  │
│  │   - Admin Services for API-based management            │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                              ↑
┌──────────────────────────────────────────────────────────────┐
│                      mcb-providers                            │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ ADAPTERS (implement ports from mcb-domain)             │  │
│  │   - OllamaEmbeddingProvider                            │  │
│  │   - MilvusVectorStore                                  │  │
│  │   - MokaCacheProvider                                  │  │
│  │   - Register via linkme distributed slices             │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### 4. Validation Rules

New mcb-validate rules enforce the architecture:

| Rule ID | Description |
|---------|-------------|
| CA007 | Infrastructure cannot import concrete types from Application |
| CA008 | Application must import ports from mcb-domain |

## Consequences

### Positive

1.  **Clear layer separation**: Ports in domain, implementations in providers
2.  **IoC container benefits**: dill Catalog manages service lifecycle
3.  **Gradual migration**: `add_value()` allows mixing with existing pattern
4.  **Compile-time validation**: mcb-validate enforces architecture
5.  **Runtime switching**: Provider handles still support admin API

### Negative

1.  **Additional dependency**: dill crate added to workspace
2.  **Learning curve**: Developers must understand dill API
3.  **Migration effort**: Existing code updated to new import paths

### Neutral

1.  **Bootstrap still exists**: `init_app()` wraps `build_catalog()`
2.  **AppContext unchanged**: Same public interface for consumers

## References

-   [dill-rs Documentation](https://docs.rs/dill/latest/dill/)
-   [ADR 023: Inventory to linkme Migration](023-inventory-to-linkme-migration.md)
-   [ADR 024: Simplified Dependency Injection](024-simplified-dependency-injection.md)
-   [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
