# ADR 024: Shaku to dill DI Migration

## Status

**Implemented** (v0.1.2)

> Replacement for [ADR 002: Dependency Injection with Shaku](002-dependency-injection-shaku.md) using a handle-based DI pattern with linkme registry.
>
> **Implementation Note (2026-01-19)**: The dill `#[component]` macro is incompatible with our domain error types and manual constructors. We use a handle-based pattern instead: Provider Handles (RwLock wrappers), Resolvers (linkme registry), and Admin Services (runtime switching via API).

## Context

The current dependency injection system uses Shaku (version 0.6), a compile-time DI container that provides trait-based dependency resolution. While effective, this approach introduces substantial complexity that impacts development velocity and maintainability.

### Problems with Shaku

1. **Macro complexity**: `#[derive(Component)]`, `#[shaku(interface = ...)]`, `#[shaku(inject)]` everywhere
2. **Build time impact**: Extensive macro expansion slows compilation
3. **Module sync**: Manual maintenance of module definitions as services change
4. **Over-engineering**: DI container complexity exceeds project needs

### DI Library Research

We evaluated modern Rust DI alternatives:

| Library | Type | Cross-Crate | Async | Verdict |
|---------|------|-------------|-------|---------|
| **Shaku** (current) | Compile-time | Yes | No | High boilerplate |
| **nject** | Compile-time | **NO** | No | Rejected (cross-crate limitation) |
| **dill** | Runtime | Yes | Tokio | Partial use |
| Manual injection | N/A | N/A | N/A | **SELECTED** (with patterns) |

### Why Handle-Based Pattern

After implementing the dill catalog approach, we discovered that `dill::Catalog::get_one()` doesn't work well with `add_value` for interface resolution. Instead, we adopted a handle-based pattern that provides:

1. **Runtime provider switching** via RwLock handles
2. **Compile-time discovery** via linkme distributed slices
3. **Admin API support** for provider management
4. **Direct service storage** for infrastructure components

## Decision

We replace Shaku-based DI with a handle-based pattern:

1. **Provider Handles**: RwLock wrappers allowing runtime provider switching
2. **Provider Resolvers**: Components that access the linkme registry
3. **Admin Services**: API endpoints for switching providers at runtime
4. **Direct Storage**: Infrastructure services stored directly in AppContext

### Architecture Overview

```text
AppConfig → Resolvers → Handles (RwLock) → Domain Services
               ↑              ↑
           linkme         AdminServices
          registry       (switch via API)
```

### Implementation Pattern

**Provider Handle (RwLock wrapper for runtime switching):**

```rust
// crates/mcb-infrastructure/src/di/handles.rs

pub struct EmbeddingProviderHandle {
    inner: RwLock<Arc<dyn EmbeddingProvider>>,
}

impl EmbeddingProviderHandle {
    pub fn new(provider: Arc<dyn EmbeddingProvider>) -> Self {
        Self { inner: RwLock::new(provider) }
    }

    pub fn get(&self) -> Arc<dyn EmbeddingProvider> {
        self.inner.read().expect("lock poisoned").clone()
    }

    pub fn set(&self, new_provider: Arc<dyn EmbeddingProvider>) {
        *self.inner.write().expect("lock poisoned") = new_provider;
    }
}
```

**Provider Resolver (linkme registry access):**

```rust
// crates/mcb-infrastructure/src/di/provider_resolvers.rs

pub struct EmbeddingProviderResolver {
    config: Arc<AppConfig>,
}

impl EmbeddingProviderResolver {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    pub fn resolve_from_config(&self) -> Result<Arc<dyn EmbeddingProvider>, String> {
        let registry_config = /* extract from config */;
        resolve_embedding_provider(&registry_config)  // linkme registry
    }

    pub fn resolve_from_override(&self, config: &EmbeddingProviderConfig) -> Result<Arc<dyn EmbeddingProvider>, String> {
        resolve_embedding_provider(config)
    }

    pub fn list_available(&self) -> Vec<(&'static str, &'static str)> {
        list_embedding_providers()  // linkme registry
    }
}
```

**Admin Service (runtime provider switching via API):**

```rust
// crates/mcb-infrastructure/src/di/admin.rs

pub struct EmbeddingAdminService {
    resolver: Arc<EmbeddingProviderResolver>,
    handle: Arc<EmbeddingProviderHandle>,
}

impl EmbeddingAdminService {
    pub fn list_providers(&self) -> Vec<ProviderInfo> {
        self.resolver.list_available()
            .into_iter()
            .map(|(name, desc)| ProviderInfo { name: name.to_string(), description: desc.to_string() })
            .collect()
    }

    pub fn current_provider(&self) -> String {
        self.handle.provider_name()
    }

    pub fn switch_provider(&self, config: EmbeddingProviderConfig) -> Result<(), String> {
        let new_provider = self.resolver.resolve_from_override(&config)?;
        self.handle.set(new_provider);
        Ok(())
    }
}
```

**AppContext (Composition Root):**

```rust
// crates/mcb-infrastructure/src/di/bootstrap.rs

pub struct AppContext {
    pub config: Arc<AppConfig>,

    // Provider Handles (runtime-swappable)
    embedding_handle: Arc<EmbeddingProviderHandle>,
    vector_store_handle: Arc<VectorStoreProviderHandle>,
    cache_handle: Arc<CacheProviderHandle>,
    language_handle: Arc<LanguageProviderHandle>,

    // Admin Services (switch providers via API)
    embedding_admin: Arc<EmbeddingAdminService>,
    vector_store_admin: Arc<VectorStoreAdminService>,
    cache_admin: Arc<CacheAdminService>,
    language_admin: Arc<LanguageAdminService>,

    // Infrastructure Services (direct storage)
    auth_service: Arc<dyn AuthServiceInterface>,
    event_bus: Arc<dyn EventBusProvider>,
    // ... more infrastructure services
}

impl AppContext {
    // Provider access via handles
    pub fn embedding_handle(&self) -> Arc<EmbeddingProviderHandle> {
        self.embedding_handle.clone()
    }

    // Admin service access
    pub fn embedding_admin(&self) -> Arc<EmbeddingAdminService> {
        self.embedding_admin.clone()
    }

    // Infrastructure service access
    pub fn auth(&self) -> Arc<dyn AuthServiceInterface> {
        self.auth_service.clone()
    }
}
```

### Usage Example

```rust
// Create AppContext with provider handles
let context = init_app(AppConfig::default()).await?;

// Get provider via handle (current provider)
let embedding = context.embedding_handle().get();
let embeddings = embedding.embed_batch(&texts).await?;

// Switch provider at runtime via admin API
let admin = context.embedding_admin();
admin.switch_provider(EmbeddingProviderConfig::new("openai"))?;

// Subsequent calls use new provider
let embedding = context.embedding_handle().get();  // Now OpenAI
```

## Consequences

### Positive

- **Runtime switching**: Providers can be changed without restart
- **Admin API ready**: Built-in support for provider management endpoints
- **Type-safe**: All trait bounds enforced at compile time
- **Testable**: Handles and resolvers can be mocked independently
- **Simple**: No complex DI macros or catalog resolution

### Negative

- **Manual wiring**: Services must be explicitly constructed in bootstrap.rs
- **Boilerplate**: Each provider type needs Handle, Resolver, AdminService
- **Lock overhead**: RwLock adds minimal runtime overhead

## Validation Criteria

- [x] All provider types have Handle, Resolver, AdminService
- [x] AppContext provides access to all services
- [x] Runtime provider switching works via admin services
- [x] All tests pass
- [x] No Shaku references remain in production code
- [x] Domain services use providers via handles

## Implementation Summary (2026-01-19)

| Component | Pattern | Status |
|-----------|---------|--------|
| EmbeddingProvider | Handle + Resolver + AdminService | Implemented |
| VectorStoreProvider | Handle + Resolver + AdminService | Implemented |
| CacheProvider | Handle + Resolver + AdminService | Implemented |
| LanguageChunkingProvider | Handle + Resolver + AdminService | Implemented |
| Infrastructure services | Direct storage in AppContext | Implemented |
| Shaku removal | All macros removed | Completed |
| linkme registry | Function pointers (not closures) | Implemented |

### File Structure

```
crates/mcb-infrastructure/src/di/
├── admin.rs           # Admin services for runtime switching
├── bootstrap.rs       # Composition root (AppContext, init_app)
├── dispatch.rs        # Dispatch utilities
├── handles.rs         # RwLock provider handles
├── mod.rs             # Module exports
├── modules/           # Domain services factory
├── provider_resolvers.rs  # Linkme registry access
└── resolver.rs        # Provider resolution utilities
```

## Related ADRs

- [ADR 002: Dependency Injection with Shaku](002-dependency-injection-shaku.md) - **SUPERSEDED** by this ADR
- [ADR 012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) - **SUPERSEDED**
- [ADR 013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Multi-crate organization

## References

- [linkme crate](https://docs.rs/linkme) - Compile-time distributed slices for provider registration
- [dill-rs GitHub](https://github.com/sergiimk/dill-rs) - Evaluated but `add_value` pattern insufficient
