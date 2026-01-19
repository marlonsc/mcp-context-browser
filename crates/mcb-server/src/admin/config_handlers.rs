//! Configuration Management Handlers
//!
//! HTTP handlers for runtime configuration management including
//! reading, updating, and reloading configuration.
//!
//! Migrated from Axum to Rocket in v0.1.2 (ADR-026).
//! Authentication guards added in v0.1.2.

use mcb_infrastructure::config::watcher::ConfigWatcher;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, get, patch, post};
use std::path::PathBuf;
use std::sync::Arc;

use super::auth::AdminAuth;
use super::config::{
    ConfigReloadResponse, ConfigResponse, ConfigSectionUpdateRequest, ConfigSectionUpdateResponse,
    SanitizedConfig,
};
use super::handlers::AdminState;

/// Internal error type for config update operations
enum ConfigUpdateError {
    InvalidSection,
    WatcherUnavailable,
    PathUnavailable,
    ReadFailed(String),
    ParseFailed(String),
    InvalidFormat,
    SerializeFailed(String),
    WriteFailed(String),
    ReloadFailed(String),
}

// ============================================================================
// Configuration Management Endpoints
// ============================================================================

/// Get current configuration (sanitized, protected)
///
/// Returns the current configuration with sensitive fields removed.
/// API keys, secrets, and passwords are not exposed.
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
#[get("/config")]
pub async fn get_config(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> (Status, Json<ConfigResponse>) {
    let Some(watcher) = &state.config_watcher else {
        return (
            Status::ServiceUnavailable,
            Json(ConfigResponse {
                success: false,
                config: SanitizedConfig::default(),
                config_path: None,
                last_reload: None,
            }),
        );
    };

    let config = watcher.get_config().await;
    let sanitized = SanitizedConfig::from_app_config(&config);

    (
        Status::Ok,
        Json(ConfigResponse {
            success: true,
            config: sanitized,
            config_path: state.config_path.as_ref().map(|p| p.display().to_string()),
            last_reload: Some(chrono::Utc::now().to_rfc3339()),
        }),
    )
}

/// Reload configuration from file (protected)
///
/// Triggers a manual configuration reload. The new configuration
/// will be validated before being applied.
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
#[post("/config/reload")]
pub async fn reload_config(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> (Status, Json<ConfigReloadResponse>) {
    let Some(watcher) = &state.config_watcher else {
        return (
            Status::ServiceUnavailable,
            Json(ConfigReloadResponse::watcher_unavailable()),
        );
    };

    match watcher.reload().await {
        Ok(new_config) => {
            let sanitized = SanitizedConfig::from_app_config(&new_config);
            (Status::Ok, Json(ConfigReloadResponse::success(sanitized)))
        }
        Err(e) => (
            Status::InternalServerError,
            Json(ConfigReloadResponse::failure(format!(
                "Failed to reload configuration: {}",
                e
            ))),
        ),
    }
}

/// Update a specific configuration section (protected)
///
/// Updates a configuration section by merging the provided values
/// with the existing configuration, then writing to the config file
/// and triggering a reload.
///
/// Valid sections: server, logging, cache, metrics, limits, resilience
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
#[patch("/config/<section>", format = "json", data = "<request>")]
pub async fn update_config_section(
    _auth: AdminAuth,
    state: &State<AdminState>,
    section: &str,
    request: Json<ConfigSectionUpdateRequest>,
) -> (Status, Json<ConfigSectionUpdateResponse>) {
    let request = request.into_inner();

    // Validate and get required resources
    let (watcher, config_path) = match validate_update_prerequisites(state, section) {
        Ok(resources) => resources,
        Err(e) => return config_update_error_response(section, e),
    };

    // Read and update configuration
    let updated_config = match read_update_config(&config_path, section, &request.values) {
        Ok(config) => config,
        Err(e) => return config_update_error_response(section, e),
    };

    // Write and reload
    match write_and_reload_config(&config_path, &updated_config, &watcher, section).await {
        Ok(sanitized) => (
            Status::Ok,
            Json(ConfigSectionUpdateResponse::success(section, sanitized)),
        ),
        Err(e) => config_update_error_response(section, e),
    }
}

/// Validate prerequisites for config update
fn validate_update_prerequisites(
    state: &AdminState,
    section: &str,
) -> Result<(Arc<ConfigWatcher>, PathBuf), ConfigUpdateError> {
    use super::config::is_valid_section;

    if !is_valid_section(section) {
        return Err(ConfigUpdateError::InvalidSection);
    }

    let watcher = state
        .config_watcher
        .as_ref()
        .ok_or(ConfigUpdateError::WatcherUnavailable)?;
    let config_path = state
        .config_path
        .as_ref()
        .ok_or(ConfigUpdateError::PathUnavailable)?;

    Ok((Arc::clone(watcher), config_path.clone()))
}

/// Read, parse, and update the configuration
fn read_update_config(
    config_path: &PathBuf,
    section: &str,
    values: &serde_json::Value,
) -> Result<toml::Value, ConfigUpdateError> {
    let content = std::fs::read_to_string(config_path)
        .map_err(|e| ConfigUpdateError::ReadFailed(e.to_string()))?;

    let mut config: toml::Value =
        toml::from_str(&content).map_err(|e| ConfigUpdateError::ParseFailed(e.to_string()))?;

    let toml_value = json_to_toml(values).ok_or(ConfigUpdateError::InvalidFormat)?;

    if let Some(table) = config.as_table_mut() {
        merge_section(table, section, toml_value);
    }

    Ok(config)
}

/// Merge new values into a config section
fn merge_section(
    table: &mut toml::map::Map<String, toml::Value>,
    section: &str,
    new_value: toml::Value,
) {
    let toml::Value::Table(new_table) = new_value else {
        table.insert(section.to_string(), new_value);
        return;
    };

    let Some(existing) = table.get_mut(section).and_then(|v| v.as_table_mut()) else {
        table.insert(section.to_string(), toml::Value::Table(new_table));
        return;
    };

    for (key, value) in new_table {
        existing.insert(key, value);
    }
}

/// Write config to file and reload
async fn write_and_reload_config(
    config_path: &PathBuf,
    config: &toml::Value,
    watcher: &ConfigWatcher,
    _section: &str,
) -> Result<SanitizedConfig, ConfigUpdateError> {
    let content = toml::to_string_pretty(config)
        .map_err(|e| ConfigUpdateError::SerializeFailed(e.to_string()))?;

    std::fs::write(config_path, content)
        .map_err(|e| ConfigUpdateError::WriteFailed(e.to_string()))?;

    let new_config = watcher
        .reload()
        .await
        .map_err(|e| ConfigUpdateError::ReloadFailed(e.to_string()))?;

    Ok(SanitizedConfig::from_app_config(&new_config))
}

/// Convert ConfigUpdateError to HTTP response
fn config_update_error_response(
    section: &str,
    error: ConfigUpdateError,
) -> (Status, Json<ConfigSectionUpdateResponse>) {
    match error {
        ConfigUpdateError::InvalidSection => {
            create_bad_request_response(section, "invalid_section")
        }
        ConfigUpdateError::WatcherUnavailable => {
            create_service_unavailable_response(section, "watcher_unavailable")
        }
        ConfigUpdateError::PathUnavailable => create_path_unavailable_response(section),
        ConfigUpdateError::ReadFailed(e) => create_read_error_response(section, e),
        ConfigUpdateError::ParseFailed(e) => create_parse_error_response(section, e),
        ConfigUpdateError::InvalidFormat => create_invalid_format_response(section),
        ConfigUpdateError::SerializeFailed(e) => create_serialize_error_response(section, e),
        ConfigUpdateError::WriteFailed(e) => create_write_error_response(section, e),
        ConfigUpdateError::ReloadFailed(e) => create_reload_error_response(section, e),
    }
}

/// Create bad request response
fn create_bad_request_response(
    section: &str,
    method: &str,
) -> (Status, Json<ConfigSectionUpdateResponse>) {
    let response = match method {
        "invalid_section" => ConfigSectionUpdateResponse::invalid_section(section),
        _ => ConfigSectionUpdateResponse::failure(section, "Bad request"),
    };
    (Status::BadRequest, Json(response))
}

/// Create service unavailable response
fn create_service_unavailable_response(
    section: &str,
    method: &str,
) -> (Status, Json<ConfigSectionUpdateResponse>) {
    let response = match method {
        "watcher_unavailable" => ConfigSectionUpdateResponse::watcher_unavailable(section),
        _ => ConfigSectionUpdateResponse::failure(section, "Service unavailable"),
    };
    (Status::ServiceUnavailable, Json(response))
}

/// Create path unavailable response
fn create_path_unavailable_response(section: &str) -> (Status, Json<ConfigSectionUpdateResponse>) {
    (
        Status::ServiceUnavailable,
        Json(ConfigSectionUpdateResponse::failure(
            section,
            "Configuration file path not available",
        )),
    )
}

/// Create read error response
fn create_read_error_response(
    section: &str,
    error: String,
) -> (Status, Json<ConfigSectionUpdateResponse>) {
    (
        Status::InternalServerError,
        Json(ConfigSectionUpdateResponse::failure(
            section,
            format!("Failed to read configuration file: {}", error),
        )),
    )
}

/// Create parse error response
fn create_parse_error_response(
    section: &str,
    error: String,
) -> (Status, Json<ConfigSectionUpdateResponse>) {
    (
        Status::InternalServerError,
        Json(ConfigSectionUpdateResponse::failure(
            section,
            format!("Failed to parse configuration file: {}", error),
        )),
    )
}

/// Create invalid format response
fn create_invalid_format_response(section: &str) -> (Status, Json<ConfigSectionUpdateResponse>) {
    (
        Status::BadRequest,
        Json(ConfigSectionUpdateResponse::failure(
            section,
            "Invalid configuration value format",
        )),
    )
}

/// Create serialize error response
fn create_serialize_error_response(
    section: &str,
    error: String,
) -> (Status, Json<ConfigSectionUpdateResponse>) {
    (
        Status::InternalServerError,
        Json(ConfigSectionUpdateResponse::failure(
            section,
            format!("Failed to serialize configuration: {}", error),
        )),
    )
}

/// Create write error response
fn create_write_error_response(
    section: &str,
    error: String,
) -> (Status, Json<ConfigSectionUpdateResponse>) {
    (
        Status::InternalServerError,
        Json(ConfigSectionUpdateResponse::failure(
            section,
            format!("Failed to write configuration file: {}", error),
        )),
    )
}

/// Create reload error response
fn create_reload_error_response(
    section: &str,
    error: String,
) -> (Status, Json<ConfigSectionUpdateResponse>) {
    (
        Status::InternalServerError,
        Json(ConfigSectionUpdateResponse::failure(
            section,
            format!("Configuration updated but reload failed: {}", error),
        )),
    )
}

/// Convert a JSON value to a TOML value
fn json_to_toml(json: &serde_json::Value) -> Option<toml::Value> {
    match json {
        serde_json::Value::Null => Some(toml::Value::String(String::new())),
        serde_json::Value::Bool(b) => Some(toml::Value::Boolean(*b)),
        serde_json::Value::Number(n) => n
            .as_i64()
            .map(toml::Value::Integer)
            .or_else(|| n.as_f64().map(toml::Value::Float)),
        serde_json::Value::String(s) => Some(toml::Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let toml_arr: Option<Vec<toml::Value>> = arr.iter().map(json_to_toml).collect();
            toml_arr.map(toml::Value::Array)
        }
        serde_json::Value::Object(obj) => {
            let mut table = toml::map::Map::new();
            for (k, v) in obj {
                table.insert(k.clone(), json_to_toml(v)?);
            }
            Some(toml::Value::Table(table))
        }
    }
}
