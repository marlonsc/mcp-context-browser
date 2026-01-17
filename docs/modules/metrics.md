# Metrics Module

**Source**: `crates/mcb-providers/src/admin/metrics.rs` and `crates/mcb-server/src/admin/`
**Crates**: `mcb-providers`, `mcb-server`

System monitoring, performance tracking, and HTTP metrics API.

## Overview

The metrics functionality is distributed across crates in v0.1.1:

-   **mcb-providers**: `AtomicPerformanceMetrics` - Performance tracking
-   **mcb-server**: Admin endpoints for metrics exposure

## Components

### AtomicPerformanceMetrics (`mcb-providers`)

Thread-safe performance metrics collection:

-   Query latency (P50, P95, P99)
-   Cache hit/miss rates
-   Request throughput
-   Error rates

### Metrics Endpoints (`mcb-server`)

HTTP API for metrics access via admin router.

**Endpoints**:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Health check |
| `/health/ready` | GET | Readiness probe |
| `/health/live` | GET | Liveness probe |
| `/metrics` | GET | Performance metrics JSON |

## File Structure

```text
crates/mcb-providers/src/admin/
└── metrics.rs               # AtomicPerformanceMetrics

crates/mcb-server/src/admin/
├── handlers.rs              # Metrics endpoint handlers
└── models.rs                # MetricsResponse types
```

## Key Exports

```rust
// From mcb-providers
pub use admin::metrics::AtomicPerformanceMetrics;

// From mcb-server
pub use admin::{metrics_handler, MetricsResponse};
```

## Configuration

Environment variables:

-   `MCP_METRICS_ENABLED=true` - Enable metrics collection
-   `MCP_PORT=3000` - Unified HTTP port (Admin + Metrics + MCP)

## Cross-References

-   **Admin**: [admin.md](./admin.md) (metrics endpoints)
-   **Server**: [server.md](./server.md) (HTTP server)
-   **Providers**: [providers.md](./providers.md) (metrics implementation)
-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

*Updated 2026-01-17 - Reflects modular crate architecture (v0.1.1)*
