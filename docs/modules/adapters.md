# adapters Module

**Note**: In v0.1.1, adapters have been reorganized into dedicated crates.

**Previous Source**: `src/adapters/`
**New Location**: `crates/mcb-providers/src/` and `crates/mcb-infrastructure/src/adapters/`

## Overview

The adapters layer has been split into two crates in v0.1.1:

1. **mcb-providers** - External service integrations (embedding, vector store, cache, language processors)
2. **mcb-infrastructure** - Null adapters for testing and DI

## Migration Mapping

| Old Location | New Location |
|--------------|--------------|
| `src/adapters/providers/embedding/` | `crates/mcb-providers/src/embedding/` |
| `src/adapters/providers/vector_store/` | `crates/mcb-providers/src/vector_store/` |
| `src/adapters/providers/routing/` | `crates/mcb-providers/src/routing/` |
| `src/adapters/hybrid_search/` | `crates/mcb-providers/src/hybrid_search/` |
| `src/adapters/repository/` | `crates/mcb-infrastructure/src/adapters/repository/` |
| Null adapters | `crates/mcb-infrastructure/src/adapters/infrastructure/` |

## Related Documentation

-   **Providers**: [providers.md](./providers.md) - Provider implementations
-   **Infrastructure**: [infrastructure.md](./infrastructure.md) - Null adapters
-   **Domain**: [domain.md](./domain.md) - Port trait definitions
-   **Module Structure**: [module-structure.md](./module-structure.md) - Full architecture

---

*Updated 2026-01-17 - Reflects modular crate architecture (v0.1.1)*
