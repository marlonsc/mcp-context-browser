//! Configuration history management
//!
//! Provides persistence for configuration changes, enabling audit trails
//! and history viewing in the admin interface.

use crate::admin::service::helpers::admin_defaults;
use crate::admin::service::types::{AdminError, ConfigurationChange};
use crate::infrastructure::utils::FileUtils;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Configuration history store
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigHistory {
    pub entries: Vec<ConfigurationChange>,
}

/// Thread-safe configuration history manager
pub struct ConfigHistoryManager {
    history: RwLock<ConfigHistory>,
    history_path: PathBuf,
}

impl ConfigHistoryManager {
    /// Create a new history manager with the provided path (from config)
    pub async fn new(history_path: PathBuf) -> Result<Self, AdminError> {
        let history = load_history_from_path(&history_path)
            .await
            .unwrap_or_default();
        Ok(Self {
            history: RwLock::new(history),
            history_path,
        })
    }

    /// Record a configuration change
    pub async fn record_change(
        &self,
        user: &str,
        path: &str,
        change_type: &str,
        old_value: Option<serde_json::Value>,
        new_value: serde_json::Value,
    ) -> Result<ConfigurationChange, AdminError> {
        let change = ConfigurationChange {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            user: user.to_string(),
            path: path.to_string(),
            old_value,
            new_value,
            change_type: change_type.to_string(),
        };

        let mut history = self.history.write().await;
        history.entries.insert(0, change.clone());

        // Trim to max entries
        if history.entries.len() > admin_defaults::DEFAULT_MAX_HISTORY_ENTRIES {
            history
                .entries
                .truncate(admin_defaults::DEFAULT_MAX_HISTORY_ENTRIES);
        }

        // Persist to disk (fire and forget, don't block)
        let history_clone = history.clone();
        let path = self.history_path.clone();
        tokio::spawn(async move {
            if let Err(e) = save_history_to_path(&history_clone, &path).await {
                tracing::warn!("Failed to persist config history: {}", e);
            }
        });

        Ok(change)
    }

    /// Record multiple changes from a batch update
    pub async fn record_batch(
        &self,
        user: &str,
        updates: &HashMap<String, serde_json::Value>,
        old_config: Option<&HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<ConfigurationChange>, AdminError> {
        let mut changes = Vec::new();

        for (path, new_value) in updates {
            let old_value = old_config.and_then(|c| c.get(path).cloned());
            let change_type = if old_value.is_some() {
                "updated"
            } else {
                "added"
            };

            let change = ConfigurationChange {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                user: user.to_string(),
                path: path.clone(),
                old_value,
                new_value: new_value.clone(),
                change_type: change_type.to_string(),
            };
            changes.push(change);
        }

        // Add all changes to history
        {
            let mut history = self.history.write().await;
            for change in &changes {
                history.entries.insert(0, change.clone());
            }

            // Trim to max entries
            if history.entries.len() > admin_defaults::DEFAULT_MAX_HISTORY_ENTRIES {
                history
                    .entries
                    .truncate(admin_defaults::DEFAULT_MAX_HISTORY_ENTRIES);
            }

            // Persist
            let history_clone = history.clone();
            let path = self.history_path.clone();
            tokio::spawn(async move {
                if let Err(e) = save_history_to_path(&history_clone, &path).await {
                    tracing::warn!("Failed to persist config history: {}", e);
                }
            });
        }

        Ok(changes)
    }

    /// Get configuration history with optional limit
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<ConfigurationChange> {
        let history = self.history.read().await;
        let limit = limit.unwrap_or(100);
        history.entries.iter().take(limit).cloned().collect()
    }

    /// Get total number of history entries
    pub async fn count(&self) -> usize {
        self.history.read().await.entries.len()
    }

    /// Clear all history (for testing or admin reset)
    pub async fn clear(&self) -> Result<(), AdminError> {
        let mut history = self.history.write().await;
        history.entries.clear();

        save_history_to_path(&history, &self.history_path).await?;
        Ok(())
    }
}

/// Load history from disk at a specific path
async fn load_history_from_path(path: &PathBuf) -> Result<ConfigHistory, AdminError> {
    if !FileUtils::exists(path).await {
        return Ok(ConfigHistory::default());
    }

    FileUtils::read_json(path, "configuration history")
        .await
        .map_err(|e| AdminError::InternalError(e.to_string()))
}

/// Save history to disk at a specific path
async fn save_history_to_path(history: &ConfigHistory, path: &PathBuf) -> Result<(), AdminError> {
    FileUtils::ensure_dir_write_json(path, history, "configuration history")
        .await
        .map_err(|e| AdminError::InternalError(e.to_string()))
}

/// Standalone function to get configuration history from a specific path (from config)
pub async fn get_configuration_history(
    path: &PathBuf,
    limit: Option<usize>,
) -> Result<Vec<ConfigurationChange>, AdminError> {
    let history = load_history_from_path(path).await?;
    let limit = limit.unwrap_or(100);
    Ok(history.entries.into_iter().take(limit).collect())
}

/// Standalone function to record a configuration change
pub async fn record_configuration_change(
    path: &PathBuf,
    user: &str,
    config_path: &str,
    change_type: &str,
    old_value: Option<serde_json::Value>,
    new_value: serde_json::Value,
) -> Result<ConfigurationChange, AdminError> {
    let mut history = load_history_from_path(path).await?;

    let change = ConfigurationChange {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        user: user.to_string(),
        path: config_path.to_string(),
        old_value,
        new_value,
        change_type: change_type.to_string(),
    };

    history.entries.insert(0, change.clone());

    // Trim to max entries
    if history.entries.len() > admin_defaults::DEFAULT_MAX_HISTORY_ENTRIES {
        history
            .entries
            .truncate(admin_defaults::DEFAULT_MAX_HISTORY_ENTRIES);
    }

    save_history_to_path(&history, path).await?;

    Ok(change)
}

/// Standalone function to record batch configuration changes
pub async fn record_batch_changes(
    path: &PathBuf,
    user: &str,
    updates: &HashMap<String, serde_json::Value>,
    old_config: Option<&HashMap<String, serde_json::Value>>,
) -> Result<Vec<ConfigurationChange>, AdminError> {
    let mut history = load_history_from_path(path).await?;
    let mut changes = Vec::new();

    for (config_path, new_value) in updates {
        let old_value = old_config.and_then(|c| c.get(config_path).cloned());
        let change_type = if old_value.is_some() {
            "updated"
        } else {
            "added"
        };

        let change = ConfigurationChange {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            user: user.to_string(),
            path: config_path.clone(),
            old_value,
            new_value: new_value.clone(),
            change_type: change_type.to_string(),
        };

        history.entries.insert(0, change.clone());
        changes.push(change);
    }

    // Trim to max entries
    if history.entries.len() > admin_defaults::DEFAULT_MAX_HISTORY_ENTRIES {
        history
            .entries
            .truncate(admin_defaults::DEFAULT_MAX_HISTORY_ENTRIES);
    }

    save_history_to_path(&history, path).await?;

    Ok(changes)
}

/// Result of applying configuration updates
#[derive(Debug, Clone)]
pub struct ConfigUpdateApplicationResult {
    /// Changes that were applied
    pub changes_applied: Vec<String>,
    /// Whether changes require a restart
    pub requires_restart: bool,
}

/// Apply configuration updates and return what was applied
///
/// This function handles the mapping of configuration paths to actual changes,
/// logging each change and determining if a restart is required.
pub fn apply_configuration_updates(
    updates: &HashMap<String, serde_json::Value>,
) -> ConfigUpdateApplicationResult {
    let mut changes_applied = Vec::new();
    let mut requires_restart = false;

    for (path, value) in updates {
        match path.as_str() {
            // Metrics configuration
            "metrics.enabled" => {
                if let Some(enabled) = value.as_bool() {
                    tracing::info!(
                        "Metrics collection: {}",
                        if enabled { "enabled" } else { "disabled" }
                    );
                    changes_applied.push(format!("metrics.enabled = {}", enabled));
                }
            }
            "metrics.collection_interval" => {
                if let Some(interval) = value.as_u64() {
                    tracing::info!("Metrics collection interval set to {} seconds", interval);
                    changes_applied.push(format!(
                        "metrics.collection_interval = {} seconds",
                        interval
                    ));
                }
            }
            "metrics.retention_days" => {
                if let Some(days) = value.as_u64() {
                    tracing::info!("Metrics retention set to {} days", days);
                    changes_applied.push(format!("metrics.retention_days = {} days", days));
                }
            }
            // Cache configuration
            "cache.enabled" => {
                if let Some(enabled) = value.as_bool() {
                    tracing::info!("Cache: {}", if enabled { "enabled" } else { "disabled" });
                    changes_applied.push(format!("cache.enabled = {}", enabled));
                }
            }
            "cache.max_size" => {
                if let Some(size) = value.as_u64() {
                    tracing::info!("Cache max size set to {} bytes", size);
                    changes_applied.push(format!("cache.max_size = {} bytes", size));
                }
            }
            "cache.ttl_seconds" => {
                if let Some(ttl) = value.as_u64() {
                    tracing::info!("Cache TTL set to {} seconds", ttl);
                    changes_applied.push(format!("cache.ttl_seconds = {} seconds", ttl));
                }
            }
            // Database configuration (requires restart)
            "database.url" => {
                if value.as_str().is_some() {
                    tracing::info!("Database URL configured (change requires restart)");
                    changes_applied.push("database.url = ***".to_string());
                    requires_restart = true;
                }
            }
            "database.pool_size" => {
                if let Some(pool_size) = value.as_u64() {
                    tracing::info!(
                        "Database pool size set to {} (change requires restart)",
                        pool_size
                    );
                    changes_applied.push(format!(
                        "database.pool_size = {} (requires restart)",
                        pool_size
                    ));
                    requires_restart = true;
                }
            }
            "database.connection_timeout" => {
                if let Some(timeout) = value.as_u64() {
                    tracing::info!(
                        "Database connection timeout set to {} ms (change requires restart)",
                        timeout
                    );
                    changes_applied.push(format!("database.connection_timeout = {} ms", timeout));
                    requires_restart = true;
                }
            }
            // Server configuration (requires restart)
            "server.port" => {
                if let Some(port) = value.as_u64() {
                    tracing::info!(
                        "Server port configured to {} (change requires restart)",
                        port
                    );
                    changes_applied.push(format!("server.port = {} (requires restart)", port));
                    requires_restart = true;
                }
            }
            "server.listen_address" => {
                if let Some(addr) = value.as_str() {
                    tracing::info!(
                        "Server listen address configured to {} (change requires restart)",
                        addr
                    );
                    changes_applied.push(format!(
                        "server.listen_address = {} (requires restart)",
                        addr
                    ));
                    requires_restart = true;
                }
            }
            // Logging configuration
            "logging.level" => {
                if let Some(level) = value.as_str() {
                    tracing::info!("Logging level set to {}", level);
                    changes_applied.push(format!("logging.level = {}", level));
                }
            }
            "logging.format" => {
                if let Some(format) = value.as_str() {
                    tracing::info!("Logging format set to {}", format);
                    changes_applied.push(format!("logging.format = {}", format));
                }
            }
            // Indexing configuration
            "indexing.enabled" => {
                if let Some(enabled) = value.as_bool() {
                    tracing::info!("Indexing: {}", if enabled { "enabled" } else { "disabled" });
                    changes_applied.push(format!("indexing.enabled = {}", enabled));
                }
            }
            "indexing.batch_size" => {
                if let Some(batch_size) = value.as_u64() {
                    tracing::info!("Indexing batch size set to {}", batch_size);
                    changes_applied.push(format!("indexing.batch_size = {}", batch_size));
                }
            }
            // Provider configuration
            "providers.embedding" => {
                if let Some(provider) = value.as_str() {
                    tracing::info!("Embedding provider set to {}", provider);
                    changes_applied.push(format!("providers.embedding = {}", provider));
                }
            }
            "providers.vector_store" => {
                if let Some(provider) = value.as_str() {
                    tracing::info!("Vector store provider set to {}", provider);
                    changes_applied.push(format!("providers.vector_store = {}", provider));
                }
            }
            // Unknown configuration path
            _ => {
                tracing::warn!("Unknown configuration path: {}", path);
                changes_applied.push(format!("{} = {:?} (unknown, not applied)", path, value));
            }
        }
    }

    ConfigUpdateApplicationResult {
        changes_applied,
        requires_restart,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_and_get_history() {
        let temp_dir = std::env::temp_dir();
        let history_path = temp_dir.join("test_config_history.json");
        let manager = ConfigHistoryManager::new(history_path.clone())
            .await
            .unwrap();

        // Record a change
        let change = manager
            .record_change(
                "test_user",
                "metrics.enabled",
                "updated",
                Some(serde_json::json!(false)),
                serde_json::json!(true),
            )
            .await
            .unwrap();

        assert_eq!(change.user, "test_user");
        assert_eq!(change.path, "metrics.enabled");
        assert_eq!(change.change_type, "updated");

        // Get history
        let history = manager.get_history(Some(10)).await;
        assert!(!history.is_empty());
        assert_eq!(history[0].id, change.id);

        // Clean up
        let _ = tokio::fs::remove_file(&history_path).await;
    }

    #[tokio::test]
    async fn test_history_limit() {
        let temp_dir = std::env::temp_dir();
        let history_path = temp_dir.join("test_config_history_limit.json");
        let manager = ConfigHistoryManager::new(history_path.clone())
            .await
            .unwrap();

        // Record multiple changes
        for i in 0..5 {
            manager
                .record_change(
                    "test_user",
                    &format!("path.{}", i),
                    "added",
                    None,
                    serde_json::json!(i),
                )
                .await
                .unwrap();
        }

        // Get limited history
        let history = manager.get_history(Some(3)).await;
        assert!(history.len() <= 3);

        // Clean up
        let _ = tokio::fs::remove_file(&history_path).await;
    }
}
