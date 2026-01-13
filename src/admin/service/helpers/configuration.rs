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

/// Helper for processing configuration changes with consistent patterns.
/// Reduces boilerplate in apply_configuration_updates from ~150 lines to ~50 lines.
struct ConfigChangeHandler {
    changes_applied: Vec<String>,
    requires_restart: bool,
}

impl ConfigChangeHandler {
    fn new() -> Self {
        Self {
            changes_applied: Vec::new(),
            requires_restart: false,
        }
    }

    /// Handle boolean configuration change
    fn handle_bool(&mut self, path: &str, value: &serde_json::Value, label: &str) {
        if let Some(enabled) = value.as_bool() {
            tracing::info!("{}: {}", label, if enabled { "enabled" } else { "disabled" });
            self.changes_applied.push(format!("{} = {}", path, enabled));
        }
    }

    /// Handle u64 configuration change
    fn handle_u64(&mut self, path: &str, value: &serde_json::Value, label: &str, unit: &str) {
        if let Some(val) = value.as_u64() {
            let unit_suffix = if unit.is_empty() { String::new() } else { format!(" {}", unit) };
            tracing::info!("{} set to {}{}", label, val, unit_suffix);
            self.changes_applied.push(format!("{} = {}{}", path, val, unit_suffix));
        }
    }

    /// Handle u64 configuration change requiring restart
    fn handle_u64_restart(&mut self, path: &str, value: &serde_json::Value, label: &str, unit: &str) {
        if let Some(val) = value.as_u64() {
            let unit_suffix = if unit.is_empty() { String::new() } else { format!(" {}", unit) };
            tracing::info!("{} set to {}{} (change requires restart)", label, val, unit_suffix);
            self.changes_applied.push(format!("{} = {}{} (requires restart)", path, val, unit_suffix));
            self.requires_restart = true;
        }
    }

    /// Handle string configuration change
    fn handle_string(&mut self, path: &str, value: &serde_json::Value, label: &str) {
        if let Some(val) = value.as_str() {
            tracing::info!("{} set to {}", label, val);
            self.changes_applied.push(format!("{} = {}", path, val));
        }
    }

    /// Handle string configuration change requiring restart
    fn handle_string_restart(&mut self, path: &str, value: &serde_json::Value, label: &str) {
        if let Some(val) = value.as_str() {
            tracing::info!("{} configured to {} (change requires restart)", label, val);
            self.changes_applied.push(format!("{} = {} (requires restart)", path, val));
            self.requires_restart = true;
        }
    }

    /// Handle secret/sensitive configuration change (always requires restart, masked value)
    fn handle_secret(&mut self, path: &str, value: &serde_json::Value, label: &str) {
        if value.as_str().is_some() {
            tracing::info!("{} configured (change requires restart)", label);
            self.changes_applied.push(format!("{} = ***", path));
            self.requires_restart = true;
        }
    }

    /// Handle unknown configuration path
    fn handle_unknown(&mut self, path: &str, value: &serde_json::Value) {
        tracing::warn!("Unknown configuration path: {}", path);
        self.changes_applied.push(format!("{} = {:?} (unknown, not applied)", path, value));
    }

    /// Get final results
    fn into_result(self) -> ConfigUpdateApplicationResult {
        ConfigUpdateApplicationResult {
            changes_applied: self.changes_applied,
            requires_restart: self.requires_restart,
        }
    }
}

/// Apply configuration updates and return what was applied
///
/// This function handles the mapping of configuration paths to actual changes,
/// logging each change and determining if a restart is required.
pub fn apply_configuration_updates(
    updates: &HashMap<String, serde_json::Value>,
) -> ConfigUpdateApplicationResult {
    let mut handler = ConfigChangeHandler::new();

    for (path, value) in updates {
        match path.as_str() {
            // Metrics configuration
            "metrics.enabled" => handler.handle_bool(path, value, "Metrics collection"),
            "metrics.collection_interval" => handler.handle_u64(path, value, "Metrics collection interval", "seconds"),
            "metrics.retention_days" => handler.handle_u64(path, value, "Metrics retention", "days"),

            // Cache configuration
            "cache.enabled" => handler.handle_bool(path, value, "Cache"),
            "cache.max_size" => handler.handle_u64(path, value, "Cache max size", "bytes"),
            "cache.ttl_seconds" => handler.handle_u64(path, value, "Cache TTL", "seconds"),

            // Database configuration (requires restart)
            "database.url" => handler.handle_secret(path, value, "Database URL"),
            "database.pool_size" => handler.handle_u64_restart(path, value, "Database pool size", ""),
            "database.connection_timeout" => handler.handle_u64_restart(path, value, "Database connection timeout", "ms"),

            // Server configuration (requires restart)
            "server.port" => handler.handle_u64_restart(path, value, "Server port", ""),
            "server.listen_address" => handler.handle_string_restart(path, value, "Server listen address"),

            // Logging configuration
            "logging.level" => handler.handle_string(path, value, "Logging level"),
            "logging.format" => handler.handle_string(path, value, "Logging format"),

            // Indexing configuration
            "indexing.enabled" => handler.handle_bool(path, value, "Indexing"),
            "indexing.batch_size" => handler.handle_u64(path, value, "Indexing batch size", ""),

            // Provider configuration
            "providers.embedding" => handler.handle_string(path, value, "Embedding provider"),
            "providers.vector_store" => handler.handle_string(path, value, "Vector store provider"),

            // Unknown configuration path
            _ => handler.handle_unknown(path, value),
        }
    }

    handler.into_result()
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
