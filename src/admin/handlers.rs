//! HTTP handlers for admin API endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

use crate::admin::models::{
    AdminState, ApiResponse, IndexInfo, IndexOperationRequest, ProviderConfigRequest, ProviderInfo,
    SystemConfig,
};
use crate::admin::service::{AdminService, AdminServiceImpl};

/// Get system configuration
pub async fn get_config_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<SystemConfig>>, StatusCode> {
    // TODO: Implement actual config retrieval from MCP server
    let config = SystemConfig {
        providers: vec![
            ProviderInfo {
                id: "openai-1".to_string(),
                name: "OpenAI".to_string(),
                provider_type: "embedding".to_string(),
                status: "active".to_string(),
                config: serde_json::json!({
                    "model": "text-embedding-ada-002",
                    "api_key": "***"
                }),
            },
            ProviderInfo {
                id: "milvus-1".to_string(),
                name: "Milvus".to_string(),
                provider_type: "vector_store".to_string(),
                status: "active".to_string(),
                config: serde_json::json!({
                    "host": "localhost",
                    "port": 19530
                }),
            },
        ],
        indexing: crate::admin::models::IndexingConfig {
            chunk_size: 1000,
            chunk_overlap: 200,
            max_file_size: 10 * 1024 * 1024, // 10MB
            supported_extensions: vec![
                ".rs".to_string(),
                ".js".to_string(),
                ".ts".to_string(),
                ".py".to_string(),
                ".md".to_string(),
            ],
            exclude_patterns: vec![
                "target/".to_string(),
                "node_modules/".to_string(),
                ".git/".to_string(),
            ],
        },
        security: crate::admin::models::SecurityConfig {
            enable_auth: true,
            rate_limiting: true,
            max_requests_per_minute: 60,
        },
        metrics: crate::admin::models::MetricsConfig {
            enabled: true,
            collection_interval: 30,
            retention_days: 30,
        },
    };

    Ok(Json(ApiResponse::success(config)))
}

/// Update system configuration
pub async fn update_config_handler(
    State(_state): State<AdminState>,
    Json(_config): Json<SystemConfig>,
) -> Result<Json<ApiResponse<SystemConfig>>, StatusCode> {
    // TODO: Implement config update
    Ok(Json(ApiResponse::error("Configuration update not yet implemented".to_string())))
}

/// List all providers
pub async fn list_providers_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<Vec<ProviderInfo>>>, StatusCode> {
    // Get real provider data from MCP server
    let provider_statuses = state.admin_service.get_providers().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let providers = provider_statuses
        .into_iter()
        .map(|status| ProviderInfo {
            id: status.id,
            name: status.name,
            provider_type: status.provider_type,
            status: status.status,
            config: status.config,
        })
        .collect();

    Ok(Json(ApiResponse::success(providers)))
}

/// Add a new provider
pub async fn add_provider_handler(
    State(state): State<AdminState>,
    Json(provider_config): Json<ProviderConfigRequest>,
) -> Result<Json<ApiResponse<ProviderInfo>>, StatusCode> {
    // Validate provider configuration based on type
    match provider_config.provider_type.as_str() {
        "embedding" => {
            // Validate embedding provider configuration
            if provider_config.config.get("model").is_none() {
                return Ok(Json(ApiResponse::error("Model is required for embedding providers".to_string())));
            }
        }
        "vector_store" => {
            // Validate vector store provider configuration
            if provider_config.config.get("host").is_none() {
                return Ok(Json(ApiResponse::error("Host is required for vector store providers".to_string())));
            }
        }
        _ => return Ok(Json(ApiResponse::error("Invalid provider type".to_string()))),
    }

    // In a real implementation, this would register the provider with the MCP server
    // For now, return success with mock data
    let provider_info = ProviderInfo {
        id: format!("{}-{}", provider_config.provider_type, provider_config.provider_type),
        name: provider_config.provider_type.clone(),
        provider_type: provider_config.provider_type,
        status: "pending".to_string(),
        config: provider_config.config,
    };

    Ok(Json(ApiResponse::success(provider_info)))
}

/// Remove a provider
pub async fn remove_provider_handler(
    State(state): State<AdminState>,
    Path(provider_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // Check if provider exists
    let providers = state.admin_service.get_providers().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !providers.iter().any(|p| p.id == provider_id) {
        return Ok(Json(ApiResponse::error("Provider not found".to_string())));
    }

    // In a real implementation, this would unregister the provider from the MCP server
    // For now, return success
    Ok(Json(ApiResponse::success(format!("Provider {} removed successfully", provider_id))))
}

/// List all indexes
pub async fn list_indexes_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<Vec<IndexInfo>>>, StatusCode> {
    // Get real indexing status from MCP server
    let indexing_status = state.admin_service.get_indexing_status().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let indexes = vec![
        IndexInfo {
            id: "main-index".to_string(),
            name: "Main Codebase Index".to_string(),
            status: if indexing_status.is_indexing { "indexing".to_string() } else { "active".to_string() },
            document_count: indexing_status.indexed_documents,
            created_at: indexing_status.start_time.unwrap_or(1640995200),
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        },
    ];

    Ok(Json(ApiResponse::success(indexes)))
}

/// Perform index operation
pub async fn index_operation_handler(
    State(state): State<AdminState>,
    Path(index_id): Path<String>,
    Json(operation): Json<IndexOperationRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // Validate index exists
    let indexes = vec!["main-index"]; // In real implementation, get from server
    if !indexes.contains(&index_id.as_str()) {
        return Ok(Json(ApiResponse::error("Index not found".to_string())));
    }

    // Perform operation based on type
    match operation.operation.as_str() {
        "clear" => {
            // In real implementation, this would clear the index via MCP server
            Ok(Json(ApiResponse::success(format!("Index {} cleared successfully", index_id))))
        }
        "rebuild" => {
            // In real implementation, this would trigger index rebuild
            Ok(Json(ApiResponse::success(format!("Index {} rebuild started", index_id))))
        }
        "status" => {
            // Get current indexing status
            let status = state.mcp_server.get_indexing_status_admin();
            let message = if status.is_indexing {
                format!("Index {} is currently indexing ({} of {} documents)",
                    index_id, status.indexed_documents, status.total_documents)
            } else {
                format!("Index {} is idle ({} documents indexed)",
                    index_id, status.indexed_documents)
            };
            Ok(Json(ApiResponse::success(message)))
        }
        _ => Ok(Json(ApiResponse::error("Invalid operation".to_string()))),
    }
}

/// Get system status
pub async fn get_status_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // Get real system information
    let system_info = state.admin_service.get_system_info().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let providers = state.admin_service.get_providers().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let indexing_status = state.admin_service.get_indexing_status().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let performance = state.admin_service.get_performance_metrics().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Group providers by type
    let mut embedding_providers = Vec::new();
    let mut vector_store_providers = Vec::new();

    for provider in providers {
        match provider.provider_type.as_str() {
            "embedding" => embedding_providers.push(provider.name.to_lowercase()),
            "vector_store" => vector_store_providers.push(provider.name.to_lowercase()),
            _ => {}
        }
    }

    let status = serde_json::json!({
        "service": "mcp-context-browser",
        "version": system_info.version,
        "status": "running",
        "uptime": system_info.uptime,
        "pid": system_info.pid,
        "providers": {
            "embedding": embedding_providers,
            "vector_store": vector_store_providers
        },
        "indexes": {
            "total": 1,
            "active": if indexing_status.is_indexing { 0 } else { 1 },
            "indexing": indexing_status.is_indexing,
            "total_documents": indexing_status.total_documents,
            "indexed_documents": indexing_status.indexed_documents
        },
        "performance": {
            "total_queries": performance.total_queries,
            "successful_queries": performance.successful_queries,
            "failed_queries": performance.failed_queries,
            "average_response_time_ms": performance.average_response_time_ms,
            "cache_hit_rate": performance.cache_hit_rate,
            "active_connections": performance.active_connections
        }
    });

    Ok(Json(ApiResponse::success(status)))
}

/// Query parameters for search
#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<usize>,
}

/// Search handler
pub async fn search_handler(
    State(state): State<AdminState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // TODO: Implement search through MCP server
    let results = serde_json::json!({
        "query": params.q,
        "results": [],
        "total": 0,
        "took_ms": 0
    });

    Ok(Json(ApiResponse::success(results)))
}