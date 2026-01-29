# ADR 007: Integrated Web Administration Interface

## Status

**In Progress** (v0.1.1 → v0.2.0)

> Backend infrastructure implemented in `crates/mcb-server/src/admin/`:
>
> -   AdminService trait with 32 methods (traits.rs)
> -   AdminServiceImpl with full implementation (implementation.rs, helpers/)
> -   REST API routes for config, health, backup, maintenance (routes.rs, handlers.rs)
> -   JWT authentication integration
> -   **New (v0.2.0)**:
>     -   Unified port architecture (MCP + Admin + Metrics on port 3001)
>     -   Subsystem control via EventBus (6 new AdminService methods)
>     -   Configuration persistence with explicit save pattern
>     -   14 SystemEvent variants for inter-subsystem communication
> -   **Pending**: Frontend HTML/CSS/JS refinement, WebSocket real-time updates

## Context

MCP Context Browser provides comprehensive system monitoring and metrics through HTTP endpoints on port 3001, but lacks a user-friendly web interface for administration, configuration, and visualization. Users currently need to interact with the system through:

1.  Environment variables for configuration
2.  MCP protocol tools for basic operations
3.  HTTP API endpoints for metrics (no authentication)
4.  Terminal/command-line for management tasks

This creates barriers for non-technical users and makes it difficult to:

-   Monitor system health in real-time
-   Configure providers dynamically
-   Manage indexes and search operations
-   Troubleshoot issues without technical expertise
-   Visualize performance metrics and trends

The existing infrastructure already includes:

-   HTTP metrics server on port 3001
-   Comprehensive metrics collection
-   Provider management capabilities
-   Configuration system

## Decision

We will implement an integrated web administration interface that runs on the same port as the metrics server (3001), providing a modern, responsive web UI for:

1.  **System Dashboard**: Real-time metrics visualization with interactive charts
2.  **Configuration Management**: Dynamic provider and system configuration
3.  **Provider Management**: Add/remove/configure embedding and vector store providers
4.  **Index Management**: Control indexing operations, view status, clear indexes
5.  **Security**: Basic authentication for administrative access
6.  **Monitoring**: Enhanced health monitoring and alerting

The interface will be implemented using:

-   **Backend**: Extend existing Axum HTTP server with new REST endpoints
-   **Frontend**: Modern HTML/CSS/JavaScript with responsive design
-   **Authentication**: JWT-based authentication for admin access
-   **Real-time**: WebSocket support for live metrics updates
-   **API**: RESTful JSON API for all administrative operations

## Consequences

### Positive Consequences

-   **Improved User Experience**: Non-technical users can manage the system through a web interface
-   **Real-time Monitoring**: Live dashboards with interactive charts and alerts
-   **Operational Efficiency**: Faster troubleshooting and configuration changes
-   **Security Enhancement**: Authentication and authorization for administrative access
-   **Unified Interface**: Single port (3001) serves both metrics API and admin interface
-   **Extensibility**: Foundation for future web-based features
-   **Developer Productivity**: Easier testing and development workflows

### Negative Consequences

-   **Increased Complexity**: Additional code and maintenance overhead
-   **Security Surface**: Web interface introduces new attack vectors
-   **Resource Usage**: Additional memory/CPU for serving static assets and WebSocket connections
-   **Deployment Complexity**: Web assets need to be bundled and served
-   **Browser Dependencies**: Interface requires modern browsers with JavaScript enabled

## Alternatives Considered

### Alternative 1: Separate Administration Service

-   **Description**: Create a standalone web service on a different port for administration
-   **Pros**: Clean separation, independent scaling, dedicated resources
-   **Cons**: Additional port management, deployment complexity, CORS issues
-   **Rejection Reason**: Increases operational complexity and goes against the requirement to run on the same port

### Alternative 2: Terminal-based Administration Only

-   **Description**: Enhance CLI tools and keep all administration terminal-based
-   **Pros**: Lower resource usage, simpler architecture, no web dependencies
-   **Cons**: Poor user experience, limited visualization, accessibility issues
-   **Rejection Reason**: Doesn't address the need for web-based administration and visualization

### Alternative 3: Third-party Admin Interface

-   **Description**: Use existing tools like Grafana or custom dashboards
-   **Pros**: Leverage existing ecosystems, faster implementation
-   **Cons**: External dependencies, integration complexity, customization limitations
-   **Rejection Reason**: Doesn't provide integrated experience and requires additional setup

### Alternative 4: Desktop Application

-   **Description**: Native desktop app for administration
-   **Pros**: Better performance, native integrations
-   **Cons**: Platform-specific development, deployment challenges, additional maintenance
-   **Rejection Reason**: Web-based requirement and cross-platform needs

## Implementation Notes

### Architecture Integration

The web interface will extend the existing `MetricsApiServer` in `crates/mcb-infrastructure/src/metrics/http_server.rs`:

```rust
pub struct AdminApiServer {
    metrics_server: MetricsApiServer,
    admin_routes: Router,
    websocket_handler: WebSocketHandler,
}
```

### API Endpoints

New REST endpoints under `/admin/` prefix:

-   `GET /admin/` - Serve main admin interface
-   `GET /admin/config` - Get current configuration
-   `PUT /admin/config` - Update configuration
-   `GET /admin/providers` - List providers
-   `POST /admin/providers` - Add provider
-   `DELETE /admin/providers/{id}` - Remove provider
-   `GET /admin/indexes` - List indexes
-   `POST /admin/indexes/{id}/clear` - Clear index
-   `POST /admin/auth/login` - Authentication

### Frontend Structure

Templates are **embedded at compile time** using `include_str!` macro, making the binary self-contained:

```
crates/mcb-server/src/admin/web/templates/
├── base.html              # Master layout (Tailwind + Alpine.js + HTMX)
├── dashboard.html         # Main dashboard
├── providers.html         # Provider management
├── indexes.html           # Index management
├── configuration.html     # Config editor
├── maintenance.html       # Maintenance operations
├── diagnostics.html       # Diagnostic tools
├── data_management.html   # Backup UI
├── logs.html              # Log viewer
├── login.html             # Authentication
├── admin.css              # Custom styles
└── htmx/                  # HTMX partials for dynamic updates
    ├── dashboard_metrics.html
    ├── providers_list.html
    ├── indexes_list.html
    ├── subsystems_list.html
    └── config_diff.html
```

**Key implementation detail**: All templates are loaded via `include_str!` in `crates/mcb-server/src/admin/web.rs` and added to Tera at compile time, eliminating runtime filesystem access.

### Security Implementation

-   JWT authentication with configurable expiration
-   Role-based access (admin vs read-only)
-   CSRF protection for state-changing operations
-   HTTPS enforcement in production
-   Rate limiting for API endpoints

### Testing Strategy

-   Unit tests for new API endpoints
-   Integration tests for web interface functionality
-   E2E tests for critical admin workflows
-   Security testing for authentication and authorization

### Migration Path

1.  **Phase 1**: Extend HTTP server with admin routes
2.  **Phase 2**: Implement authentication middleware
3.  **Phase 3**: Create basic HTML/CSS/JS interface
4.  **Phase 4**: Add WebSocket support for real-time updates
5.  **Phase 5**: Implement advanced features (charts, provider management)

### Performance Considerations

-   Static asset compression and caching
-   Lazy loading for JavaScript modules
-   Efficient WebSocket connection management
-   Minimal impact on existing metrics endpoints

### Rollback Plan

-   Feature flag to disable admin interface
-   Separate admin routes can be easily removed
-   Database migrations (if any) are reversible
-   Static assets can be excluded from builds

## Unified Port Architecture (v0.2.0)

All HTTP services run on a single unified port (default: 3001).

### Port Configuration

Configure via environment variable:

```bash
export MCP_PORT=3001  # Default unified port for Admin + Metrics + MCP
```

### URL Structure

```
Port 3001 (Unified: Admin + Metrics + MCP HTTP)
├── /                - Admin dashboard (root redirects to dashboard)
├── /dashboard       - Admin dashboard
├── /providers       - Provider management UI
├── /indexes         - Index management UI
├── /config          - Configuration UI
├── /logs            - Log viewer UI
├── /maintenance     - Maintenance UI
├── /diagnostics     - Diagnostic tools UI
├── /data            - Data management UI
├── /login           - Authentication page
├── /admin.css       - Admin stylesheet
├── /htmx*/         - HTMX partial endpoints
├── /admin*/        - Admin REST API endpoints
├── /api*/          - Metrics API endpoints
└── /mcp*/          - MCP protocol HTTP transport
```

**Note**: The root path `/` serves the admin dashboard directly, making it the default landing page.

### Implementation

The `MetricsApiServer` now accepts multiple routers via builder pattern:

```rust
// crates/mcb-infrastructure/src/metrics/http_server.rs
impl MetricsApiServer {
    pub fn with_external_router(mut self, router: Router) -> Self;
    pub fn with_mcp_router(mut self, router: Router) -> Self;
}
```

Initialization in `crates/mcb-server/src/init.rs`:

```rust
let metrics_server = MetricsApiServer::with_limits(...)
    .with_external_router(admin_router)   // Admin routes
    .with_mcp_router(mcp_router);         // MCP protocol
```

### Benefits

-   Single port simplifies firewall/proxy configuration
-   Eliminates port 3002 (previously MCP HTTP transport)
-   Unified graceful shutdown via ConnectionTracker
-   Consistent rate limiting and CORS across all endpoints

## Subsystem Control Protocol (v0.2.0)

Web admin can monitor and control all MCP subsystems via EventBus.

### New SystemEvent Variants

Added to `crates/mcb-infrastructure/src/events/mod.rs`:

```rust
pub enum SystemEvent {
    // Existing (10 variants): CacheClear, BackupCreate, BackupRestore,
    // IndexRebuild, ConfigReloaded, Shutdown, Reload, Respawn,
    // BinaryUpdated, SyncCompleted

    // New (4 variants for subsystem control):
    ProviderRestart { provider_type: String, provider_id: String },
    ProviderReconfigure { provider_type: String, config: serde_json::Value },
    SubsystemHealthCheck { subsystem_id: String },
    RouterReload,
}
```

### New AdminService Methods

Added to `crates/mcb-server/src/admin/service/traits.rs`:

```rust
#[async_trait]
pub trait AdminService: Interface + Send + Sync {
    // ... existing 26 methods ...

    // Subsystem introspection (6 new methods)
    async fn get_subsystems(&self) -> Result<Vec<SubsystemInfo>, AdminError>;
    async fn send_subsystem_signal(&self, id: &str, signal: SubsystemSignal) -> Result<SignalResult, AdminError>;
    async fn get_routes(&self) -> Result<Vec<RouteInfo>, AdminError>;
    async fn reload_routes(&self) -> Result<MaintenanceResult, AdminError>;
    async fn persist_configuration(&self) -> Result<ConfigPersistResult, AdminError>;
    async fn get_config_diff(&self) -> Result<ConfigDiff, AdminError>;
}
```

### Subsystem Types

Defined in `crates/mcb-server/src/admin/service/types.rs`:

```rust
pub enum SubsystemType {
    Embedding,      // Embedding providers
    VectorStore,    // Vector database providers
    Search,         // Search service
    Indexing,       // Indexing service
    Cache,          // Cache manager
    Metrics,        // Metrics collector
    Daemon,         // Background daemon
    HttpTransport,  // HTTP server
}

pub enum SubsystemStatus {
    Running, Stopped, Error, Starting, Paused, Unknown
}

pub struct SubsystemInfo {
    pub id: String,
    pub name: String,
    pub subsystem_type: SubsystemType,
    pub status: SubsystemStatus,
    pub health: HealthCheck,
    pub config: serde_json::Value,
    pub metrics: SubsystemMetrics,
}
```

### Event Flow Example

```

1. User clicks "Restart" on embedding subsystem
2. POST /admin/subsystems/embedding:ollama/signal {"signal":"restart"}
3. AdminService::send_subsystem_signal() called
4. EventBus publishes SystemEvent::ProviderRestart
5. Embedding provider's event listener receives and restarts
6. Dashboard HTMX poll shows updated status

```

## Configuration Management (v0.2.0)

Implements explicit save pattern for configuration changes.

### Runtime vs Persisted Configuration

-   **Runtime**: Changes via `update_configuration()` apply immediately (ArcSwap)
-   **Persisted**: `persist_configuration()` writes to `~/.context/config.toml`
-   **Diff**: `get_config_diff()` shows runtime vs file differences

### API Flow

```

1. GET /admin/configuration         # View current config
2. PUT /admin/configuration         # Update runtime only
3. POST /admin/configuration/save   # Persist to file
4. GET /admin/configuration/diff    # Compare runtime vs file

```

### Implementation Pattern

```rust
// Runtime update (immediate, not persisted)
pub async fn update_configuration(&self, updates: HashMap<String, Value>, user: &str)
    -> Result<ConfigurationUpdateResult, AdminError>;

// Explicit persist to file
pub async fn persist_configuration(&self) -> Result<ConfigPersistResult, AdminError>;

// Check for unsaved changes
pub async fn get_config_diff(&self) -> Result<ConfigDiff, AdminError>;
```

## Template Organization

Templates located in `crates/mcb-server/src/admin/web/templates/`:

```
templates/
├── base.html           # Master layout (Alpine.js + Tailwind + HTMX)
├── dashboard.html      # Real-time metrics dashboard
├── providers.html      # Provider management
├── indexes.html        # Index management
├── configuration.html  # Config editor
├── maintenance.html    # Maintenance operations
├── diagnostics.html    # Diagnostic tools
├── data_management.html # Backup UI
├── logs.html           # Log viewer
├── login.html          # Authentication
├── admin.css           # Custom styles
└── htmx/               # HTMX partials for dynamic updates
    ├── dashboard_metrics.html
    ├── providers_list.html
    └── indexes_list.html
```

## Related ADRs

-   [ADR-001: Modular Crates Architecture](001-modular-crates-architecture.md) - Provider pattern for admin services
-   [ADR-002: Async-First Architecture](002-async-first-architecture.md) - Async handlers
-   [ADR-006: Code Audit and Improvements](006-code-audit-and-improvements.md) - Code quality standards
-   [ADR-008: Git-Aware Semantic Indexing](008-git-aware-semantic-indexing-v0.2.0.md) - Git integration for admin UI
-   [ADR-012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) - DI for admin services
-   [ADR-013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Crate organization

## References

-   [Existing HTTP Server](../../crates/mcb-infrastructure/src/metrics/http_server.rs)
-   [Server Initialization](../../crates/mcb-server/src/init.rs)
-   [Admin Service](../../crates/mcb-server/src/admin/service/)
-   [Shaku Documentation](https://docs.rs/shaku) (historical; DI is now dill, ADR-029)
