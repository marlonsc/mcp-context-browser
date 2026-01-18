# ADR 023: Inventory to Linkme Migration

## Status

**Accepted** (v0.1.2)

> Completed in v0.1.2. All 15 providers migrated to linkme.
> Supersedes inventory-based registration.

## Context

The current codebase uses the `inventory` crate for plugin registration, which provides compile-time collection of static items through procedural macros (`inventory::submit!` and `inventory::collect!`). While functional, this approach has several drawbacks:

1. **Heavy infrastructure**: Generates significant boilerplate code for registration
2. **Limited platform support**: May not work well with WebAssembly (WASM) targets
3. **Complex macros**: Requires understanding of inventory's macro system
4. **Maintenance overhead**: Additional dependency that could be simplified

The `linkme` crate offers an alternative approach using distributed slices, which are aggregated by the linker at compile time. This approach:

1. **Eliminates boilerplate**: Uses simple attribute macros instead of complex registration calls
2. **Broader platform support**: Works with WASM and other constrained environments
3. **Simpler API**: Just add `#[linkme::distributed_slice]` and static items are automatically collected
4. **Better performance**: Linker-based collection is more efficient than runtime iteration

## Decision

We will migrate from `inventory` to `linkme` for all plugin registration across the codebase. The migration will:

1. Replace `inventory::collect!` declarations with `linkme::distributed_slice!`
2. Replace `inventory::submit!` calls with simple static item declarations using `#[linkme::distributed_slice(MY_SLICE)]`
3. Update all provider registration code (embedding, vector store, cache, language providers)
4. Maintain the same runtime API for provider discovery and resolution
5. Remove the `inventory` dependency and add `linkme`

### Migration Pattern

**Before (inventory):**
```rust
// Declaration
inventory::collect!(EmbeddingProviderEntry);

// Registration
inventory::submit! {
    EmbeddingProviderEntry {
        name: "ollama",
        description: "Ollama provider",
        factory: |config| { /* ... */ },
    }
}
```

**After (linkme):**
```rust
// Declaration
#[linkme::distributed_slice]
pub static EMBEDDING_PROVIDERS: [EmbeddingProviderEntry] = [..];

// Registration
#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static OLLAMA_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "ollama",
    description: "Ollama provider",
    factory: |config| { /* ... */ },
};
```

## Consequences

### Positive
- **Reduced complexity**: Eliminates complex macro infrastructure
- **Better platform support**: Enables WASM and embedded targets
- **Smaller binary size**: Less generated code from macros
- **Simpler maintenance**: Fewer dependencies and less boilerplate
- **Performance**: Linker-based collection is more efficient

### Negative
- **Breaking change**: Requires updating all provider registration code
- **Migration effort**: Need to update ~20+ provider registration sites
- **Testing required**: Must verify all providers are still discovered correctly

### Risks
- **Linker compatibility**: Some build environments may have linker limitations
- **Debugging difficulty**: Distributed slice issues may be harder to debug
- **Migration complexity**: Large-scale change affecting multiple crates

## Implementation Plan

1. **Phase 1**: Add linkme dependency and create migration utility functions
2. **Phase 2**: Migrate one provider type (e.g., embedding) as proof of concept
3. **Phase 3**: Migrate remaining provider types (vector store, cache, language)
4. **Phase 4**: Remove inventory dependency and clean up old code
5. **Phase 5**: Comprehensive testing and validation

## Implementation Status

- [x] All embedding providers (6): ollama, openai, voyageai, gemini, fastembed, null
- [x] All cache providers (3): moka, redis, null
- [x] All vector store providers (5): milvus, filesystem, in_memory, edgevec, null
- [x] All language providers (1): universal
- [x] Pure linkme registries (no inventory fallback)
- [x] All provider entry structs use `#[linkme::distributed_slice]`
- [x] Remove stale `inventory` dependency from workspace Cargo.toml (cleanup)

## Validation Criteria

- [x] All providers are correctly registered and discoverable
- [x] Build succeeds on all supported platforms (Linux, macOS, Windows)
- [ ] WASM builds work (future compatibility)
- [ ] Performance benchmarks show no regression
- [x] All integration tests pass
- [ ] Binary size is reduced or maintained

## Related ADRs

- [ADR 002: Dependency Injection with Shaku](002-dependency-injection-shaku.md) - Related DI strategy
- [ADR 003: Unified Provider Architecture](003-unified-provider-architecture.md) - Provider registration system
- [ADR 013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Multi-crate organization