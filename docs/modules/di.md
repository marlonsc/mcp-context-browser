# di Module

**Source**: `crates/mcb-infrastructure/src/di/`
**Crate**: `mcb-infrastructure`
**Files**: 5+
**Lines of Code**: ~500

## Overview

Shaku-based hierarchical dependency injection container system for Clean Architecture.

## Key Components

### Bootstrap (`bootstrap.rs`)

DI container initialization and module wiring.

### Modules (`modules/`)

Shaku module definitions:

-   `infrastructure.rs` - Infrastructure services module
-   `server.rs` - Server components module
-   `providers.rs` - Provider registrations
-   `traits.rs` - Module trait definitions

## File Structure

```text
crates/mcb-infrastructure/src/di/
├── bootstrap.rs          # DI container setup
├── modules/
│   ├── infrastructure.rs # Infrastructure services
│   ├── server.rs         # Server components
│   ├── providers.rs      # Provider registrations
│   ├── traits.rs         # Module traits
│   └── mod.rs
└── mod.rs                # Module exports
```

## DI Pattern

```rust
// Define module
shaku::module! {
    pub InfrastructureModule {
        components = [ConfigComponent, LoggingComponent],
        providers = []
    }
}

// Register component
#[derive(Component)]
#[shaku(interface = ConfigInterface)]
pub struct ConfigComponent {
    config: AppConfig,
}

// Resolve dependency
let config: &dyn ConfigInterface = module.resolve();
```

## Module Hierarchy

```text
InfrastructureModule (config, logging, health)
    └── ProvidersModule (embedding, vector_store, cache)
        └── ServerModule (handlers, admin, transport)
```

## Key Exports

```rust
pub use bootstrap::{create_module, McpModule};
pub use modules::traits::{ConfigHealthAccess, StorageComponentsAccess, ProviderComponentsAccess};
```

## Cross-References

-   **Infrastructure**: [infrastructure.md](./infrastructure.md) (parent module)
-   **Domain Ports**: [domain.md](./domain.md) (interface definitions)
-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

*Updated 2026-01-17 - Reflects modular crate architecture (v0.1.1)*
