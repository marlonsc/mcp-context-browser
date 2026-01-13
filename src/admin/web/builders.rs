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
use crate::admin::models::AdminState;
use crate::infrastructure::utils::{css, FormattingUtils, HealthUtils, TimeUtils};

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
        let filter = crate::admin::service::LogFilter {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tera::{Context, Tera};

    #[test]
    fn test_error_view_model() {
        let vm = ViewModelBuilder::build_error("Test Error", "Something went wrong", None);
        assert_eq!(vm.title, "Test Error");
        assert_eq!(vm.message, "Something went wrong");
        assert!(vm.details.is_none());

        let vm = ViewModelBuilder::build_error(
            "Another Error",
            "Details below",
            Some("Stack trace here"),
        );
        assert_eq!(vm.details, Some("Stack trace here".to_string()));
    }

    // ==========================================================================
    // TEMPLATE RENDERING TESTS - Validate ALL templates with REAL data
    // ==========================================================================

    fn create_test_tera() -> Tera {
        let mut tera = Tera::default();
        tera.add_raw_template("icons.html", include_str!("templates/icons.html"))
            .expect("Failed to load icons.html");
        tera.add_raw_template("base.html", include_str!("templates/base.html"))
            .expect("Failed to load base.html");
        tera.add_raw_template("dashboard.html", include_str!("templates/dashboard.html"))
            .expect("Failed to load dashboard.html");
        tera.add_raw_template("providers.html", include_str!("templates/providers.html"))
            .expect("Failed to load providers.html");
        tera.add_raw_template("indexes.html", include_str!("templates/indexes.html"))
            .expect("Failed to load indexes.html");
        tera.add_raw_template(
            "configuration.html",
            include_str!("templates/configuration.html"),
        )
        .expect("Failed to load configuration.html");
        tera.add_raw_template("logs.html", include_str!("templates/logs.html"))
            .expect("Failed to load logs.html");
        tera.add_raw_template("error.html", include_str!("templates/error.html"))
            .expect("Failed to load error.html");
        tera.add_raw_template(
            "htmx/dashboard_metrics.html",
            include_str!("templates/htmx/dashboard_metrics.html"),
        )
        .expect("Failed to load htmx/dashboard_metrics.html");
        tera.add_raw_template(
            "htmx/providers_list.html",
            include_str!("templates/htmx/providers_list.html"),
        )
        .expect("Failed to load htmx/providers_list.html");
        tera.add_raw_template(
            "htmx/indexes_list.html",
            include_str!("templates/htmx/indexes_list.html"),
        )
        .expect("Failed to load htmx/indexes_list.html");
        tera
    }

    fn create_dashboard_view_model() -> DashboardViewModel {
        DashboardViewModel {
            page: "dashboard",
            metrics: MetricsViewModel::new(45.5, 62.3, 1234, 15.7),
            providers: ProvidersViewModel::new(vec![
                ProviderViewModel::new(
                    "openai-1".to_string(),
                    "OpenAI GPT".to_string(),
                    "embedding".to_string(),
                    "available".to_string(),
                ),
                ProviderViewModel::new(
                    "ollama-1".to_string(),
                    "Ollama Local".to_string(),
                    "embedding".to_string(),
                    "unavailable".to_string(),
                ),
            ]),
            indexes: IndexesSummaryViewModel {
                active_count: 1,
                total_documents: 5000,
                total_documents_formatted: "5,000".to_string(),
                is_indexing: false,
            },
            activities: vec![
                ActivityViewModel::new(
                    "act-1".to_string(),
                    "Index completed successfully".to_string(),
                    chrono::Utc::now(),
                    "success",
                    "indexing".to_string(),
                ),
                ActivityViewModel::new(
                    "act-2".to_string(),
                    "Provider health check failed".to_string(),
                    chrono::Utc::now(),
                    "error",
                    "health".to_string(),
                ),
            ],
            system_health: HealthViewModel::new("healthy", 3661, 12345),
        }
    }

    #[test]
    fn test_dashboard_template_renders() {
        let tera = create_test_tera();
        let vm = create_dashboard_view_model();
        let vm_json = serde_json::to_string(&vm).expect("Failed to serialize view model");

        let mut context = Context::new();
        context.insert("vm", &vm);
        context.insert("vm_json", &vm_json);
        context.insert("page", &vm.page);

        let result = tera.render("dashboard.html", &context);
        assert!(
            result.is_ok(),
            "Dashboard template failed to render: {:?}",
            result.err()
        );

        let html = result.unwrap();
        assert!(
            html.contains("System Dashboard"),
            "Dashboard should contain title"
        );
        assert!(
            html.len() > 1000,
            "Dashboard should have substantial content"
        );
    }

    #[test]
    fn test_providers_template_renders() {
        let tera = create_test_tera();
        let vm = ProvidersViewModel::new(vec![ProviderViewModel::new(
            "openai-1".to_string(),
            "OpenAI".to_string(),
            "embedding".to_string(),
            "available".to_string(),
        )]);

        let mut context = Context::new();
        context.insert("vm", &vm);
        context.insert("page", &vm.page);

        let result = tera.render("providers.html", &context);
        assert!(
            result.is_ok(),
            "Providers template failed to render: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_indexes_template_renders() {
        let tera = create_test_tera();
        let vm = IndexesViewModel::new(
            vec![IndexViewModel::new(
                "main-index".to_string(),
                "Main Codebase Index".to_string(),
                "active".to_string(),
                5000,
                1704067200,
                1704153600,
            )],
            5000,
        );

        let mut context = Context::new();
        context.insert("vm", &vm);
        context.insert("page", &vm.page);

        let result = tera.render("indexes.html", &context);
        assert!(
            result.is_ok(),
            "Indexes template failed to render: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_configuration_template_renders() {
        let tera = create_test_tera();
        let vm = ConfigurationViewModel {
            page: "config",
            page_description: "Manage system settings",
            categories: vec![ConfigCategoryViewModel {
                name: "Indexing".to_string(),
                description: "Indexing settings".to_string(),
                settings: vec![ConfigSettingViewModel {
                    key: "indexing.chunk_size".to_string(),
                    label: "Chunk Size".to_string(),
                    value: serde_json::json!(512),
                    value_display: "512".to_string(),
                    setting_type: "number",
                    description: "Size of chunks".to_string(),
                    editable: true,
                }],
            }],
        };

        let mut context = Context::new();
        context.insert("vm", &vm);
        // Use the &'static str directly without extra reference
        context.insert("page", vm.page);
        context.insert("page_description", vm.page_description);

        let result = tera.render("configuration.html", &context);
        assert!(
            result.is_ok(),
            "Configuration template failed to render: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_logs_template_renders() {
        let tera = create_test_tera();
        let vm = LogsViewModel {
            page: "logs",
            page_description: "View and filter system logs",
            entries: vec![LogEntryViewModel {
                timestamp: "2024-01-01 12:00:00".to_string(),
                level: "INFO".to_string(),
                level_class: css::badge::INFO,
                message: "Server started".to_string(),
                source: "main".to_string(),
            }],
            total_count: 1,
            stats: LogStatsViewModel {
                total: 100,
                errors: 5,
                warnings: 10,
                info: 85,
            },
        };

        let mut context = Context::new();
        context.insert("vm", &vm);
        // Use the &'static str directly without extra reference
        context.insert("page", vm.page);
        context.insert("page_description", vm.page_description);

        let result = tera.render("logs.html", &context);
        assert!(
            result.is_ok(),
            "Logs template failed to render: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_error_template_renders() {
        let tera = create_test_tera();
        let error_vm = ErrorViewModel::new("Test Error", "Something went wrong");

        let mut context = Context::new();
        context.insert("error", &error_vm);
        context.insert("page", "error");

        let result = tera.render("error.html", &context);
        assert!(
            result.is_ok(),
            "Error template failed to render: {:?}",
            result.err()
        );

        let html = result.unwrap();
        assert!(
            html.contains("Test Error"),
            "Error page should contain error title"
        );
    }

    #[test]
    fn test_htmx_dashboard_metrics_renders() {
        let tera = create_test_tera();
        let vm = create_dashboard_view_model();

        let mut context = Context::new();
        context.insert("vm", &vm);

        let result = tera.render("htmx/dashboard_metrics.html", &context);
        assert!(
            result.is_ok(),
            "HTMX dashboard metrics failed to render: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_htmx_providers_list_renders() {
        let tera = create_test_tera();
        let providers = vec![ProviderViewModel::new(
            "openai-1".to_string(),
            "OpenAI".to_string(),
            "embedding".to_string(),
            "available".to_string(),
        )];

        let mut context = Context::new();
        context.insert("providers", &providers);

        let result = tera.render("htmx/providers_list.html", &context);
        assert!(
            result.is_ok(),
            "HTMX providers list failed to render: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_htmx_indexes_list_renders() {
        let tera = create_test_tera();
        let indexes = vec![IndexViewModel::new(
            "main-index".to_string(),
            "Main Index".to_string(),
            "active".to_string(),
            1000,
            1704067200,
            1704153600,
        )];

        let mut context = Context::new();
        context.insert("indexes", &indexes);

        let result = tera.render("htmx/indexes_list.html", &context);
        assert!(
            result.is_ok(),
            "HTMX indexes list failed to render: {:?}",
            result.err()
        );
    }
}
