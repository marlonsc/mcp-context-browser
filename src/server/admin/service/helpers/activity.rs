//! Activity feed tracking from EventBus
//!
//! Converts SystemEvents into human-readable activity log entries
//! for display in the admin dashboard.

use crate::infrastructure::events::SystemEvent;
use crate::server::admin::service::helpers::admin_defaults;
use crate::server::admin::service::types::AdminError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Activity level/severity
/// Activity severity levels for logging and display
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ActivityLevel {
    /// Informational activity
    Info,
    /// Warning level activity
    Warning,
    /// Error level activity
    Error,
    /// Successful operation activity
    Success,
}

/// A single activity entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    /// Unique activity ID
    pub id: String,
    /// When the activity occurred
    pub timestamp: DateTime<Utc>,
    /// Activity level/severity
    pub level: ActivityLevel,
    /// Category of the activity (cache, provider, index, etc.)
    pub category: String,
    /// Human-readable activity message
    pub message: String,
    /// Optional details as JSON
    pub details: Option<serde_json::Value>,
}

/// Thread-safe activity logger that tracks system events
pub struct ActivityLogger {
    activities: Arc<RwLock<Vec<Activity>>>,
}

impl ActivityLogger {
    /// Create a new activity logger
    pub fn new() -> Self {
        Self {
            activities: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start listening to events from the event bus
    pub fn start_listening(
        &self,
        event_bus: crate::infrastructure::events::SharedEventBusProvider,
    ) {
        let activities = Arc::clone(&self.activities);

        tokio::spawn(async move {
            let mut receiver = match event_bus.subscribe().await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Failed to subscribe to event bus: {}", e);
                    return;
                }
            };
            loop {
                match receiver.recv().await {
                    Ok(event) => {
                        if let Some(activity) = event_to_activity(&event) {
                            let mut activities = activities.write().await;
                            activities.insert(0, activity);
                            if activities.len() > admin_defaults::DEFAULT_MAX_ACTIVITIES {
                                activities.truncate(admin_defaults::DEFAULT_MAX_ACTIVITIES);
                            }
                        }
                    }
                    Err(_e) => {
                        tracing::info!("Activity logger: event bus closed or error occurred");
                        break;
                    }
                }
            }
        });
    }

    /// Get recent activities with optional limit
    pub async fn get_activities(&self, limit: Option<usize>) -> Vec<Activity> {
        let activities = self.activities.read().await;
        let limit = limit.unwrap_or(admin_defaults::DEFAULT_MAX_ACTIVITIES);
        activities.iter().take(limit).cloned().collect()
    }

    /// Get activities count
    pub async fn count(&self) -> usize {
        self.activities.read().await.len()
    }

    /// Manually add an activity (for non-event activities)
    pub async fn add_activity(
        &self,
        level: ActivityLevel,
        category: &str,
        message: &str,
        details: Option<serde_json::Value>,
    ) {
        let activity = Activity {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level,
            category: category.to_string(),
            message: message.to_string(),
            details,
        };

        let mut activities = self.activities.write().await;
        activities.insert(0, activity);
        if activities.len() > admin_defaults::DEFAULT_MAX_ACTIVITIES {
            activities.truncate(admin_defaults::DEFAULT_MAX_ACTIVITIES);
        }
    }
}

impl Default for ActivityLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a SystemEvent to an Activity entry
fn event_to_activity(event: &SystemEvent) -> Option<Activity> {
    let (level, category, message, details): (
        ActivityLevel,
        &str,
        String,
        Option<serde_json::Value>,
    ) = match event {
        SystemEvent::CacheClear { namespace } => {
            let msg = match namespace {
                Some(ns) => format!("Cache cleared: {}", ns),
                None => "All caches cleared".to_string(),
            };
            (ActivityLevel::Success, "cache", msg, None)
        }
        SystemEvent::BackupCreate { path } => (
            ActivityLevel::Info,
            "backup",
            format!("Backup created: {}", path),
            None,
        ),
        SystemEvent::BackupRestore { path } => (
            ActivityLevel::Warning,
            "backup",
            format!("Backup restore started: {}", path),
            None,
        ),
        SystemEvent::IndexRebuild { collection } => {
            let msg = match collection {
                Some(c) => format!("Index rebuild started: {}", c),
                None => "Full index rebuild started".to_string(),
            };
            (ActivityLevel::Info, "index", msg, None)
        }
        SystemEvent::IndexClear { collection } => {
            let msg = match collection {
                Some(c) => format!("Index cleared: {}", c),
                None => "All indexes cleared".to_string(),
            };
            (ActivityLevel::Warning, "index", msg, None)
        }
        SystemEvent::IndexOptimize { collection } => {
            let msg = match collection {
                Some(c) => format!("Index optimization started: {}", c),
                None => "Index optimization started".to_string(),
            };
            (ActivityLevel::Info, "index", msg, None)
        }
        SystemEvent::ConfigReloaded => (
            ActivityLevel::Success,
            "config",
            "Configuration reloaded".to_string(),
            None,
        ),
        SystemEvent::ConfigurationChanged {
            user,
            changes,
            requires_restart,
            timestamp: _,
        } => (
            ActivityLevel::Warning,
            "config",
            format!(
                "Configuration changed by {}: {} changes{}",
                user,
                changes.len(),
                if *requires_restart {
                    " (restart required)"
                } else {
                    ""
                }
            ),
            Some(serde_json::json!({
                "user": user,
                "change_count": changes.len(),
                "requires_restart": requires_restart,
                "changes": changes
            })),
        ),
        SystemEvent::Shutdown => (
            ActivityLevel::Warning,
            "system",
            "Server shutdown initiated".to_string(),
            None,
        ),
        SystemEvent::Reload => (
            ActivityLevel::Info,
            "system",
            "Server reload requested".to_string(),
            None,
        ),
        SystemEvent::Respawn => (
            ActivityLevel::Warning,
            "system",
            "Server respawn requested".to_string(),
            None,
        ),
        SystemEvent::BinaryUpdated { path } => (
            ActivityLevel::Info,
            "system",
            format!("Binary updated: {}", path),
            None,
        ),
        SystemEvent::SyncCompleted {
            path,
            files_changed,
        } => (
            ActivityLevel::Success,
            "sync",
            format!("Sync completed: {} ({} files changed)", path, files_changed),
            None,
        ),
        SystemEvent::ProviderRestart {
            provider_type,
            provider_id,
        } => (
            ActivityLevel::Warning,
            "provider",
            format!(
                "Provider restart requested: {}:{}",
                provider_type, provider_id
            ),
            Some(serde_json::json!({
                "provider_type": provider_type,
                "provider_id": provider_id
            })),
        ),
        SystemEvent::ProviderReconfigure {
            provider_type,
            config: _,
        } => (
            ActivityLevel::Info,
            "provider",
            format!("Provider reconfiguration: {}", provider_type),
            None,
        ),
        SystemEvent::SubsystemHealthCheck { subsystem_id } => (
            ActivityLevel::Info,
            "health",
            format!("Health check requested: {}", subsystem_id),
            None,
        ),
        SystemEvent::RouterReload => (
            ActivityLevel::Info,
            "router",
            "Router reload requested".to_string(),
            None,
        ),
        SystemEvent::ProviderRestarted {
            provider_type,
            provider_id,
        } => (
            ActivityLevel::Success,
            "provider",
            format!(
                "Provider restarted successfully: {}:{}",
                provider_type, provider_id
            ),
            Some(serde_json::json!({
                "provider_type": provider_type,
                "provider_id": provider_id
            })),
        ),
        SystemEvent::RecoveryStarted {
            subsystem_id,
            retry_attempt,
        } => (
            ActivityLevel::Warning,
            "recovery",
            format!(
                "Recovery started for {} (attempt #{})",
                subsystem_id, retry_attempt
            ),
            Some(serde_json::json!({
                "subsystem_id": subsystem_id,
                "retry_attempt": retry_attempt
            })),
        ),
        SystemEvent::RecoveryCompleted {
            subsystem_id,
            success,
            message,
        } => {
            let level = if *success {
                ActivityLevel::Success
            } else {
                ActivityLevel::Error
            };
            (
                level,
                "recovery",
                format!("Recovery completed for {}: {}", subsystem_id, message),
                Some(serde_json::json!({
                    "subsystem_id": subsystem_id,
                    "success": success
                })),
            )
        }
        SystemEvent::RecoveryExhausted {
            subsystem_id,
            total_retries,
            last_error,
        } => (
            ActivityLevel::Error,
            "recovery",
            format!(
                "Recovery exhausted for {} after {} retries: {}",
                subsystem_id,
                total_retries,
                last_error.as_deref().unwrap_or("Unknown error")
            ),
            Some(serde_json::json!({
                "subsystem_id": subsystem_id,
                "total_retries": total_retries,
                "last_error": last_error
            })),
        ),
    };

    Some(Activity {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        level,
        category: category.to_string(),
        message,
        details,
    })
}

/// Standalone function to get recent activities (for use by AdminService)
pub async fn get_recent_activities(
    activity_logger: &ActivityLogger,
    limit: Option<usize>,
) -> Result<Vec<Activity>, AdminError> {
    Ok(activity_logger.get_activities(limit).await)
}
