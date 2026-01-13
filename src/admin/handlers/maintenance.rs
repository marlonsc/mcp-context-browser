//! Maintenance operation handlers

use super::common::*;
use super::{CleanupRequest, ExportQuery};
use crate::admin::web::html_helpers::*;
use crate::infrastructure::utils::IntoStatusCode;

/// Get system logs with filtering
pub async fn get_logs_handler(
    State(state): State<AdminState>,
    Query(filter): Query<crate::admin::service::LogFilter>,
) -> Result<Json<ApiResponse<crate::admin::service::LogEntries>>, StatusCode> {
    let logs = state.admin_service.get_logs(filter).await.to_500()?;

    Ok(Json(ApiResponse::success(logs)))
}

/// Export logs to file
pub async fn export_logs_handler(
    State(state): State<AdminState>,
    Query(filter): Query<crate::admin::service::LogFilter>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let filename = state
        .admin_service
        .export_logs(filter, params.format)
        .await
        .to_500()?;

    Ok(Json(ApiResponse::success(filename)))
}

/// Get log statistics
pub async fn get_log_stats_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<crate::admin::service::LogStats>>, StatusCode> {
    let stats = state.admin_service.get_log_stats().await.to_500()?;

    Ok(Json(ApiResponse::success(stats)))
}

/// Clear system cache
pub async fn clear_cache_handler(
    State(state): State<AdminState>,
    Path(cache_type): Path<String>,
) -> Result<Json<ApiResponse<crate::admin::service::MaintenanceResult>>, StatusCode> {
    let cache_type_enum = match cache_type.as_str() {
        "all" => crate::admin::service::CacheType::All,
        "query" => crate::admin::service::CacheType::QueryResults,
        "embeddings" => crate::admin::service::CacheType::Embeddings,
        "indexes" => crate::admin::service::CacheType::Indexes,
        _ => return Ok(Json(ApiResponse::error("Invalid cache type".to_string()))),
    };

    let result = state
        .admin_service
        .clear_cache(cache_type_enum)
        .await
        .to_500()?;

    Ok(Json(ApiResponse::success(result)))
}

/// Restart provider connection
pub async fn restart_provider_handler(
    State(state): State<AdminState>,
    Path(provider_id): Path<String>,
) -> Result<Json<ApiResponse<crate::admin::service::MaintenanceResult>>, StatusCode> {
    let result = state
        .admin_service
        .restart_provider(&provider_id)
        .await
        .to_500()?;

    Ok(Json(ApiResponse::success(result)))
}

/// Rebuild search index
pub async fn rebuild_index_handler(
    State(state): State<AdminState>,
    Path(index_id): Path<String>,
) -> Result<Json<ApiResponse<crate::admin::service::MaintenanceResult>>, StatusCode> {
    let result = state
        .admin_service
        .rebuild_index(&index_id)
        .await
        .to_500()?;

    Ok(Json(ApiResponse::success(result)))
}

/// Cleanup old data
pub async fn cleanup_data_handler(
    State(state): State<AdminState>,
    Json(cleanup_config): Json<crate::admin::service::CleanupConfig>,
) -> Result<Json<ApiResponse<crate::admin::service::MaintenanceResult>>, StatusCode> {
    let result = state
        .admin_service
        .cleanup_data(cleanup_config)
        .await
        .to_500()?;

    Ok(Json(ApiResponse::success(result)))
}

// ============================================================================
// Simplified API Handlers (for HTMX buttons)
// ============================================================================

/// Clear cache by type (simplified endpoint for HTMX)
pub async fn api_clear_cache_handler(
    State(state): State<AdminState>,
    Path(cache_type): Path<String>,
) -> Html<String> {
    let cache_type_enum = match cache_type.as_str() {
        "all" => crate::admin::service::CacheType::All,
        "query" => crate::admin::service::CacheType::QueryResults,
        "embeddings" => crate::admin::service::CacheType::Embeddings,
        "indexes" => crate::admin::service::CacheType::Indexes,
        _ => return html_error(format!("Invalid cache type: {}", cache_type)),
    };

    match state.admin_service.clear_cache(cache_type_enum).await {
        Ok(result) => html_success(&result.message),
        Err(e) => html_error(format!("Error: {}", e)),
    }
}

/// Restart all providers (simplified endpoint for HTMX)
pub async fn api_restart_all_providers_handler(State(state): State<AdminState>) -> Html<String> {
    let providers = match state.admin_service.get_providers().await {
        Ok(p) => p,
        Err(e) => return html_error(format!("Error getting providers: {}", e)),
    };

    let provider_list: Vec<(String, String)> = providers
        .into_iter()
        .map(|p| (p.provider_type, p.id))
        .collect();

    match crate::admin::service::helpers::maintenance::restart_all_providers(
        &state.event_bus,
        &provider_list,
    )
    .await
    {
        Ok(result) => html_success(&result.message),
        Err(e) => html_error(&format!("Error: {}", e)),
    }
}

/// Restart providers by type (simplified endpoint for HTMX)
pub async fn api_restart_providers_by_type_handler(
    State(state): State<AdminState>,
    Path(provider_type): Path<String>,
) -> Html<String> {
    let providers = match state.admin_service.get_providers().await {
        Ok(p) => p,
        Err(e) => return html_error(format!("Error getting providers: {}", e)),
    };

    let provider_list: Vec<(String, String)> = providers
        .into_iter()
        .filter(|p| p.provider_type == provider_type)
        .map(|p| (p.provider_type, p.id))
        .collect();

    if provider_list.is_empty() {
        return html_warning(format!("No {} providers found", provider_type));
    }

    match crate::admin::service::helpers::maintenance::restart_all_providers(
        &state.event_bus,
        &provider_list,
    )
    .await
    {
        Ok(result) => html_success(&result.message),
        Err(e) => html_error(format!("Error: {}", e)),
    }
}

/// Reconfigure a provider without restart
pub async fn api_reconfigure_provider_handler(
    State(state): State<AdminState>,
    Path((provider_type, provider_id)): Path<(String, String)>,
    Json(config): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<MaintenanceResult>>, StatusCode> {
    let full_provider_id = format!("{}:{}", provider_type, provider_id);

    match state
        .admin_service
        .reconfigure_provider(&full_provider_id, config)
        .await
    {
        Ok(result) => {
            tracing::info!(
                "[ADMIN] Provider reconfiguration successful for {}",
                full_provider_id
            );
            Ok(Json(ApiResponse::success(result)))
        }
        Err(e) => {
            tracing::error!("[ADMIN] Provider reconfiguration failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Rebuild all indexes (simplified endpoint for HTMX)
pub async fn api_rebuild_indexes_handler(State(state): State<AdminState>) -> Html<String> {
    match state.admin_service.rebuild_index("default").await {
        Ok(result) => html_success(&result.message),
        Err(e) => html_error(format!("Error: {}", e)),
    }
}

/// Optimize indexes (simplified endpoint for HTMX)
pub async fn api_optimize_indexes_handler(State(_state): State<AdminState>) -> Html<String> {
    html_warning("Index optimization requested. This is a placeholder for future implementation.")
}

/// Clear all indexes (simplified endpoint for HTMX)
pub async fn api_clear_indexes_handler(State(state): State<AdminState>) -> Html<String> {
    match state.admin_service.rebuild_index("__clear__").await {
        Ok(result) => html_success(&result.message),
        Err(e) => html_error(format!("Error: {}", e)),
    }
}

/// Cleanup old data (simplified endpoint for HTMX)
pub async fn api_cleanup_handler(
    State(state): State<AdminState>,
    Query(params): Query<CleanupRequest>,
) -> Html<String> {
    let cleanup_config = crate::admin::service::CleanupConfig {
        older_than_days: params.older_than_days.unwrap_or(30),
        max_items_to_keep: None,
        cleanup_types: vec!["logs".to_string(), "exports".to_string()],
    };

    match state.admin_service.cleanup_data(cleanup_config).await {
        Ok(result) => html_success(&result.message),
        Err(e) => html_error(format!("Error: {}", e)),
    }
}
