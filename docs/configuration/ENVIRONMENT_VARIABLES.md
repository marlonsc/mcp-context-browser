# Environment Variables Reference

Complete reference for all environment variables supported by MCP Context Browser.

**Version**: 0.1.4
**Last Updated**: 2026-01-28

See also [CONFIGURATION.md](../CONFIGURATION.md) for Figment-based config (ADR-025) and `MCP__` env pattern.

---

## Overview

MCP Context Browser uses a hierarchical configuration system:

1.**Configuration Files**: `config/default.toml` and `config/local.toml`
2.**Environment Variables**: Override config file settings with prefix `MCP_`
3.**Defaults**: Built-in defaults when neither file nor env var is set

All environment variables use the pattern:

```
MCP_<SUBSYSTEM>_<PARAMETER>
```

For nested settings, use double underscores:

```
MCP_CACHE__NAMESPACES__EMBEDDINGS__TTL_SECONDS=7200
```

---

## Server Configuration

### Host and Port

| Variable | Default | Type | Description |
|----------|---------|------|-------------|
| `MCP_SERVER__HOST` | `127.0.0.1` | String | Server bind address |
| `MCP_SERVER__PORT` | `3000` | Integer | Server listen port (MCP protocol) |
| `MCP_PORT` | `3001` | Integer | Unified port for Admin + Metrics APIs |

**Usage**:

```bash
export MCP_SERVER__HOST=0.0.0.0
export MCP_SERVER__PORT=9000
export MCP_PORT=9001
```

---

## Cache Configuration

### Global Cache Settings

| Variable | Default | Type | Description |
|----------|---------|------|-------------|
| `MCP_CACHE__ENABLED` | `true` | Boolean | Enable/disable caching system |
| `MCP_CACHE__REDIS_URL` | `` (empty) | String | Redis connection URL; empty = local Moka mode |
| `MCP_CACHE__DEFAULT_TTL_SECONDS` | `3600` | Integer | Default TTL in seconds (1 hour) |
| `MCP_CACHE__MAX_SIZE` | `10000` | Integer | Max cache entries (for local Moka mode) |

**Current Architecture Note**:

-   If `REDIS_URL` is**empty**→ Uses Moka (local in-memory cache)
-   If `REDIS_URL` is**non-empty**→ Uses Redis (distributed cache)

**Migration Path**(Phase 2):
This will be replaced with:

```
MCP_CACHE__BACKEND=local|redis
MCP_CACHE__TTL_SECONDS=<seconds>
MCP_REDIS_URL=<url>  (when backend=redis)
```

### Cache Namespaces

Each namespace has its own TTL and size settings:

#### Embeddings Cache

```
MCP_CACHE__NAMESPACES__EMBEDDINGS__TTL_SECONDS=7200      # 2 hours
MCP_CACHE__NAMESPACES__EMBEDDINGS__MAX_ENTRIES=5000
MCP_CACHE__NAMESPACES__EMBEDDINGS__COMPRESSION=true
```

#### Search Results Cache

```
MCP_CACHE__NAMESPACES__SEARCH_RESULTS__TTL_SECONDS=1800   # 30 minutes
MCP_CACHE__NAMESPACES__SEARCH_RESULTS__MAX_ENTRIES=2000
MCP_CACHE__NAMESPACES__SEARCH_RESULTS__COMPRESSION=false
```

#### Metadata Cache

```
MCP_CACHE__NAMESPACES__METADATA__TTL_SECONDS=3600         # 1 hour
MCP_CACHE__NAMESPACES__METADATA__MAX_ENTRIES=1000
MCP_CACHE__NAMESPACES__METADATA__COMPRESSION=false
```

#### Provider Responses Cache

```
MCP_CACHE__NAMESPACES__PROVIDER_RESPONSES__TTL_SECONDS=300  # 5 minutes
MCP_CACHE__NAMESPACES__PROVIDER_RESPONSES__MAX_ENTRIES=3000
MCP_CACHE__NAMESPACES__PROVIDER_RESPONSES__COMPRESSION=true
```

#### Sync Batches Cache

```
MCP_CACHE__NAMESPACES__SYNC_BATCHES__TTL_SECONDS=86400    # 24 hours
MCP_CACHE__NAMESPACES__SYNC_BATCHES__MAX_ENTRIES=1000
MCP_CACHE__NAMESPACES__SYNC_BATCHES__COMPRESSION=false
```

**Usage Example**:

```bash

# Use Redis instead of local Moka
export MCP_CACHE__REDIS_URL=redis://localhost:6379

# Override embeddings TTL to 4 hours
export MCP_CACHE__NAMESPACES__EMBEDDINGS__TTL_SECONDS=14400
```

---

## Event Bus Configuration

### Event Bus Backend Selection

| Variable | Default | Type | Description |
|----------|---------|------|-------------|
| `MCP_EVENT_BUS_TYPE` | `tokio` | String | Backend: `tokio` or `nats` |
| `MCP_EVENT_BUS_CAPACITY` | `100` | Integer | Tokio channel capacity |

### NATS Configuration (when `MCP_EVENT_BUS_TYPE=nats`)

| Variable | Default | Type | Description |
|----------|---------|------|-------------|
| `MCP_NATS_URL` | `nats://localhost:4222` | String | NATS server URL |
| `MCP_NATS_RETENTION_HOURS` | `1` | Integer | Event retention in hours |
| `MCP_NATS_MAX_MSGS` | `10000` | Integer | Max messages per subject |

**Usage**:

```bash

# Use Tokio (default, single-node)
export MCP_EVENT_BUS_TYPE=tokio
export MCP_EVENT_BUS_CAPACITY=200

# Use NATS (cluster deployment)
export MCP_EVENT_BUS_TYPE=nats
export MCP_NATS_URL=nats://nats-server:4222
export MCP_NATS_RETENTION_HOURS=24
```

---

## Authentication Configuration

### JWT and Auth Settings

| Variable | Default | Type | Description |
|----------|---------|------|-------------|
| `JWT_SECRET` | `` (empty) | String | JWT signing secret (min 32 chars, required if auth enabled) |
| `JWT_EXPIRATION` | `86400` | Integer | JWT expiration in seconds (24 hours) |
| `ADMIN_PASSWORD` | `` (empty) | String | Admin account password (min 8 chars) |

**Security Model**:

-   If**both**`JWT_SECRET` and `ADMIN_PASSWORD` are set → Auth**enabled**
-   If**either**is empty → Auth**disabled**(graceful degradation)

**Production Setup**:

```bash
export JWT_SECRET="your-32-character-secret-key-here-minimum"
export ADMIN_PASSWORD="your-secure-admin-password"
export JWT_EXPIRATION="3600"  # 1 hour
```

---

## Admin Interface Configuration

### Admin API Settings

| Variable | Default | Type | Description |
|----------|---------|------|-------------|
| `ADMIN_USERNAME` | `` (empty) | String | Admin username (required if admin enabled) |
| `ADMIN_PASSWORD` | `` (empty) | String | Admin account password |
| `JWT_SECRET` | `` (empty) | String | JWT signing secret (shared with auth system) |
| `JWT_EXPIRATION` | `3600` | Integer | JWT token expiration in seconds |

**Note**: Admin interface is**optional**. It's only created if `ADMIN_USERNAME` and `ADMIN_PASSWORD` are provided.

---

## Database Configuration

### PostgreSQL Connection

| Variable | Default | Type | Description |
|----------|---------|------|-------------|
| `DATABASE_URL` | `` (empty) | String | PostgreSQL connection String; empty = disabled |
| `DATABASE_MAX_CONNECTIONS` | `20` | Integer | Connection pool size |
| `DATABASE_MIN_IDLE` | `5` | Integer | Minimum idle connections |
| `DATABASE_MAX_LIFETIME_SECS` | `1800` | Integer | Max connection lifetime (30 min) |
| `DATABASE_IDLE_TIMEOUT_SECS` | `600` | Integer | Idle timeout (10 min) |
| `DATABASE_CONNECTION_TIMEOUT_SECS` | `30` | Integer | Connection establishment timeout |

**Security Model**:

-   If `DATABASE_URL` is empty → Database**disabled**(no storage)
-   If `DATABASE_URL` is set → Database**enabled**with connection pooling

**Production Setup**:

```bash
export DATABASE_URL="postgresql://user:password@localhost:5432/mcp_context_browser"
export DATABASE_MAX_CONNECTIONS=30
export DATABASE_MIN_IDLE=10
```

---

## Metrics Configuration

### Metrics API Settings

| Variable | Default | Type | Description |
|----------|---------|------|-------------|
| `MCP_METRICS_ENABLED` | `true` | Boolean | Enable/disable metrics collection |
| `MCP_PORT` | `3001` | Integer | Metrics API port (shared with Admin) |

### Rate Limiting for Metrics

See**Rate Limiting**section below.

---

## Rate Limiting Configuration

### Rate Limit Backend and Settings

| Variable | Default | Type | Description |
|----------|---------|------|-------------|
| `MCP_RATE_LIMIT__BACKEND__TYPE` | `memory` | String | Backend: `memory` or `redis` |
| `MCP_RATE_LIMIT__ENABLED` | `true` | Boolean | Enable/disable rate limiting |
| `MCP_RATE_LIMIT__WINDOW_SECONDS` | `60` | Integer | Sliding window duration |
| `MCP_RATE_LIMIT__MAX_REQUESTS_PER_WINDOW` | `100` | Integer | Max requests per window |
| `MCP_RATE_LIMIT__BURST_ALLOWANCE` | `10` | Integer | Extra requests beyond limit |
| `MCP_RATE_LIMIT__REDIS_TIMEOUT_SECONDS` | `5` | Integer | Redis operation timeout |
| `MCP_RATE_LIMIT__CACHE_TTL_SECONDS` | `1` | Integer | Rate limit cache TTL |

### Memory Backend (Single-Node)

```
MCP_RATE_LIMIT__BACKEND__TYPE=memory
MCP_RATE_LIMIT__BACKEND__MAX_ENTRIES=10000
```

### Redis Backend (Clustered)

```
MCP_RATE_LIMIT__BACKEND__TYPE=redis
MCP_RATE_LIMIT__BACKEND__URL=redis://localhost:6379
```

---

## Resource Limits Configuration

### Memory Limits

| Variable | Default | Type | Description |
|----------|---------|------|-------------|
| `MCP_RESOURCE_LIMITS__MEMORY__MAX_USAGE_PERCENT` | `85.0` | Float | Max memory usage (%) |
| `MCP_RESOURCE_LIMITS__MEMORY__MAX_PER_OPERATION` | `536870912` | Integer | Max bytes per operation (512MB) |
| `MCP_RESOURCE_LIMITS__MEMORY__WARNING_THRESHOLD` | `75.0` | Float | Warning threshold (%) |

### CPU Limits

| Variable | Default | Type | Description |
|----------|---------|------|-------------|
| `MCP_RESOURCE_LIMITS__CPU__MAX_USAGE_PERCENT` | `80.0` | Float | Max CPU usage (%) |
| `MCP_RESOURCE_LIMITS__CPU__MAX_TIME_PER_OPERATION` | `300` | Integer | Max operation time (seconds) |
| `MCP_RESOURCE_LIMITS__CPU__WARNING_THRESHOLD` | `70.0` | Float | Warning threshold (%) |

### Disk Limits

| Variable | Default | Type | Description |
|----------|---------|------|-------------|
| `MCP_RESOURCE_LIMITS__DISK__MAX_USAGE_PERCENT` | `90.0` | Float | Max disk usage (%) |
| `MCP_RESOURCE_LIMITS__DISK__MIN_FREE_SPACE` | `1073741824` | Integer | Min free space required (1GB) |
| `MCP_RESOURCE_LIMITS__DISK__WARNING_THRESHOLD` | `80.0` | Float | Warning threshold (%) |

### Operation Limits

| Variable | Default | Type | Description |
|----------|---------|------|-------------|
| `MCP_RESOURCE_LIMITS__OPERATIONS__MAX_CONCURRENT_INDEXING` | `3` | Integer | Concurrent indexing ops |
| `MCP_RESOURCE_LIMITS__OPERATIONS__MAX_CONCURRENT_SEARCH` | `10` | Integer | Concurrent search ops |
| `MCP_RESOURCE_LIMITS__OPERATIONS__MAX_CONCURRENT_EMBEDDING` | `5` | Integer | Concurrent embedding ops |
| `MCP_RESOURCE_LIMITS__OPERATIONS__MAX_QUEUE_SIZE` | `100` | Integer | Operation queue size |

---

## Provider Configuration

### Embedding Provider

Select embedding provider and configure credentials:

#### OpenAI

```bash
export MCP_PROVIDERS__EMBEDDING__PROVIDER=openai
export MCP_PROVIDERS__EMBEDDING__MODEL=text-embedding-3-small
export MCP_PROVIDERS__EMBEDDING__API_KEY=sk-...
export MCP_PROVIDERS__EMBEDDING__DIMENSIONS=1536
```

#### Ollama (Local)

```bash
export MCP_PROVIDERS__EMBEDDING__PROVIDER=ollama
export MCP_PROVIDERS__EMBEDDING__MODEL=nomic-embed-text
export MCP_PROVIDERS__EMBEDDING__HOST=http://localhost:11434
export MCP_PROVIDERS__EMBEDDING__DIMENSIONS=768
```

#### VoyageAI

```bash
export MCP_PROVIDERS__EMBEDDING__PROVIDER=voyageai
export MCP_PROVIDERS__EMBEDDING__MODEL=voyage-2
export MCP_PROVIDERS__EMBEDDING__API_KEY=pa-...
```

#### Gemini

```bash
export MCP_PROVIDERS__EMBEDDING__PROVIDER=gemini
export MCP_PROVIDERS__EMBEDDING__MODEL=models/embedding-001
export MCP_PROVIDERS__EMBEDDING__API_KEY=...
```

### Vector Store Provider

#### In-Memory (Default, Single-Node)

```bash
export MCP_PROVIDERS__VECTOR_STORE__PROVIDER=in-memory
export MCP_PROVIDERS__VECTOR_STORE__DIMENSIONS=768
```

#### Milvus (Clustered)

```bash
export MCP_PROVIDERS__VECTOR_STORE__PROVIDER=milvus
export MCP_PROVIDERS__VECTOR_STORE__ADDRESS=milvus-server:19530
export MCP_PROVIDERS__VECTOR_STORE__DIMENSIONS=768
```

#### EdgeVec

```bash
export MCP_PROVIDERS__VECTOR_STORE__PROVIDER=edgevec
export MCP_PROVIDERS__VECTOR_STORE__ADDRESS=localhost:7374
export MCP_PROVIDERS__VECTOR_STORE__DIMENSIONS=768
```

---

## Admin Defaults

See [admin_defaults.rs](../../src/server/admin/service/helpers/admin_defaults.rs) for operational defaults:

| Variable | Default | Description |
|----------|---------|-------------|
| `ADMIN_MAX_ACTIVITIES` | `100` | Max activities in memory |
| `ADMIN_ACTIVITY_RETENTION_DAYS` | `30` | Activity history retention |
| `ADMIN_ACTIVITY_BUFFER_SIZE` | `1000` | Activity buffer capacity |
| `ADMIN_MAX_HISTORY_ENTRIES` | `1000` | Config history max entries |
| `ADMIN_HISTORY_RETENTION_DAYS` | `90` | Config history retention |
| `ADMIN_CONFIG_QUERY_LIMIT` | `100` | History query limit |
| `ADMIN_LOG_BUFFER_SIZE` | `1000` | Log buffer capacity |
| `ADMIN_LOG_RETENTION_DAYS` | `7` | Log retention |
| `ADMIN_LOG_QUERY_LIMIT` | `100` | Log query limit |
| `ADMIN_BACKUP_RETENTION_DAYS` | `30` | Backup retention |
| `ADMIN_BACKUP_COMPRESSION_LEVEL` | `6` | Gzip compression (1-9) |
| `ADMIN_MAX_BACKUPS` | `10` | Max backup files |
| `ADMIN_ROUTE_RATE_LIMIT_HEALTH` | `100` | Health endpoint rate (req/min) |
| `ADMIN_ROUTE_RATE_LIMIT_ADMIN` | `100` | Admin endpoint rate (req/min) |
| `ADMIN_ROUTE_RATE_LIMIT_INDEXING` | `10` | Indexing rate (req/min) |
| `ADMIN_ROUTE_RATE_LIMIT_SEARCH` | `10` | Search rate (req/min) |
| `ADMIN_ROUTE_RATE_LIMIT_SHUTDOWN` | `60` | Shutdown cooldown (seconds) |
| `ADMIN_ROUTE_RATE_LIMIT_RELOAD` | `30` | Reload cooldown (seconds) |
| `ADMIN_ROUTE_RATE_LIMIT_BACKUP` | `60` | Backup cooldown (seconds) |
| `ADMIN_ROUTE_RATE_LIMIT_RESTORE` | `10` | Restore rate (req/min) |
| `ADMIN_CLEANUP_BATCH_SIZE` | `100` | Cleanup batch size |
| `ADMIN_CLEANUP_RETENTION_DAYS` | `30` | Cleanup retention days |
| `ADMIN_INDEX_REBUILD_TIMEOUT_SECS` | `3600` | Rebuild timeout (1 hour) |
| `ADMIN_CACHE_CLEAR_TIMEOUT_SECS` | `300` | Cache clear timeout (5 min) |

---

## Configuration Loading Priority

Settings are loaded in this order (highest priority first):

1.**Environment Variables**(prefix `MCP_`)
2.**Local Config File**(`config/local.toml`)
3.**Default Config File**(`config/default.toml`)
4.**Built-in Defaults**(hardcoded in Rust)

Example: To override default port:

```bash
export MCP_SERVER__PORT=8080  # Environment takes precedence
```

---

## Configuration Best Practices

### Development Environment

```bash

# Enable all optional systems for local testing
export MCP_EVENT_BUS_TYPE=tokio
export MCP_CACHE__ENABLED=true
export MCP_METRICS_ENABLED=true
export ADMIN_USERNAME=admin
export ADMIN_PASSWORD=dev-password
export JWT_SECRET=dev-secret-key-32-chars-minimum
```

### Single-Node Production

```bash

# Local cache, Tokio events, no external dependencies
export MCP_CACHE__REDIS_URL=""
export MCP_EVENT_BUS_TYPE=tokio
export DATABASE_URL=postgresql://...
export ADMIN_PASSWORD=<secure-password>
export JWT_SECRET=<secure-32-char-key>
```

### Clustered Production

```bash

# Distributed cache and events
export MCP_CACHE__REDIS_URL=redis://redis-cluster:6379
export MCP_EVENT_BUS_TYPE=nats
export MCP_NATS_URL=nats://nats-cluster:4222
export DATABASE_URL=postgresql://...
export MCP_RATE_LIMIT__BACKEND__TYPE=redis
export MCP_RATE_LIMIT__BACKEND__URL=redis://redis-cluster:6379
```

### Minimal Setup (No Optional Systems)

```bash

# Bare minimum for MCP protocol operation

# - No database

# - No admin interface

# - No metrics

# - Local cache and Tokio events
unset DATABASE_URL
unset ADMIN_USERNAME
unset ADMIN_PASSWORD
unset MCP_METRICS_ENABLED
```

---

## Configuration Validation

All configuration is validated at startup:

```
✅ Configuration validated successfully
ℹ️  Server: http://127.0.0.1:3000
ℹ️  Cache: Local (Moka) mode
ℹ️  Event Bus: Tokio (in-process)
ℹ️  Admin: Disabled (no credentials provided)
```

### Common Validation Errors

**Database Disabled**

```
⚠️  Warning: Database disabled (DATABASE_URL not set)
   System will operate without persistent storage
```

**Auth Disabled**

```
⚠️  Warning: Authentication disabled
   Set JWT_SECRET and ADMIN_PASSWORD to enable auth
```

**Weak JWT Secret**

```
❌ Error: JWT_SECRET too short (must be ≥ 32 characters)
```

---

## Next Steps (Phase 2-3)

The configuration system will be refactored for better type safety:

**Current**(Magic String Detection):

```rust
let backend = if config.redis_url.is_empty() { "Moka" } else { "Redis" };
```

**Future**(Type-Safe Enum):

```rust
pub enum CacheBackendConfig {
    Local { max_entries: usize, ttl: Duration },
    Redis { url: String, pool_size: usize },
}
```

This will replace:

-   `MCP_CACHE__REDIS_URL` with `MCP_CACHE__BACKEND=local|redis`
-   `MCP_EVENT_BUS_TYPE` will be moved to `EventBusConfig::from_env()`
-   All config enums will support environment variable overrides

---

## Troubleshooting

### Configuration Not Being Applied

1.**Check environment variable format**: Use `MCP_` prefix and `__` for nesting
2.**Verify spacing**: No spaces around `=` in exports
3.**Check precedence**: Environment variables override config files
4.**Enable debug logging**: `RUST_LOG=debug` to see config loading

### Port Already in Use

```bash

# Server port conflict
export MCP_SERVER__PORT=9000

# Metrics port conflict
export MCP_PORT=9001
```

### Cache Not Working

```bash

# Verify cache enabled
export MCP_CACHE__ENABLED=true

# If using Redis, verify URL
export MCP_CACHE__REDIS_URL=redis://localhost:6379
redis-cli ping  # Should respond with PONG
```

---

## See Also

-   [Configuration Types](../../src/infrastructure/config/) - Source code
-   [Admin Defaults](../../src/server/admin/service/helpers/admin_defaults.rs) - Operational settings
-   [CONFIGURATION.md](../CONFIGURATION.md) - General configuration guide
