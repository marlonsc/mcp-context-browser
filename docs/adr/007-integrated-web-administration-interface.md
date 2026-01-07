# ADR 007: Integrated Web Administration Interface

## Status

Proposed

## Context

MCP Context Browser v0.0.4 provides comprehensive system monitoring and metrics through HTTP endpoints on port 3001, but lacks a user-friendly web interface for administration, configuration, and visualization. Users currently need to interact with the system through:

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
-   Basic HTML dashboard (assets/dashboard.html)
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

The web interface will extend the existing `MetricsApiServer` in `src/metrics/http_server.rs`:

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

Static assets served from embedded resources:

```
assets/
├── admin/
│   ├── index.html
│   ├── css/
│   │   ├── main.css
│   │   └── components.css
│   ├── js/
│   │   ├── app.js
│   │   ├── api.js
│   │   ├── charts.js
│   │   └── websocket.js
│   └── img/
│       └── logo.svg
```

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

## References

-   [ADR 006: Code Audit and Improvements v0.0.4](006-code-audit-and-improvements-v0.0.4.md)
-   [Existing HTTP Server](src/metrics/http_server.rs)
-   [Current Dashboard](assets/dashboard.html)
-   [Metrics API Design](docs/api-reference.md)
