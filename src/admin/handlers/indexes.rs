//! Index management handlers

use super::common::*;
use crate::infrastructure::utils::{IntoStatusCode, TimeUtils};

/// List all indexes
pub async fn list_indexes_handler(
    State(state): State<AdminState>,
) -> Result<Json<ApiResponse<Vec<IndexInfo>>>, StatusCode> {
    let indexing_status = state.admin_service.get_indexing_status().await.to_500()?;

    let indexes = vec![IndexInfo {
        id: "main-index".to_string(),
        name: "Main Codebase Index".to_string(),
        status: if indexing_status.is_indexing {
            "indexing".to_string()
        } else {
            "active".to_string()
        },
        document_count: indexing_status.indexed_documents,
        created_at: indexing_status.start_time.unwrap_or(1640995200),
        updated_at: TimeUtils::now_unix_secs(),
    }];

    Ok(Json(ApiResponse::success(indexes)))
}

/// Perform index operation
pub async fn index_operation_handler(
    State(state): State<AdminState>,
    Path(index_id): Path<String>,
    Json(operation): Json<IndexOperationRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    use crate::infrastructure::events::SystemEvent;

    match operation.operation.as_str() {
        "clear" => {
            if let Err(e) = state
                .event_bus
                .publish(SystemEvent::IndexClear {
                    collection: Some(index_id.clone()),
                })
                .await
            {
                tracing::error!("Failed to publish IndexClear event: {}", e);
                return Ok(Json(ApiResponse::error(format!(
                    "Failed to clear index: {}",
                    e
                ))));
            }

            if let Err(e) = state
                .event_bus
                .publish(SystemEvent::CacheClear {
                    namespace: Some("indexes".to_string()),
                })
                .await
            {
                tracing::warn!("Failed to clear index cache: {}", e);
            }

            tracing::info!("[ADMIN] Index clear requested for: {}", index_id);
            Ok(Json(ApiResponse::success(format!(
                "Index {} clear initiated. The operation is running asynchronously.",
                index_id
            ))))
        }
        "rebuild" => {
            if let Err(e) = state
                .event_bus
                .publish(SystemEvent::IndexRebuild {
                    collection: Some(index_id.clone()),
                })
                .await
            {
                tracing::error!("Failed to publish IndexRebuild event: {}", e);
                return Ok(Json(ApiResponse::error(format!(
                    "Failed to start index rebuild: {}",
                    e
                ))));
            }

            tracing::info!("[ADMIN] Index rebuild requested for: {}", index_id);
            Ok(Json(ApiResponse::success(format!(
                "Index {} rebuild initiated. The operation is running asynchronously.",
                index_id
            ))))
        }
        "optimize" => {
            if let Err(e) = state
                .event_bus
                .publish(SystemEvent::IndexOptimize {
                    collection: Some(index_id.clone()),
                })
                .await
            {
                tracing::error!("Failed to publish IndexOptimize event: {}", e);
                return Ok(Json(ApiResponse::error(format!(
                    "Failed to optimize index: {}",
                    e
                ))));
            }

            tracing::info!("[ADMIN] Index optimization requested for: {}", index_id);
            Ok(Json(ApiResponse::success(format!(
                "Index {} optimization initiated. The operation is running asynchronously.",
                index_id
            ))))
        }
        "status" => {
            let status = state.mcp_server.get_indexing_status_admin().await;
            let message = if status.is_indexing {
                format!(
                    "Index {} is currently indexing ({} of {} documents)",
                    index_id, status.indexed_documents, status.total_documents
                )
            } else {
                format!(
                    "Index {} is idle ({} documents indexed)",
                    index_id, status.indexed_documents
                )
            };
            Ok(Json(ApiResponse::success(message)))
        }
        _ => Ok(Json(ApiResponse::error(format!(
            "Invalid operation '{}'. Valid operations: clear, rebuild, optimize, status",
            operation.operation
        )))),
    }
}
