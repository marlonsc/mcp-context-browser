# infrastructure Module

**Source**: `crates/mcb-infrastructure/src/`
**Crate**: `mcb-infrastructure`
**Files**: 40+
**Lines of Code**: ~6,000

## Overview

The infrastructure module provides shared technical services and cross-cutting concerns for the MCP Context Browser system. It implements Shaku-based dependency injection, configuration management, caching, health checks, and null adapters for testing.

## Key Components

### Dependency Injection (`di/`)

Shaku-based hierarchical DI container system:

-   `bootstrap.rs` - DI container initialization
-   `modules/` - Shaku module definitions
    -   `infrastructure.rs` - Infrastructure services module
    -   `server.rs` - Server components module
    -   `providers.rs` - Provider registrations
    -   `traits.rs` - Module trait definitions

### Configuration (`config/`)

Application configuration management:

-   Type-safe configuration with nested structures
-   Environment variable overrides
-   Server, auth, cache, and provider configurations

### Cache (`cache/`)

Caching infrastructure:

-   Cache configuration and management
-   Integration with mcb-providers cache implementations

### Crypto (`crypto/`)

Encryption and hashing utilities:

-   AES-GCM encryption support
-   Hash computation utilities

### Health (`health/`)

Health check infrastructure:

-   Component health monitoring
-   Readiness and liveness checks

### Logging (`logging/`)

Structured logging configuration:

-   Tracing integration
-   Log level management

### Adapters (`adapters/`)

Null implementations for DI testing:

-   `infrastructure/` - Null adapters for infrastructure ports
    -   `NullAuthService`
    -   `NullEventBus`
    -   `NullSyncProvider`
    -   `NullLockProvider`
    -   `NullSnapshotProvider`
    -   `NullStateStoreProvider`
    -   `NullPerformanceMetrics`
    -   `NullIndexingOperations`
    -   `NullSystemMetricsCollector`
-   `providers/` - Provider adapter bindings
-   `repository/` - Repository adapters
    -   `NullChunkRepository`
    -   `NullSearchRepository`

## File Structure

```text
crates/mcb-infrastructure/src/
├── di/
│   ├── bootstrap.rs        # DI container setup
│   ├── modules/
│   │   ├── infrastructure.rs
│   │   ├── server.rs
│   │   ├── providers.rs
│   │   ├── traits.rs
│   │   └── mod.rs
│   └── mod.rs
├── config/
│   ├── types.rs            # Configuration types
│   └── mod.rs
├── cache/
│   └── mod.rs
├── crypto/
│   └── mod.rs
├── health/
│   └── mod.rs
├── logging/
│   └── mod.rs
├── adapters/
│   ├── infrastructure/     # Null infrastructure adapters
│   │   ├── auth.rs
│   │   ├── events.rs
│   │   ├── metrics.rs
│   │   ├── snapshot.rs
│   │   ├── sync.rs
│   │   ├── admin.rs
│   │   └── mod.rs
│   ├── providers/          # Provider bindings
│   │   └── mod.rs
│   ├── repository/         # Repository adapters
│   │   └── mod.rs
│   └── mod.rs
├── infrastructure/         # Re-exports
│   └── mod.rs
└── lib.rs                  # Crate root
```

## Key Exports

```rust
// DI
pub use di::{bootstrap, McpModule};

// Configuration
pub use config::{AppConfig, ServerConfig, AuthConfig};

// Null Adapters (for testing)
pub use adapters::infrastructure::{
    NullAuthService, NullEventBus, NullSyncProvider,
    NullSnapshotProvider, NullPerformanceMetrics,
};
```

## Testing

Infrastructure tests are located in `crates/mcb-infrastructure/tests/`.

## Cross-References

-   **Domain Ports**: [domain.md](./domain.md) (interfaces implemented)
-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
-   **Module Structure**: [module-structure.md](./module-structure.md)

---

*Updated 2026-01-17 - Reflects modular crate architecture (v0.1.1)*
