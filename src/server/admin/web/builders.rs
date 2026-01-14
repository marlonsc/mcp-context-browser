//! View model builders - compose AdminService data into view models
//!
//! This module provides the ViewModelBuilder service that bridges the gap between
//! the AdminService (application layer) and the web templates (presentation layer).
//!
//! ## Architecture
//!
//! ```text
//! Web Handler → ViewModelBuilder → AdminService
//!     ↓                ↓                ↓
//! Render HTML    Compose DTOs    Business Logic
//! ```
//!
//! The builder:
//! - Fetches data from AdminService using existing methods
//! - Transforms service types into view models with pre-computed presentation values
//! - Uses parallel fetching where possible for performance
//! - Handles errors gracefully with meaningful error messages

use anyhow::{Context, Result};

use super::view_model_builders::{
    ActivityLevelFormatter, ConfigCategoryBuilder, ConfigSettingBuilder, MetricsCollector,
};
use super::view_models::*;
use crate::infrastructure::utils::{css, FormattingUtils, HealthUtils, TimeUtils};
use crate::server::admin::models::AdminState;

/// Builds view models from AdminService data
///
/// This is the main service for constructing presentation-ready data structures
/// from the underlying admin service. Each build method fetches the necessary
/// data and transforms it into a view model suitable for Tera template rendering.
pub struct ViewModelBuilder<'a> {
    state: &'a AdminState,
}

impl<'a> ViewModelBuilder<'a> {
    /// Create a new ViewModelBuilder with the given AdminState
    pub fn new(state: &'a AdminState) -> Self {
        Self { state }
    }

    // =========================================================================
    // Dashboard Builders
    // =========================================================================

    /// Build complete dashboard view model
    ///
    /// Fetches metrics, providers, indexes, activities, and health data in parallel
    /// for optimal performance.
    pub async fn build_dashboard(&self) -> Result<DashboardViewModel> {
        // Parallel fetch for performance - all these are independent
        let (metrics, providers, indexes, activities, health) = tokio::try_join!(
            self.build_metrics(),
            self.build_providers_summary(),
            self.build_indexes_summary(),
            self.build_activities(10),
            self.build_health(),
        )?;

        Ok(DashboardViewModel {
            page: "dashboard",
            metrics,
            providers,
            indexes,
            activities,
            system_health: health,
        })
    }

    /// Build system metrics view model
    async fn build_metrics(&self) -> Result<MetricsViewModel> {
        let performance = self
            .state
            .admin_service
            .get_performance_metrics()
            .await
            .context("Failed to get performance metrics")?;

        let (cpu_usage, memory_usage) = MetricsCollector::new(self.state).collect_system().await?;

        Ok(MetricsViewModel::new(
            cpu_usage,
            memory_usage,
            performance.total_queries,
            performance.average_response_time_ms,
        ))
    }

    /// Build providers summary for dashboard
    async fn build_providers_summary(&self) -> Result<ProvidersViewModel> {
        let providers = self
            .state
            .admin_service
            .get_providers()
            .await
            .context("Failed to get providers")?;

        let provider_vms: Vec<ProviderViewModel> = providers
            .into_iter()
            .map(|p| ProviderViewModel::new(p.id, p.name, p.provider_type, p.status))
            .collect();

        Ok(ProvidersViewModel::new(provider_vms))
    }

    /// Build indexes summary for dashboard
    async fn build_indexes_summary(&self) -> Result<IndexesSummaryViewModel> {
        let status = self
            .state
            .admin_service
            .get_indexing_status()
            .await
            .context("Failed to get indexing status")?;

        Ok(IndexesSummaryViewModel {
            active_count: if status.is_indexing { 0 } else { 1 },
            total_documents: status.total_documents,
            total_documents_formatted: FormattingUtils::format_number(status.total_documents),
            is_indexing: status.is_indexing,
        })
    }

    /// Build activity list view model
    async fn build_activities(&self, limit: usize) -> Result<Vec<ActivityViewModel>> {
        let activities = self.state.activity_logger.get_activities(Some(limit)).await;

        Ok(activities
            .into_iter()
            .map(|a| {
                let level_str = ActivityLevelFormatter::to_css_class(a.level);
                ActivityViewModel::new(a.id, a.message, a.timestamp, level_str, a.category)
            })
            .collect())
    }

    /// Build system health view model
    async fn build_health(&self) -> Result<HealthViewModel> {
        let system_info = self
            .state
            .admin_service
            .get_system_info()
            .await
            .context("Failed to get system info")?;

        let (cpu_usage, memory_usage) = MetricsCollector::new(self.state).collect_system().await?;
        let status = HealthUtils::compute_status(cpu_usage, memory_usage);

        Ok(HealthViewModel::new(
            status,
            system_info.uptime,
            system_info.pid,
        ))
    }

    // =========================================================================
    // Providers Page Builders
    // =========================================================================

    /// Build providers page view model
    pub async fn build_providers_page(&self) -> Result<ProvidersViewModel> {
        self.build_providers_summary().await
    }

    // =========================================================================
    // Indexes Page Builders
    // =========================================================================

    /// Build indexes page view model
    pub async fn build_indexes_page(&self) -> Result<IndexesViewModel> {
        let status = self
            .state
            .admin_service
            .get_indexing_status()
            .await
            .context("Failed to get indexing status")?;

        let now = TimeUtils::now_unix_secs();

        let indexes = vec![IndexViewModel::new(
            "main-index".to_string(),
            "Main Codebase Index".to_string(),
            if status.is_indexing {
                "indexing"
            } else {
                "active"
            }
            .to_string(),
            status.indexed_documents,
            status.start_time.unwrap_or(0),
            now,
        )];

        Ok(IndexesViewModel::new(indexes, status.total_documents))
    }

    // =========================================================================
    // Configuration Page Builders
    // =========================================================================

    /// Build configuration page view model
    pub async fn build_configuration_page(&self) -> Result<ConfigurationViewModel> {
        let config = self
            .state
            .admin_service
            .get_configuration()
            .await
            .context("Failed to get configuration")?;

        let categories = vec![
            // Indexing settings
            ConfigCategoryBuilder::new(
                "Indexing",
                "Code indexing and chunking settings",
                vec![
                    ConfigSettingBuilder::number(
                        "indexing.chunk_size",
                        "Chunk Size",
                        config.indexing.chunk_size,
                        "Size of code chunks for embedding",
                    ),
                    ConfigSettingBuilder::number(
                        "indexing.chunk_overlap",
                        "Chunk Overlap",
                        config.indexing.chunk_overlap,
                        "Overlap between adjacent chunks",
                    ),
                    ConfigSettingBuilder::bytes(
                        "indexing.max_file_size",
                        "Max File Size",
                        config.indexing.max_file_size,
                        "Maximum file size to index",
                    ),
                ],
            ),
            // Security settings
            ConfigCategoryBuilder::new(
                "Security",
                "Authentication and rate limiting",
                vec![
                    ConfigSettingBuilder::boolean(
                        "security.enable_auth",
                        "Enable Authentication",
                        config.security.enable_auth,
                        "Require authentication for API access",
                    ),
                    ConfigSettingBuilder::boolean(
                        "security.rate_limiting",
                        "Rate Limiting",
                        config.security.rate_limiting,
                        "Enable request rate limiting",
                    ),
                    ConfigSettingBuilder::number(
                        "security.max_requests_per_minute",
                        "Max Requests/Minute",
                        config.security.max_requests_per_minute,
                        "Maximum requests per minute per client",
                    ),
                ],
            ),
            // Metrics settings
            ConfigCategoryBuilder::new(
                "Metrics",
                "Performance monitoring configuration",
                vec![
                    ConfigSettingBuilder::boolean(
                        "metrics.enabled",
                        "Enable Metrics",
                        config.metrics.enabled,
                        "Enable metrics collection",
                    ),
                    ConfigSettingBuilder::number(
                        "metrics.collection_interval",
                        "Collection Interval",
                        format!("{}s", config.metrics.collection_interval),
                        "Metrics collection interval in seconds",
                    ),
                    ConfigSettingBuilder::number(
                        "metrics.retention_days",
                        "Retention Days",
                        format!("{} days", config.metrics.retention_days),
                        "How long to keep metrics data",
                    ),
                ],
            ),
        ];

        Ok(ConfigurationViewModel {
            page: "config",
            page_description: "Manage system settings and parameters",
            categories,
        })
    }

    // =========================================================================
    // Logs Page Builders
    // =========================================================================

    /// Build logs page view model
    pub async fn build_logs_page(&self) -> Result<LogsViewModel> {
        // Create filter manually as LogFilter doesn't impl Default
        let filter = crate::application::admin::types::LogFilter {
            level: None,
            module: None,
            message_contains: None,
            start_time: None,
            end_time: None,
            limit: Some(100),
        };
        let logs = self
            .state
            .admin_service
            .get_logs(filter)
            .await
            .context("Failed to get logs")?;

        let stats = self
            .state
            .admin_service
            .get_log_stats()
            .await
            .context("Failed to get log stats")?;

        let entries: Vec<LogEntryViewModel> = logs
            .entries
            .into_iter()
            .map(|entry| {
                let level_class = css::badge_for_level(&entry.level);

                LogEntryViewModel {
                    timestamp: entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                    level: entry.level,
                    level_class,
                    message: entry.message,
                    source: entry.module, // Use module as source
                }
            })
            .collect();

        // Extract counts from entries_by_level HashMap
        let errors = *stats.entries_by_level.get("error").unwrap_or(&0);
        let warnings = *stats.entries_by_level.get("warn").unwrap_or(&0)
            + *stats.entries_by_level.get("warning").unwrap_or(&0);
        let info = *stats.entries_by_level.get("info").unwrap_or(&0);

        Ok(LogsViewModel {
            page: "logs",
            page_description: "View and filter system logs",
            entries,
            total_count: logs.total_count,
            stats: LogStatsViewModel {
                total: stats.total_entries,
                errors,
                warnings,
                info,
            },
        })
    }

    // =========================================================================
    // Data Management Builders
    // =========================================================================

    /// Build data management page view model
    ///
    /// Fetches backup information from the admin service and transforms
    /// it into a view model suitable for the data management template.
    pub async fn build_data_management(&self) -> Result<DataManagementViewModel> {
        // Fetch backup list from admin service
        let backups = self
            .state
            .admin_service
            .list_backups()
            .await
            .context("Failed to fetch backup list")?;

        // Transform BackupInfo into BackupViewModel
        let backup_view_models = backups
            .iter()
            .map(BackupViewModel::from_backup_info)
            .collect();

        Ok(DataManagementViewModel::new(backup_view_models))
    }

    // =========================================================================
    // Diagnostics Builders
    // =========================================================================

    /// Build diagnostics page view model
    ///
    /// Fetches health check results from the admin service and transforms
    /// them into a view model for the diagnostics template.
    pub async fn build_diagnostics_page(&self) -> Result<DiagnosticsViewModel> {
        // Fetch actual health check results from admin service
        let health_result = self
            .state
            .admin_service
            .run_health_check()
            .await
            .context("Failed to run health check")?;

        // Transform HealthCheckResult into view model items
        let checks: Vec<HealthCheckItemViewModel> = health_result
            .checks
            .into_iter()
            .map(|check| HealthCheckItemViewModel {
                name: check.name,
                status: check.status,
                message: Some(check.message),
                duration_ms: Some(check.duration_ms),
            })
            .collect();

        Ok(DiagnosticsViewModel::new().with_health_check(
            health_result.overall_status,
            health_result.duration_ms,
            checks,
        ))
    }

    // =========================================================================
    // Error Page Builder
    // =========================================================================

    /// Build error page view model
    pub fn build_error(title: &str, message: &str, details: Option<&str>) -> ErrorViewModel {
        let mut vm = ErrorViewModel::new(title, message);
        if let Some(d) = details {
            vm = vm.with_details(d);
        }
        vm
    }
}
