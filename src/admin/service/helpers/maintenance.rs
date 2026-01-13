//! Maintenance operations helper module
//!
//! Provides functions for cache management, provider restart, index rebuilding, and data cleanup.

use crate::admin::service::helpers::admin_defaults;
use crate::admin::service::types::{AdminError, CacheType, CleanupConfig, MaintenanceResult};
use crate::infrastructure::events::SharedEventBusProvider;
use crate::infrastructure::logging::SharedLogBuffer;
use crate::infrastructure::utils::ProviderUtils;

/// Clear cache by type
pub async fn clear_cache(
    event_bus: &SharedEventBusProvider,
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
        .await
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
pub async fn restart_provider(
    event_bus: &SharedEventBusProvider,
    provider_id: &str,
) -> Result<MaintenanceResult, AdminError> {
    let start_time = std::time::Instant::now();

    // Parse provider_id to extract type and id
    let (provider_type, actual_id) = ProviderUtils::parse_provider_id(provider_id);

    tracing::info!(
        "[ADMIN] Requesting restart for provider: {} (type: {}, id: {})",
        provider_id,
        provider_type,
        actual_id
    );

    // Publish restart event
    event_bus
        .publish(
            crate::infrastructure::events::SystemEvent::ProviderRestart {
                provider_type: provider_type.clone(),
                provider_id: actual_id.clone(),
            },
        )
        .await
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

/// Reconfigure a provider without a full restart
///
/// This allows hot-updating provider configuration (e.g., API keys, endpoints)
/// without requiring a full provider restart or server restart.
///
/// The provider_id should be in the format "type:id" (e.g., "embedding:ollama").
pub async fn reconfigure_provider(
    event_bus: &SharedEventBusProvider,
    provider_id: &str,
    config: serde_json::Value,
) -> Result<MaintenanceResult, AdminError> {
    let start_time = std::time::Instant::now();

    // Parse provider_id to extract type
    let (provider_type, _) = ProviderUtils::parse_provider_id(provider_id);

    tracing::info!(
        "[ADMIN] Reconfiguring provider: {} (type: {}) with new config",
        provider_id,
        provider_type
    );

    // Publish reconfiguration event
    event_bus
        .publish(
            crate::infrastructure::events::SystemEvent::ProviderReconfigure {
                provider_type: provider_type.clone(),
                config: config.clone(),
            },
        )
        .await
        .map_err(|e| {
            AdminError::McpServerError(format!(
                "Failed to publish ProviderReconfigure event: {}",
                e
            ))
        })?;

    // Log the configuration change for audit trail
    tracing::info!(
        "[ADMIN] Provider '{}' reconfiguration event published successfully",
        provider_id
    );

    Ok(MaintenanceResult {
        success: true,
        operation: "reconfigure_provider".to_string(),
        message: format!(
            "Reconfiguration signal sent for provider '{}' (type: {}). \
             Configuration will be applied without requiring a restart.",
            provider_id, provider_type
        ),
        affected_items: 1,
        execution_time_ms: start_time.elapsed().as_millis() as u64,
    })
}

/// Restart all providers in the given list
///
/// Each provider is specified as a tuple of (provider_type, provider_id).
/// This function publishes a ProviderRestart event for each provider.
pub async fn restart_all_providers(
    event_bus: &SharedEventBusProvider,
    providers: &[(String, String)],
) -> Result<MaintenanceResult, AdminError> {
    let start_time = std::time::Instant::now();
    let mut successful = 0;
    let mut errors = Vec::new();

    for (provider_type, provider_id) in providers {
        let full_id = format!("{}:{}", provider_type, provider_id);
        match restart_provider(event_bus, &full_id).await {
            Ok(_) => successful += 1,
            Err(e) => errors.push(format!("{}: {}", full_id, e)),
        }
    }

    let message = if errors.is_empty() {
        format!(
            "Successfully requested restart for {} provider(s). \
             The RecoveryManager will handle restarts asynchronously.",
            successful
        )
    } else {
        format!(
            "Restarted {} provider(s) with {} error(s): {}",
            successful,
            errors.len(),
            errors.join(", ")
        )
    };

    Ok(MaintenanceResult {
        success: errors.is_empty(),
        operation: "restart_all_providers".to_string(),
        message,
        affected_items: successful as u64,
        execution_time_ms: start_time.elapsed().as_millis() as u64,
    })
}

/// Request index rebuild
pub async fn rebuild_index(
    event_bus: &SharedEventBusProvider,
    index_id: &str,
) -> Result<MaintenanceResult, AdminError> {
    let start_time = std::time::Instant::now();
    event_bus
        .publish(crate::infrastructure::events::SystemEvent::IndexRebuild {
            collection: Some(index_id.to_string()),
        })
        .await
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
                let export_dir = std::path::PathBuf::from(admin_defaults::DEFAULT_EXPORTS_DIR);
                if export_dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(export_dir) {
                        for entry in entries.flatten() {
                            if let Ok(metadata) = entry.metadata() {
                                let created =
                                    metadata.created().unwrap_or(std::time::SystemTime::now());
                                let age = std::time::SystemTime::now()
                                    .duration_since(created)
                                    .unwrap_or_default();
                                if age.as_secs()
                                    > (cleanup_config.older_than_days as u64
                                        * admin_defaults::SECONDS_PER_DAY)
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
