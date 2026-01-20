# Configuration Reference

MCP Context Browser uses Figment for configuration management per [ADR-025](adr/025-figment-configuration.md).

## Overview

All configuration follows a strict precedence order (later sources override earlier):

1.  **Default values** (built into code)
2.  **TOML configuration file** (`mcb.toml`)
3.  **Environment variables** (highest precedence)

## Environment Variable Pattern

All environment variables **MUST** follow the pattern:

```
MCP__<SECTION>__<SUBSECTION>__<KEY>
```

-   Double underscore (`__`) as separator
-   Prefix is `MCP__` (not `MCB_`)
-   Keys are lowercased during parsing

### Examples

```bash
# Embedding provider
export MCP__PROVIDERS__EMBEDDING__PROVIDER=ollama
export MCP__PROVIDERS__EMBEDDING__MODEL=nomic-embed-text
export MCP__PROVIDERS__EMBEDDING__BASE_URL=http://localhost:11434

# Vector store provider
export MCP__PROVIDERS__VECTOR_STORE__PROVIDER=memory
export MCP__PROVIDERS__VECTOR_STORE__DIMENSIONS=768

# Authentication
export MCP__AUTH__ENABLED=true
export MCP__AUTH__JWT__SECRET=your-secret-at-least-32-characters-long

# Admin API key
export MCP__AUTH__ADMIN__ENABLED=true
export MCP__AUTH__ADMIN__KEY=your-admin-api-key

# Server
export MCP__SERVER__NETWORK__PORT=8080
export MCP__SERVER__NETWORK__HOST=127.0.0.1

# Cache system
export MCP__SYSTEM__INFRASTRUCTURE__CACHE__PROVIDER=Moka
export MCP__SYSTEM__INFRASTRUCTURE__CACHE__ENABLED=true

# File watching (for config hot-reload)
export MCP__SYSTEM__DATA__SYNC__WATCHING_ENABLED=false
```

## Required Configuration

### Authentication (when enabled)

When `auth.enabled = true`, the following **MUST** be configured:

| Variable | Required | Description |
|----------|----------|-------------|
| `MCP__AUTH__JWT__SECRET` | **Yes** | JWT signing secret (minimum 32 characters) |

The JWT secret is intentionally not auto-generated. This ensures:

-   No weak secrets in production
-   Fail-fast behavior if config is missing
-   Explicit security configuration

### Providers

When not using TOML configuration:

| Variable | Required | Description |
|----------|----------|-------------|
| `MCP__PROVIDERS__EMBEDDING__PROVIDER` | Yes | Embedding provider name (`ollama`, `openai`, etc.) |
| `MCP__PROVIDERS__VECTOR_STORE__PROVIDER` | Yes | Vector store provider name (`memory`, `milvus`, etc.) |

## TOML Configuration File

Default search locations (in order):

1.  `./mcb.toml` (current directory)
2.  `./mcb/mcb.toml` (mcb subdirectory)
3.  `$XDG_CONFIG_HOME/mcb/mcb.toml` (XDG config directory)
4.  `~/.mcb/mcb.toml` (home directory)

### Example `mcb.toml`

```toml
[server.network]
host = "127.0.0.1"
port = 8080

[server.ssl]
https = false

[auth]
enabled = true

[auth.jwt]
secret = "your-secret-at-least-32-characters-long"
expiration_secs = 86400

[auth.admin]
enabled = false
header = "X-Admin-Key"
# key = "your-admin-key"  # Optional

[providers.embedding]
provider = "ollama"
model = "nomic-embed-text"
base_url = "http://localhost:11434"

[providers.vector_store]
provider = "memory"
dimensions = 768

[system.infrastructure.cache]
enabled = true
provider = "Moka"
default_ttl_secs = 3600
max_size = 104857600  # 100MB

[system.data.sync]
enabled = true
watching_enabled = true  # Set to false in containers
```

## Migration from Previous Versions

### Deprecated Environment Variables

The following environment variables are **no longer supported**:

| Old Variable | New Variable | Notes |
|-------------|--------------|-------|
| `MCB_ADMIN_API_KEY` | `MCP__AUTH__ADMIN__KEY` | Prefix changed to `MCP__` |
| `DISABLE_CONFIG_WATCHING` | `MCP__SYSTEM__DATA__SYNC__WATCHING_ENABLED=false` | Now part of config |

### Breaking Changes in v0.1.2

1.  **JWT Secret Required**: The default JWT secret is now empty. When authentication is enabled, you **must** configure `MCP__AUTH__JWT__SECRET` with at least 32 characters.

2.  **Environment Variable Prefix**: All environment variables now use `MCP__` prefix (double underscore) instead of `MCB_`.

3.  **Config Watching**: File watching is now configured via `watching_enabled` in config instead of an environment variable.

## Validation

Configuration is validated at startup. The following cause startup failure:

1.  **Server port 0** when not using random port allocation
2.  **HTTPS enabled without SSL certificate/key paths**
3.  **Auth enabled with empty/short JWT secret** (< 32 chars)
4.  **Cache enabled with TTL = 0**
5.  **Memory/CPU limit = 0**
6.  **Daemon enabled with max_restart_attempts = 0**
7.  **Backup enabled with interval = 0**
8.  **Operations tracking enabled with cleanup_interval = 0 or retention = 0**

## Provider Configuration

### Embedding Providers

| Provider | Required Config |
|----------|-----------------|
| `ollama` | `base_url`, `model` |
| `openai` | `api_key`, `model` |
| `voyageai` | `api_key`, `model` |
| `gemini` | `api_key` |
| `fastembed` | (none) |
| `null` | (none, for testing) |

### Vector Store Providers

| Provider | Required Config |
|----------|-----------------|
| `memory` | (none) |
| `milvus` | `address` |
| `filesystem` | `address` (path) |
| `edgevec` | `address` (path) |
| `null` | (none, for testing) |

### Cache Providers

| Provider | Required Config |
|----------|-----------------|
| `moka` | (none) |
| `redis` | `redis_url` |
| `null` | (none, for testing) |

## Debugging Configuration

To see the loaded configuration at startup, set:

```bash
export RUST_LOG=mcb_infrastructure::config=debug
```

This will log which config file was loaded and from where.

## Related Documentation

-   [Architecture Overview](./architecture/ARCHITECTURE.md) - v0.1.2 Eight-Crate Structure
-   [ADR-025: Figment Configuration](./adr/025-figment-configuration.md) - Configuration management
-   [ADR-029: Hexagonal Architecture](./adr/029-hexagonal-architecture-dill.md) - DI and provider patterns

---

**Last Updated:** 2026-01-20
**Version:** 0.1.2
