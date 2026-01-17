# admin Module

**Source**: `crates/mcb-server/src/admin/`
**Crate**: `mcb-server`
**Files**: 8
**Lines of Code**: ~1,500

## Overview

The admin module provides administrative endpoints for health checks, metrics, configuration management, and server control. Part of the mcb-server crate.

## Key Components

### Handlers (`handlers.rs`, `config_handlers.rs`)

HTTP endpoint handlers:

-   Health check endpoints (readiness, liveness)
-   Metrics retrieval
-   Graceful shutdown
-   Configuration management

### Routes (`routes.rs`)

Axum router configuration for admin API endpoints.

### Models (`models.rs`)

Request/response types:

-   `HealthResponse` - Health status response
-   `MetricsResponse` - Performance metrics
-   `ConfigResponse` - Configuration data

### Service (`service.rs`)

Admin service orchestration and business logic.

## File Structure

```text
crates/mcb-server/src/admin/
├── handlers.rs           # Core endpoint handlers
├── config_handlers.rs    # Configuration handlers
├── routes.rs             # Router setup
├── models.rs             # Data types
├── service.rs            # Service layer
└── mod.rs                # Module exports
```

## API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Health status |
| `/health/ready` | GET | Readiness probe |
| `/health/live` | GET | Liveness probe |
| `/metrics` | GET | Performance metrics |
| `/config` | GET | Current configuration |
| `/shutdown` | POST | Graceful shutdown |

## Key Exports

```rust
pub use handlers::{health_handler, metrics_handler, shutdown_handler};
pub use routes::admin_router;
pub use models::{HealthResponse, MetricsResponse};
```

## Cross-References

-   **Server**: [server.md](./server.md) (parent module)
-   **Metrics**: [metrics.md](./metrics.md) (metrics collection)
-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

*Updated 2026-01-17 - Reflects modular crate architecture (v0.1.1)*
