//! Service Lifecycle HTTP Handlers
//!
//! HTTP handlers for service lifecycle management endpoints.
//! These endpoints allow starting, stopping, and restarting
//! services via the ServiceManager.
//!
//! ## Endpoints
//!
//! | Path | Method | Description |
//! |------|--------|-------------|
//! | `/services` | GET | List all registered services and their states (protected) |
//! | `/services/{name}/start` | POST | Start a specific service (protected) |
//! | `/services/{name}/stop` | POST | Stop a specific service (protected) |
//! | `/services/{name}/restart` | POST | Restart a specific service (protected) |
//!
//! Migrated from Axum to Rocket in v0.1.2 (ADR-026).
//! Authentication guards added in v0.1.2.

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, get, post};
use serde::Serialize;
use serde_json::json;

use super::auth::AdminAuth;
use super::handlers::AdminState;

/// Response for service list endpoint
#[derive(Serialize)]
pub struct ServiceListResponse {
    /// Number of registered services
    pub count: usize,
    /// List of services with their states
    pub services: Vec<ServiceInfoResponse>,
}

/// Individual service info in the list
#[derive(Serialize)]
pub struct ServiceInfoResponse {
    /// Service name
    pub name: String,
    /// Current state as string
    pub state: String,
}

/// Service error response
#[derive(Serialize)]
pub struct ServiceErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub services: Option<Vec<ServiceInfoResponse>>,
}

/// Service action response
#[derive(Serialize)]
pub struct ServiceActionResponse {
    pub status: String,
    pub service: String,
}

/// List all registered services and their states (protected)
///
/// GET /admin/services
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
#[get("/services")]
pub fn list_services(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> Result<Json<ServiceListResponse>, (Status, Json<ServiceErrorResponse>)> {
    let Some(service_manager) = &state.service_manager else {
        return Err((
            Status::ServiceUnavailable,
            Json(ServiceErrorResponse {
                error: "Service manager not available".to_string(),
                service: None,
                count: Some(0),
                services: Some(vec![]),
            }),
        ));
    };

    let services: Vec<ServiceInfoResponse> = service_manager
        .list()
        .into_iter()
        .map(|info| ServiceInfoResponse {
            name: info.name,
            state: format!("{:?}", info.state),
        })
        .collect();

    Ok(Json(ServiceListResponse {
        count: services.len(),
        services,
    }))
}

/// Start a specific service (protected)
///
/// POST /admin/services/{name}/start
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
#[post("/services/<name>/start")]
pub async fn start_service(
    _auth: AdminAuth,
    state: &State<AdminState>,
    name: &str,
) -> Result<(Status, Json<ServiceActionResponse>), (Status, Json<ServiceErrorResponse>)> {
    let Some(service_manager) = &state.service_manager else {
        return Err((
            Status::ServiceUnavailable,
            Json(ServiceErrorResponse {
                error: "Service manager not available".to_string(),
                service: None,
                count: None,
                services: None,
            }),
        ));
    };

    match service_manager.start(name).await {
        Ok(()) => Ok((
            Status::Ok,
            Json(ServiceActionResponse {
                status: "started".to_string(),
                service: name.to_string(),
            }),
        )),
        Err(e) => Err((
            Status::BadRequest,
            Json(ServiceErrorResponse {
                error: e.to_string(),
                service: Some(name.to_string()),
                count: None,
                services: None,
            }),
        )),
    }
}

/// Stop a specific service (protected)
///
/// POST /admin/services/{name}/stop
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
#[post("/services/<name>/stop")]
pub async fn stop_service(
    _auth: AdminAuth,
    state: &State<AdminState>,
    name: &str,
) -> Result<(Status, Json<ServiceActionResponse>), (Status, Json<ServiceErrorResponse>)> {
    let Some(service_manager) = &state.service_manager else {
        return Err((
            Status::ServiceUnavailable,
            Json(ServiceErrorResponse {
                error: "Service manager not available".to_string(),
                service: None,
                count: None,
                services: None,
            }),
        ));
    };

    match service_manager.stop(name).await {
        Ok(()) => Ok((
            Status::Ok,
            Json(ServiceActionResponse {
                status: "stopped".to_string(),
                service: name.to_string(),
            }),
        )),
        Err(e) => Err((
            Status::BadRequest,
            Json(ServiceErrorResponse {
                error: e.to_string(),
                service: Some(name.to_string()),
                count: None,
                services: None,
            }),
        )),
    }
}

/// Restart a specific service (protected)
///
/// POST /admin/services/{name}/restart
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
#[post("/services/<name>/restart")]
pub async fn restart_service(
    _auth: AdminAuth,
    state: &State<AdminState>,
    name: &str,
) -> Result<(Status, Json<ServiceActionResponse>), (Status, Json<ServiceErrorResponse>)> {
    let Some(service_manager) = &state.service_manager else {
        return Err((
            Status::ServiceUnavailable,
            Json(ServiceErrorResponse {
                error: "Service manager not available".to_string(),
                service: None,
                count: None,
                services: None,
            }),
        ));
    };

    match service_manager.restart(name).await {
        Ok(()) => Ok((
            Status::Ok,
            Json(ServiceActionResponse {
                status: "restarted".to_string(),
                service: name.to_string(),
            }),
        )),
        Err(e) => Err((
            Status::BadRequest,
            Json(ServiceErrorResponse {
                error: e.to_string(),
                service: Some(name.to_string()),
                count: None,
                services: None,
            }),
        )),
    }
}

/// Services health response
#[derive(Serialize)]
pub struct ServicesHealthResponse {
    pub count: usize,
    pub checks: Vec<serde_json::Value>,
}

/// Get health check for all services (protected)
///
/// GET /admin/services/health
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
#[get("/services/health")]
pub async fn services_health(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> Result<Json<ServicesHealthResponse>, (Status, Json<ServiceErrorResponse>)> {
    let Some(service_manager) = &state.service_manager else {
        return Err((
            Status::ServiceUnavailable,
            Json(ServiceErrorResponse {
                error: "Service manager not available".to_string(),
                service: None,
                count: None,
                services: None,
            }),
        ));
    };

    let checks = service_manager.health_check_all().await;
    let checks_json: Vec<serde_json::Value> = checks
        .iter()
        .map(|c| serde_json::to_value(c).unwrap_or(json!({})))
        .collect();

    Ok(Json(ServicesHealthResponse {
        count: checks.len(),
        checks: checks_json,
    }))
}
