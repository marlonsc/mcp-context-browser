# ADR 012: Two-Layer Dependency Injection Strategy

## Status

**Superseded** by [ADR 024: Simplified Dependency Injection](024-simplified-dependency-injection.md) (v0.2.0)

> **DEPRECATED**: This two-layer approach (Shaku + runtime factories) will be simplified to direct constructor injection. The complex Shaku infrastructure will be removed in favor of simpler service composition patterns.
>
> **Code examples** below use `DiContainerBuilder` (removed). Current DI: dill Catalog, handles, linkme — see [ADR-029](029-hexagonal-architecture-dill.md).

**Originally Accepted** (v0.1.2)

## Context

The MCP Context Browser uses Shaku as its dependency injection framework. During the Clean Architecture refactoring (January 2026), we discovered that a pure compile-time DI approach via Shaku modules doesn't fit all our service creation needs.

### The Challenge

Different service categories have different requirements:

1.  **Infrastructure services** (cache, auth, events, metrics):

-   Stateless or simple state
-   Can be instantiated at compile-time with default values
-   Don't require async initialization
-   Have predictable construction parameters

1.  **Application services** (indexing, search, context):

-   Require runtime configuration (API keys, endpoints, model names)
-   Need async initialization (connecting to vector stores, loading models)
-   Have complex dependencies on production providers
-   Construction parameters vary based on configuration

### Why Not Pure Shaku?

Shaku's `module!` macro and `#[derive(Component)]` work well when:

-   All dependencies implement `Default`
-   No async initialization is needed
-   Construction is uniform across environments

However, production providers like `OllamaEmbeddingProvider` or `MilvusVectorStoreProvider` require:

-   Configuration values (URLs, API keys)
-   Async connection establishment
-   Error handling during creation

These don't map cleanly to Shaku's compile-time component model.

## Decision

We adopt a **two-layer DI strategy**:

### Layer 1: Shaku Modules (Infrastructure Defaults)

Shaku modules provide **null implementations** as testing defaults:

```rust
// Infrastructure module provides null adapters
module! {
    pub InfrastructureModuleImpl: InfrastructureModule {
        components = [
            NullAuthService,
            NullEventBus,
            NullSystemMetricsCollector,
        ],
        providers = []
    }
}

// Server module provides null admin components
module! {
    pub ServerModuleImpl: ServerModule {
        components = [
            NullPerformanceMetrics,
            NullIndexingOperations,
        ],
        providers = []
    }
}
```

### Layer 2: Runtime Factories (Production Providers)

Production providers are created at runtime via factories:

```rust
// Production flow in mcb-server/init.rs:

// Step 1: Create Shaku modules (infrastructure defaults)
let app_container = init_app(config.clone()).await?;

// Step 2: Create production providers from configuration
let embedding_provider = EmbeddingProviderFactory::create(&config.embedding, None)?;
let vector_store_provider = VectorStoreProviderFactory::create(&config.vector_store, crypto)?;

// Step 3: Get infrastructure services from Shaku
let cache_provider: Arc<dyn CacheProvider> = app_container.cache.resolve();
let language_chunker: Arc<dyn LanguageChunkingProvider> = app_container.language.resolve();

// Step 4: Create domain services with production providers
let services = DomainServicesFactory::create_services(
    cache,
    crypto,
    config,
    embedding_provider,
    vector_store_provider,
    language_chunker,
).await?;
```

### Why This Works

1.  **Testing**: Tests use Shaku modules directly, getting null providers automatically
2.  **Production**: Server initialization creates real providers from config
3.  **Flexibility**: New providers can be added without changing DI modules
4.  **Clear separation**: Infrastructure (Shaku) vs Application (runtime factories)

## Consequences

### Positive

-   **Clear mental model**: Shaku = defaults, Factories = production
-   **Easy testing**: `DiContainerBuilder::new().build()` gives working test container
-   **Configuration-driven**: Provider selection happens at runtime based on config
-   **Async-friendly**: Factories can perform async initialization

### Negative

-   **Two patterns to understand**: Developers must know when to use each
-   **Not fully type-checked**: Factory selection happens at runtime
-   **Documentation critical**: Without this ADR, the pattern may seem inconsistent

### Neutral

-   **Hybrid approach**: Neither pure compile-time nor pure runtime DI
-   **Migration path**: Can gradually move more to Shaku if Shaku adds async support

## Implementation Notes

### Where to Put What

| Category | Layer | Location |
|----------|-------|----------|
| Null providers | Shaku | `mcb-infrastructure/src/infrastructure/` |
| Production providers | Factory | `mcb-providers/src/` |
| Port traits | Neither | `mcb-application/src/ports/` |
| Domain services | Factory | Created via `DomainServicesFactory` |
| Configuration | Runtime | `mcb-infrastructure/src/config/` |

### Testing Pattern

```rust
#[tokio::test]
async fn test_with_null_providers() {
    // Shaku modules give us null providers automatically
    let container = DiContainerBuilder::new().build().await.unwrap();
    // container has NullCacheProvider, NullEmbeddingProvider, etc.
}
```

### Production Pattern

```rust
pub async fn run_server(config_path: Option<&Path>) -> Result<()> {
    let config = load_config(config_path)?;

    // Layer 1: Shaku infrastructure
    let app_container = init_app(config.clone()).await?;

    // Layer 2: Production providers from config
    let embedding = create_embedding_provider(&config)?;
    let vector_store = create_vector_store_provider(&config)?;

    // Combine layers
    let services = DomainServicesFactory::create_services(...).await?;

    // Run server with production services
    server.start(services).await
}
```

## Migration Notes

**As of v0.2.0, this ADR is being superseded** by [ADR 024: Simplified Dependency Injection](024-simplified-dependency-injection.md).

### Migration Impact

-   **Shaku modules** will be completely removed
-   **Runtime factories** will be replaced with direct constructor injection
-   **Two-layer complexity** will be eliminated in favor of simple service composition
-   **Infrastructure defaults** will be provided through constructor parameters

### Backward Compatibility

The public service interfaces will remain stable. Only the internal composition mechanism will change from complex DI containers to direct dependency passing.

## Related ADRs

-   [ADR-001: Modular Crates Architecture](001-modular-crates-architecture.md) - Trait-based provider DI
-   [ADR-002: Async-First Architecture](002-async-first-architecture.md) - **SUPERSEDED** by [ADR 024](024-simplified-dependency-injection.md)
-   [ADR-030: Multi-Provider Strategy](030-multi-provider-strategy.md) - Provider factory selection
-   [ADR-006: Code Audit and Improvements](006-code-audit-and-improvements.md) - DI pattern enforcement
-   [ADR-007: Integrated Web Administration Interface](007-integrated-web-administration-interface.md) - AdminService DI
-   [ADR-008: Git-Aware Semantic Indexing](008-git-aware-semantic-indexing-v0.2.0.md) - GitProvider factory (v0.2.0)
-   [ADR-009: Persistent Session Memory](009-persistent-session-memory-v0.2.0.md) - MemoryProvider DI (v0.2.0)
-   [ADR-010: Hooks Subsystem](010-hooks-subsystem-agent-backed.md) - HookProcessor DI (v0.2.0)
-   [ADR-013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Crate organization for DI
-   [ADR 024: Simplified Dependency Injection](024-simplified-dependency-injection.md) - **SUPERSEDES THIS ADR**

## References

-   [Shaku Documentation](https://docs.rs/shaku) (historical; see ADR-029)
-   [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
-   Workspace-next refactoring plan (January 2026)
