# config Module

**Source**: `crates/mcb-infrastructure/src/config/`
**Crate**: `mcb-infrastructure`
**Files**: 3+
**Lines of Code**: ~1,000

## Overview

Application configuration management with type-safe structures, environment variable overrides, and validation.

## Key Components

### Configuration Types (`types.rs`)

Hierarchical configuration structures:

-   `AppConfig` - Root configuration
-   `ServerConfig` - Server settings (network, SSL, CORS, timeouts)
-   `AuthConfig` - Authentication (JWT settings)
-   `CacheConfig` - Cache configuration
-   `ProviderConfig` - Provider settings

### Configuration Loader (`loader.rs`)

Multi-source configuration loading:

-   File-based configuration (TOML, JSON)
-   Environment variable overrides
-   Validation and defaults
-   Hot-reload support

## File Structure

```text
crates/mcb-infrastructure/src/config/
├── types.rs              # Configuration types
├── loader.rs             # Configuration loading
└── mod.rs                # Module exports
```

## Configuration Structure

```rust
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub cache: CacheConfig,
    pub providers: ProviderConfig,
}

pub struct ServerConfig {
    pub network: ServerNetworkConfig,  // port, host
    pub ssl: ServerSslConfig,          // https, certs
    pub cors: CorsConfig,              // allowed origins
    pub timeouts: TimeoutConfig,       // request timeouts
}

pub struct AuthConfig {
    pub jwt: JwtConfig,                // secret, expiration
    pub rate_limit: RateLimitConfig,   // request limits
}
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `MCP_PORT` | Server port | 3000 |
| `MCP_HOST` | Server host | 0.0.0.0 |
| `MCP_JWT_SECRET` | JWT signing secret | (required) |
| `MCP_CACHE_ENABLED` | Enable caching | true |
| `MCP_LOG_FORMAT` | Log format (json/text) | text |

## Key Exports

```rust
pub use types::{AppConfig, ServerConfig, AuthConfig, CacheConfig};
pub use loader::ConfigLoader;
```

## Cross-References

-   **Infrastructure**: [infrastructure.md](./infrastructure.md) (parent module)
-   **Server**: [server.md](./server.md) (uses config)
-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

*Updated 2026-01-17 - Reflects modular crate architecture (v0.1.1)*
