//! Admin Interface
//!
//! Administrative interfaces for system monitoring and management.
//! Uses domain ports to maintain Clean Architecture separation.
//!
//! ## Architecture
//!
//! The admin module follows the same Clean Architecture pattern as the rest
//! of the server:
//!
//! - **Domain Ports** (`mcb_application::ports::admin`): Define the interfaces
//! - **Infrastructure Adapters** (`mcb_infrastructure::adapters::admin`): Implementations
//! - **Server Handlers** (this module): HTTP handlers and routes
//!
//! ## Endpoints
//!
//! | Path | Method | Description |
//! |------|--------|-------------|
//! | `/health` | GET | Health check with uptime |
//! | `/health/extended` | GET | Extended health with dependency status |
//! | `/metrics` | GET | Performance metrics |
//! | `/indexing` | GET | Indexing operations status |
//! | `/ready` | GET | Kubernetes readiness probe |
//! | `/live` | GET | Kubernetes liveness probe |
//! | `/shutdown` | POST | Initiate graceful server shutdown |
//! | `/config` | GET | Current configuration (sanitized) |
//! | `/config/reload` | POST | Reload configuration from file |
//! | `/config/:section` | PATCH | Update a configuration section |

pub mod api;
pub mod auth;
pub mod config;
pub mod config_handlers;
pub mod handlers;
pub mod models;
pub mod propagation;
pub mod routes;
pub mod web;

// Re-export main types
pub use api::{AdminApi, AdminApiConfig};
pub use auth::{with_admin_auth, AdminAuthConfig, AuthErrorResponse};
pub use config::{
    ConfigReloadResponse, ConfigResponse, ConfigSectionUpdateRequest, ConfigSectionUpdateResponse,
    SanitizedConfig,
};
pub use handlers::AdminState;
pub use models::{AdminActionResponse, CollectionStats, ServerInfo};
pub use propagation::{ConfigPropagator, PropagatorHandle};
pub use routes::admin_router;
pub use web::web_router;
