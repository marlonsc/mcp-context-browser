# MCP Context Browser - Crates Separation Plan

Version: 3.0
Status: **Phase 1 Complete** (workspace-next compiles)
Owner: Engineering
Goal: Separate monolith into maintainable crates while preserving 0.1.0 API parity.

---

## Current State (2026-01-16)

### What Works

- **workspace-next/** compiles successfully with 4 crates
- All library code builds (0 warnings, 0 errors)
- Clean Architecture layers properly separated
- DI container wiring functional with config-based provider selection
- All warnings cleaned up
- All tests pass (121+ tests)
- mcb-validate architecture validation passes (0 violations)
- Public API properly exported via mcb facade crate

### What Needs Work

- Phase 4: Cutover (replace root workspace with workspace-next)

---

## 1) Implemented Crate Structure (4 Crates)

The original plan proposed 15+ crates. After implementation, we found **4 crates** is the right level of granularity for this project size:

```
workspace-next/crates/
├── mcb/                    # Facade (re-exports public API)
├── mcb-domain/             # Ports, entities, value objects, errors
├── mcb-infrastructure/     # Services, providers, DI, config, cache, crypto
└── mcb-server/             # MCP protocol, handlers, transport
```

### Why Not More Crates?

- 15+ crates adds maintenance overhead without proportional benefit
- Internal module organization handles separation of concerns
- Cross-crate imports add complexity and slow compilation
- The monolith already had clean internal boundaries

### Dependency Graph

```
mcb-domain → (nothing)
mcb-infrastructure → mcb-domain
mcb-server → mcb-domain + mcb-infrastructure
mcb → re-exports all
```

---

## 2) Clean Architecture Layers (Implemented)

### mcb-domain (Inner Layer)

Contains:
- **ports/**: Trait definitions for all interfaces
- **entities/**: CodeChunk, and domain entities
- **value_objects/**: Embedding, SearchResult, configs
- **error.rs**: Domain error types with variants:
  - `Io`, `IoSimple`, `Cache`, `Infrastructure`
  - `Configuration`, `Authentication`, `Network`, `Database`
  - `Validation`, `NotFound`, `Search`, `Embedding`, `VectorStore`
- **domain_services/**: Service interface traits

Dependencies: Only `serde`, `thiserror`, `async-trait`, `chrono`

### mcb-infrastructure (Middle Layer)

Contains:
- **di/**: Dependency injection (bootstrap, dispatch, factory, modules)
- **cache/**: Cache providers (Moka, Redis, Null), queue, factory
- **config/**: Configuration loading, types, providers
- **crypto/**: AES-GCM encryption, password hashing, token generation
- **health/**: Health check registry and checkers
- **logging/**: Structured logging with tracing

Dependencies: `mcb-domain` + infrastructure crates (redis, moka, aes-gcm, etc.)

### mcb-server (Outer Layer)

Contains:
- **mcp_server.rs**: Main server struct
- **handlers/**: Tool handlers (index_codebase, search_code, etc.)
- **transport/**: Stdio transport
- **tools/**: Tool registry and router
- **admin/**: Admin API (stub modules)
- **init.rs**: Server initialization and startup

Dependencies: `mcb-domain` + `mcb-infrastructure`

---

## 3) Key Implementation Lessons

### DI Pattern (Simplified)

We use a **manual builder pattern** instead of full Shaku module macros:

```rust
// InfrastructureComponents holds resolved dependencies
pub struct InfrastructureComponents {
    pub cache: SharedCacheProvider,
    pub crypto: CryptoService,
    pub health: HealthRegistry,
    pub config: AppConfig,
}

// FullContainer combines infrastructure + domain services
pub struct FullContainer {
    pub infrastructure: InfrastructureComponents,
    pub domain_services: DomainServicesContainer,
}
```

This is simpler than Shaku modules and sufficient for the current complexity level.

### Provider Types as Strings

`EmbeddingProviderKind` and `VectorStoreProviderKind` are type aliases to `String`, not enums:

```rust
pub type EmbeddingProviderKind = String;  // "openai", "ollama", etc.
pub type VectorStoreProviderKind = String;  // "filesystem", "milvus", etc.
```

This allows extensibility without enum exhaustiveness.

### Cache Provider Pattern

```rust
pub enum CacheProviderType {
    Moka(MokaCacheProvider),
    Redis(RedisCacheProvider),
    Null(NullCacheProvider),
}

pub struct SharedCacheProvider {
    inner: Arc<CacheProviderType>,
    namespace: Option<String>,
}
```

### Error Handling

Domain errors use `thiserror` with optional source chaining:

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("Infrastructure error: {message}")]
    Infrastructure {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    // ... other variants
}
```

---

## 4) Issues Encountered and Fixed

### Crypto Compatibility (rand 0.9 vs rand_core 0.6)

**Problem**: `aes-gcm` 0.10 uses `rand_core` 0.6, but `rand` 0.9 has incompatible `OsRng`.

**Solution**: Use re-exported RNG types from the crates that need them:
```rust
use aes_gcm::aead::{OsRng as AeadOsRng, rand_core::RngCore as AeadRngCore};
use argon2::password_hash::rand_core::OsRng as ArgonOsRng;
```

### Redis API Changes

**Problem**: Redis crate made `connection_info.addr` private and removed `dbsize()` method.

**Solution**: Use `redis::cmd("DBSIZE")` instead of method call.

### Tracing Layer Type Mismatch

**Problem**: JSON and text formatters return different layer types, can't use if/else.

**Solution**: Split into separate initialization functions:
```rust
fn init_json_logging(filter: EnvFilter, file: Option<PathBuf>) -> Result<()>
fn init_text_logging(filter: EnvFilter, file: Option<PathBuf>) -> Result<()>
```

### Thread Safety Bounds

**Problem**: Cache operations need `Send + Sync` bounds for async contexts.

**Solution**: Add explicit bounds to generic parameters:
```rust
pub async fn get_or_compute<F, V, Fut>(...) -> Result<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync,
```

---

## 5) Next Steps

### Phase 2: Fix Tests

- Update test imports for new module paths
- Fix `DomainEvent` usage (enum, not trait)
- Fix `ChunkingResult` field access
- Add missing constant exports

### Phase 3: Clean Up Warnings ✅

- ~~Remove unused imports~~ Done
- ~~Add underscore prefix to unused variables~~ Done
- ~~Add #[allow(dead_code)] for intentionally kept code~~ Done

### Phase 4: Cutover

- Replace root workspace with workspace-next
- Update CI/CD pipelines
- Release as 0.1.1

---

## 6) Validation Checklist

- [x] `make build` succeeds (0 errors, 0 warnings)
- [x] `make test` passes (121+ tests passing)
- [x] `make lint` clean (0 warnings)
- [x] `make validate` passes (0 architecture violations)
- [x] Public API paths work (`mcb::run_server`, `mcb::McpServer`)
- [x] MCP tools functional (index, search, status, clear)
- [x] Config loading works (ConfigLoader + factory pattern)
- [x] DI container wiring complete (InfrastructureComponents + FullContainer)
- [ ] All feature flags compile (needs verification)

---

## 7) Parity Rules (Still Apply)

For 0.1.1 release:
- CLI flags and behavior unchanged
- Config schema identical
- Public API paths preserved
- MCP protocol behavior same
- DI composition same concrete types

---

## 8) Future Improvements (Post 0.1.1)

### Consider for 0.2.0+

- Extract admin service to separate crate if it grows
- Add integration tests for full MCP flow
- Consider Shaku modules if DI complexity increases
- Add provider health checks to registry

### Not Recommended

- Splitting into 15+ micro-crates (over-engineering for project size)
- Full Shaku module system (manual builder is sufficient)
- Separate DI crate (keep in infrastructure)
