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
    // Get providers from MCP server
    let (embedding_providers, vector_store_providers) = state.mcp_server.list_providers();

    let mut providers = Vec::new();

    // Add embedding providers
    for name in embedding_providers {
        providers.push(ProviderInfo {
            id: format!("embedding-{}", name),
            name: name.clone(),
            provider_type: "embedding".to_string(),
            status: "active".to_string(),
            config: serde_json::json!({ "name": name }),
        });
    }

    // Add vector store providers
    for name in vector_store_providers {
        providers.push(ProviderInfo {
            id: format!("vector-store-{}", name),
            name: name.clone(),
            provider_type: "vector_store".to_string(),
            status: "active".to_string(),
            config: serde_json::json!({ "name": name }),
        });
    }

    Ok(Json(ApiResponse::success(providers)))
}

/// Add a new provider
pub async fn add_provider_handler(
    State(_state): State<AdminState>,
    Json(_provider_config): Json<ProviderConfigRequest>,
) -> Result<Json<ApiResponse<ProviderInfo>>, StatusCode> {
    // TODO: Implement provider addition
    Ok(Json(ApiResponse::error("Provider addition not yet implemented".to_string())))
}

/// Remove a provider
pub async fn remove_provider_handler(
    State(_state): State<AdminState>,
    Path(_provider_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // TODO: Implement provider removal
    Ok(Json(ApiResponse::error("Provider removal not yet implemented".to_string())))
}

/// List all indexes
pub async fn list_indexes_handler(
    State(_state): State<AdminState>,
) -> Result<Json<ApiResponse<Vec<IndexInfo>>>, StatusCode> {
    // TODO: Implement index listing
    let indexes = vec![
        IndexInfo {
            id: "main-index".to_string(),
            name: "Main Codebase Index".to_string(),
            status: "active".to_string(),
            document_count: 1500,
            created_at: 1640995200, // 2022-01-01
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
    State(_state): State<AdminState>,
    Path(_index_id): Path<String>,
    Json(_operation): Json<IndexOperationRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // TODO: Implement index operations
    Ok(Json(ApiResponse::error("Index operations not yet implemented".to_string())))
}

/// Get system status
pub async fn get_status_handler(
    State(_state): State<AdminState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let status = serde_json::json!({
        "service": "mcp-context-browser",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running",
        "uptime": 3600, // TODO: Get actual uptime
        "providers": {
            "embedding": ["openai", "ollama"],
            "vector_store": ["milvus", "in-memory"]
        },
        "indexes": {
            "total": 1,
            "active": 1
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