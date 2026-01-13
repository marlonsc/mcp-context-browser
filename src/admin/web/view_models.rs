//! View models for web templates - bridge between service layer and presentation
//!
//! These DTOs are specifically designed for template rendering, with:
//! - Pre-computed CSS classes for UI styling
//! - Pre-formatted strings for display
//! - Flat structures optimized for Tera template access

use serde::Serialize;

use crate::infrastructure::utils::{css, FormattingUtils, StringUtils};

// =============================================================================
// Dashboard View Models
// =============================================================================

/// Complete dashboard view model - aggregates data from multiple service calls
#[derive(Debug, Clone, Serialize)]
pub struct DashboardViewModel {
    pub page: &'static str,
    pub metrics: MetricsViewModel,
    pub providers: ProvidersViewModel,
    pub indexes: IndexesSummaryViewModel,
    pub activities: Vec<ActivityViewModel>,
    pub system_health: HealthViewModel,
}

/// System performance metrics
#[derive(Debug, Clone, Serialize)]
pub struct MetricsViewModel {
    pub cpu_usage: f64,
    pub cpu_usage_formatted: String,
    pub memory_usage: f64,
    pub memory_usage_formatted: String,
    pub total_queries: u64,
    pub total_queries_formatted: String,
    pub avg_latency_ms: f64,
    pub avg_latency_formatted: String,
}

impl MetricsViewModel {
    pub fn new(cpu_usage: f64, memory_usage: f64, total_queries: u64, avg_latency_ms: f64) -> Self {
        Self {
            cpu_usage,
            cpu_usage_formatted: format!("{:.1}%", cpu_usage),
            memory_usage,
            memory_usage_formatted: format!("{:.1}%", memory_usage),
            total_queries,
            total_queries_formatted: FormattingUtils::format_number(total_queries),
            avg_latency_ms,
            avg_latency_formatted: format!("{:.1}ms", avg_latency_ms),
        }
    }
}

// =============================================================================
// Providers View Models
// =============================================================================

/// Provider list view model with summary counts
#[derive(Debug, Clone, Serialize)]
pub struct ProvidersViewModel {
    pub page: &'static str,
    pub active_count: usize,
    pub total_count: usize,
    pub providers: Vec<ProviderViewModel>,
}

impl ProvidersViewModel {
    pub fn new(providers: Vec<ProviderViewModel>) -> Self {
        let active_count = providers.iter().filter(|p| p.is_active).count();
        let total_count = providers.len();
        Self {
            page: "providers",
            active_count,
            total_count,
            providers,
        }
    }
}

/// Individual provider view model
#[derive(Debug, Clone, Serialize)]
pub struct ProviderViewModel {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub provider_type_display: String,
    pub status: String,
    pub status_display: String,
    pub status_class: &'static str,
    pub is_active: bool,
}

impl ProviderViewModel {
    pub fn new(id: String, name: String, provider_type: String, status: String) -> Self {
        let is_active = matches!(status.as_str(), "available" | "active" | "healthy");
        let status_class = css::badge_for_status(&status);
        let provider_type_display = StringUtils::to_title_case(&provider_type);
        let status_display = StringUtils::capitalize_first(&status);

        Self {
            id,
            name,
            provider_type,
            provider_type_display,
            status,
            status_display,
            status_class,
            is_active,
        }
    }
}

// =============================================================================
// Indexes View Models
// =============================================================================

/// Index list view model for indexes page
#[derive(Debug, Clone, Serialize)]
pub struct IndexesViewModel {
    pub page: &'static str,
    pub indexes: Vec<IndexViewModel>,
    pub total_documents: u64,
    pub total_documents_formatted: String,
    pub active_count: usize,
}

impl IndexesViewModel {
    pub fn new(indexes: Vec<IndexViewModel>, total_documents: u64) -> Self {
        let active_count = indexes.iter().filter(|i| i.is_active).count();
        Self {
            page: "indexes",
            indexes,
            total_documents,
            total_documents_formatted: FormattingUtils::format_number(total_documents),
            active_count,
        }
    }
}

/// Summary view model for dashboard
#[derive(Debug, Clone, Serialize)]
pub struct IndexesSummaryViewModel {
    pub active_count: usize,
    pub total_documents: u64,
    pub total_documents_formatted: String,
    pub is_indexing: bool,
}

/// Individual index view model
#[derive(Debug, Clone, Serialize)]
pub struct IndexViewModel {
    pub id: String,
    pub name: String,
    pub status: String,
    pub status_display: String,
    pub status_class: &'static str,
    pub is_active: bool,
    pub is_indexing: bool,
    pub document_count: u64,
    pub document_count_formatted: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub age_display: String,
}

impl IndexViewModel {
    pub fn new(
        id: String,
        name: String,
        status: String,
        document_count: u64,
        created_at: u64,
        updated_at: u64,
    ) -> Self {
        let is_indexing = status == "indexing";
        let is_active = matches!(status.as_str(), "active" | "ready");
        let status_class = css::badge_for_status(&status);
        let age_display = FormattingUtils::format_age(created_at);

        Self {
            id,
            name,
            status_display: StringUtils::capitalize_first(&status),
            status,
            status_class,
            is_active,
            is_indexing,
            document_count,
            document_count_formatted: FormattingUtils::format_number(document_count),
            created_at,
            updated_at,
            age_display,
        }
    }
}

// =============================================================================
// Activity View Models
// =============================================================================

/// Activity item view model for activity feed
#[derive(Debug, Clone, Serialize)]
pub struct ActivityViewModel {
    pub id: String,
    pub message: String,
    pub timestamp: String,
    pub timestamp_relative: String,
    pub level: String,
    pub level_class: &'static str,
    pub indicator_class: &'static str,
    pub category: String,
}

impl ActivityViewModel {
    pub fn new(
        id: String,
        message: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        level: &str,
        category: String,
    ) -> Self {
        let level_class = css::badge_for_level(level);
        let indicator_class = css::indicator_for_level(level);
        let timestamp_str = timestamp.format("%H:%M:%S").to_string();
        let timestamp_relative = StringUtils::format_relative_time(timestamp);

        Self {
            id,
            message,
            timestamp: timestamp_str,
            timestamp_relative,
            level: level.to_string(),
            level_class,
            indicator_class,
            category,
        }
    }
}

// =============================================================================
// Health View Models
// =============================================================================

/// System health view model
#[derive(Debug, Clone, Serialize)]
pub struct HealthViewModel {
    pub status: String,
    pub status_display: String,
    pub status_class: &'static str,
    pub indicator_class: &'static str,
    pub uptime_seconds: u64,
    pub uptime_formatted: String,
    pub pid: u32,
}

impl HealthViewModel {
    pub fn new(status: &str, uptime_seconds: u64, pid: u32) -> Self {
        let status_class = css::badge_for_status(status);
        let indicator_class = css::indicator_for_status(status);

        Self {
            status: status.to_string(),
            status_display: StringUtils::capitalize_first(status),
            status_class,
            indicator_class,
            uptime_seconds,
            uptime_formatted: FormattingUtils::format_duration(uptime_seconds),
            pid,
        }
    }
}

// =============================================================================
// Configuration View Models
// =============================================================================

/// Configuration page view model
#[derive(Debug, Clone, Serialize)]
pub struct ConfigurationViewModel {
    pub page: &'static str,
    pub page_description: &'static str,
    pub categories: Vec<ConfigCategoryViewModel>,
}

/// Configuration category view model
#[derive(Debug, Clone, Serialize)]
pub struct ConfigCategoryViewModel {
    pub name: String,
    pub description: String,
    pub settings: Vec<ConfigSettingViewModel>,
}

/// Individual configuration setting view model
#[derive(Debug, Clone, Serialize)]
pub struct ConfigSettingViewModel {
    pub key: String,
    pub label: String,
    pub value: serde_json::Value,
    pub value_display: String,
    pub setting_type: &'static str,
    pub description: String,
    pub editable: bool,
}

// =============================================================================
// Logs View Models
// =============================================================================

/// Logs page view model
#[derive(Debug, Clone, Serialize)]
pub struct LogsViewModel {
    pub page: &'static str,
    pub page_description: &'static str,
    pub entries: Vec<LogEntryViewModel>,
    pub total_count: u64,
    pub stats: LogStatsViewModel,
}

/// Log entry view model
#[derive(Debug, Clone, Serialize)]
pub struct LogEntryViewModel {
    pub timestamp: String,
    pub level: String,
    pub level_class: &'static str,
    pub message: String,
    pub source: String,
}

/// Log statistics view model
#[derive(Debug, Clone, Serialize)]
pub struct LogStatsViewModel {
    pub total: u64,
    pub errors: u64,
    pub warnings: u64,
    pub info: u64,
}

// =============================================================================
// Error View Model
// =============================================================================

/// Error page view model
#[derive(Debug, Clone, Serialize)]
pub struct ErrorViewModel {
    pub title: String,
    pub message: String,
    pub details: Option<String>,
    pub back_url: &'static str,
}

impl ErrorViewModel {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            details: None,
            back_url: "/dashboard",
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_view_model() {
        let vm = ProviderViewModel::new(
            "openai-1".to_string(),
            "OpenAI".to_string(),
            "embedding".to_string(),
            "available".to_string(),
        );
        assert!(vm.is_active);
        assert_eq!(vm.provider_type_display, "Embedding");
        assert!(vm.status_class.contains("green"));
    }

    #[test]
    fn test_health_view_model() {
        let vm = HealthViewModel::new("healthy", 3661, 12345);
        assert_eq!(vm.uptime_formatted, "1h 1m 1s");
        assert!(vm.status_class.contains("green"));
    }
}
