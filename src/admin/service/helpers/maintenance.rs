//! Maintenance operations helper module
//!
//! Provides functions for cache management, provider restart, index rebuilding, and data cleanup.

use crate::admin::service::types::{AdminError, CacheType, CleanupConfig, MaintenanceResult};
use crate::infrastructure::events::SharedEventBus;
use crate::infrastructure::logging::SharedLogBuffer;

/// Clear cache by type
pub fn clear_cache(
    event_bus: &SharedEventBus,
    cache_type: CacheType,
) -> Result<MaintenanceResult, AdminError> {
    let start_time = std::time::Instant::now();
    let namespace = match cache_type {
        CacheType::All => None,
        CacheType::QueryResults => Some("search_results".to_string()),
        CacheType::Embeddings => Some("embeddings".to_string()),
        CacheType::Indexes => Some("indexes".to_string()),
    };

    event_bus
        .publish(crate::infrastructure::events::SystemEvent::CacheClear {
            namespace: namespace.clone(),
        })
        .map_err(|e| {
            AdminError::McpServerError(format!("Failed to publish CacheClear event: {}", e))
        })?;

    Ok(MaintenanceResult {
        success: true,
        operation: format!("clear_cache_{:?}", cache_type),
        message: format!("Successfully requested cache clear for {:?}", cache_type),
        affected_items: 0,
        execution_time_ms: start_time.elapsed().as_millis() as u64,
    })
}

/// Request provider restart by publishing a restart event
///
/// The provider_id should be in the format "type:id" (e.g., "embedding:ollama" or "vector_store:milvus").
/// If no type prefix is provided, it defaults to treating it as a generic provider ID.
///
/// The actual restart is handled asynchronously by the RecoveryManager which listens
/// for ProviderRestart events.
pub fn restart_provider(
    event_bus: &SharedEventBus,
    provider_id: &str,
) -> Result<MaintenanceResult, AdminError> {
    let start_time = std::time::Instant::now();

    // Parse provider_id to extract type and id
    // Format: "type:id" or just "id"
    let (provider_type, actual_id): (String, String) = if provider_id.contains(':') {
        let parts: Vec<&str> = provider_id.splitn(2, ':').collect();
        (
            parts[0].to_string(),
            parts.get(1).map(|s| s.to_string()).unwrap_or_default(),
        )
    } else {
        // Try to determine type from common provider names
        let provider_type = match provider_id.to_lowercase().as_str() {
            "ollama" | "openai" | "voyageai" | "gemini" | "fastembed" | "null_embedding" => {
                "embedding"
            }
            "milvus" | "edgevec" | "filesystem" | "encrypted" | "in_memory" | "null_vector" => {
                "vector_store"
            }
            _ => "unknown",
        };
        (provider_type.to_string(), provider_id.to_string())
    };

    tracing::info!(
        "[ADMIN] Requesting restart for provider: {} (type: {}, id: {})",
        provider_id,
        provider_type,
        actual_id
    );

    // Publish restart event
    event_bus
        .publish(crate::infrastructure::events::SystemEvent::ProviderRestart {
            provider_type: provider_type.clone(),
            provider_id: actual_id.clone(),
        })
        .map_err(|e| {
            AdminError::McpServerError(format!("Failed to publish ProviderRestart event: {}", e))
        })?;

    Ok(MaintenanceResult {
        success: true,
        operation: "restart_provider".to_string(),
        message: format!(
            "Restart signal sent for provider '{}' (type: {}). \
             The RecoveryManager will handle the restart asynchronously.",
            actual_id, provider_type
        ),
        affected_items: 1,
        execution_time_ms: start_time.elapsed().as_millis() as u64,
    })
}

/// Restart all providers of a specific type or all providers
pub fn restart_all_providers(
    event_bus: &SharedEventBus,
    provider_names: &[(String, String)], // Vec of (type, id)
) -> Result<MaintenanceResult, AdminError> {
    let start_time = std::time::Instant::now();
    let mut restarted = 0;

    for (provider_type, provider_id) in provider_names {
        if let Err(e) = event_bus.publish(crate::infrastructure::events::SystemEvent::ProviderRestart {
            provider_type: provider_type.clone(),
            provider_id: provider_id.clone(),
        }) {
            tracing::warn!(
                "[ADMIN] Failed to publish restart for {}:{}: {}",
                provider_type,
                provider_id,
                e
            );
        } else {
            restarted += 1;
        }
    }

    tracing::info!(
        "[ADMIN] Requested restart for {} providers",
        restarted
    );

    Ok(MaintenanceResult {
        success: true,
        operation: "restart_all_providers".to_string(),
        message: format!(
            "Restart signals sent for {} providers. \
             The RecoveryManager will handle restarts asynchronously.",
            restarted
        ),
        affected_items: restarted,
        execution_time_ms: start_time.elapsed().as_millis() as u64,
    })
}

/// Request index rebuild
pub fn rebuild_index(
    event_bus: &SharedEventBus,
    index_id: &str,
) -> Result<MaintenanceResult, AdminError> {
    let start_time = std::time::Instant::now();
    event_bus
        .publish(crate::infrastructure::events::SystemEvent::IndexRebuild {
            collection: Some(index_id.to_string()),
        })
        .map_err(|e| {
            AdminError::McpServerError(format!("Failed to publish IndexRebuild event: {}", e))
        })?;

    Ok(MaintenanceResult {
        success: true,
        operation: "rebuild_index".to_string(),
        message: format!("Successfully requested rebuild for index {}", index_id),
        affected_items: 0,
        execution_time_ms: start_time.elapsed().as_millis() as u64,
    })
}

/// Clean up old data based on configuration
pub async fn cleanup_data(
    log_buffer: &SharedLogBuffer,
    cleanup_config: CleanupConfig,
) -> Result<MaintenanceResult, AdminError> {
    let start_time = std::time::Instant::now();
    let mut affected_items = 0;

    for cleanup_type in &cleanup_config.cleanup_types {
        match cleanup_type.as_str() {
            "logs" => {
                let count = log_buffer.get_all().await.len();
                log_buffer.clear();
                affected_items += count as u64;
                tracing::info!("[ADMIN] Cleared {} log entries from buffer", count);
            }
            "exports" => {
                let export_dir = std::path::PathBuf::from("./exports");
                if export_dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(export_dir) {
                        for entry in entries.flatten() {
                            if let Ok(metadata) = entry.metadata() {
                                let created =
                                    metadata.created().unwrap_or(std::time::SystemTime::now());
                                let age = std::time::SystemTime::now()
                                    .duration_since(created)
                                    .unwrap_or_default();
                                if age.as_secs() > (cleanup_config.older_than_days * 86400) as u64
                                    && std::fs::remove_file(entry.path()).is_ok()
                                {
                                    affected_items += 1;
                                }
                            }
                        }
                    }
                }
            }
            unknown => {
                tracing::warn!(
                    "[ADMIN] Unknown cleanup type '{}' ignored. Valid types: logs, exports",
                    unknown
                );
            }
        }
    }

    Ok(MaintenanceResult {
        success: true,
        operation: "cleanup_data".to_string(),
        message: format!("Cleanup completed. Affected {} items.", affected_items),
        affected_items,
        execution_time_ms: start_time.elapsed().as_millis() as u64,
    })
}
