# Metrics Module

**Source**: `src/metrics/`

System monitoring, performance tracking, and HTTP metrics API.

## Overview

The metrics module provides comprehensive observability for the MCP Context Browser. It collects system metrics (CPU, memory, disk), tracks query performance, and exposes a REST API for monitoring dashboards.

## Components

### SystemMetricsCollector (`system.rs`)

Collects system-level metrics using `sysinfo` crate.

-   CPU usage and load averages
-   Memory utilization (used/total/available)
-   Disk I/O and storage capacity
-   Network statistics

### PerformanceMetrics (`performance.rs`)

Tracks application performance.

-   Query latency (P50, P95, P99)
-   Cache hit/miss rates
-   Request throughput
-   Error rates

### MetricsApiServer (`http_server.rs`)

HTTP API for metrics access (port 3001).

**Endpoints**:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/health` | GET | Health check |
| `/api/metrics` | GET | Prometheus-format metrics |
| `/api/context/metrics` | GET | Application metrics JSON |

### CacheMetrics

Cache performance tracking.

-   Hit count / miss count
-   Hit rate percentage
-   Eviction statistics

## File Structure

```text
src/metrics/
├── http_server.rs   # REST API server
├── mod.rs           # Module exports
├── performance.rs   # Query performance tracking
└── system.rs        # System metrics collection
```

## Key Exports

```rust
pub use http_server::{MetricsApiServer, HealthResponse};
pub use performance::{PerformanceMetrics, CacheMetrics, QueryPerformanceMetrics};
pub use performance::PERFORMANCE_METRICS;
```

## Configuration

Environment variables:

-   `MCP_METRICS_ENABLED=true` - Enable metrics collection
-   `MCP_PORT=3001` - Unified HTTP port (Admin + Metrics + MCP)

## Testing

5 metrics tests. See [tests/metrics.rs](../../tests/metrics.rs).

## Cross-References

-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
-   **Server**: [server.md](./server.md) (integrates metrics)
-   **Admin**: [admin.md](./admin.md) (metrics dashboard)
