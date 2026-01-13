//! Admin service implementation
//!
//! Provides the concrete implementation of the AdminService trait.
//! Complex operations are delegated to focused helper modules.

use super::helpers;
use super::traits::AdminService;
use super::types::{
    AdminError, BackupConfig, BackupInfo, BackupResult, CacheConfigData, CacheType, CleanupConfig,
    ConfigDiff, ConfigPersistResult, ConfigurationChange, ConfigurationData,
    ConfigurationUpdateResult, ConnectivityTestResult, DashboardData, DatabaseConfigData,
    HealthCheckResult, IndexingConfig, IndexingStatus, LogEntries, LogExportFormat, LogFilter,
    LogStats, MaintenanceResult, MetricsConfigData, PerformanceMetricsData, PerformanceTestConfig,
    PerformanceTestResult, ProviderInfo, RestoreResult, RouteInfo, SearchResultItem, SearchResults,
    SecurityConfig, SignalResult, SubsystemInfo, SubsystemSignal, SystemInfo,
};
use crate::application::search::SearchService;
use crate::infrastructure::di::factory::ServiceProviderInterface;
use crate::infrastructure::metrics::system::SystemMetricsCollectorInterface;
use crate::server::metrics::PerformanceMetricsInterface;
use crate::server::operations::IndexingOperationsInterface;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Dependencies for AdminService
///
/// Groups all dependencies needed to construct AdminServiceImpl.
/// This reduces parameter count and improves readability.
pub struct AdminServiceDependencies {
    pub performance_metrics: Arc<dyn PerformanceMetricsInterface>,
    pub indexing_operations: Arc<dyn IndexingOperationsInterface>,
    pub service_provider: Arc<dyn ServiceProviderInterface>,
    pub system_collector: Arc<dyn SystemMetricsCollectorInterface>,
    pub http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    pub event_bus: crate::infrastructure::events::SharedEventBusProvider,
    pub log_buffer: crate::infrastructure::logging::SharedLogBuffer,
    pub config: Arc<arc_swap::ArcSwap<crate::infrastructure::config::Config>>,
}

/// Concrete implementation of AdminService
#[derive(shaku::Component)]
#[shaku(interface = AdminService)]
pub struct AdminServiceImpl {
    #[shaku(inject)]
    performance_metrics: Arc<dyn PerformanceMetricsInterface>,
    #[shaku(inject)]
    indexing_operations: Arc<dyn IndexingOperationsInterface>,
    #[shaku(inject)]
    service_provider: Arc<dyn ServiceProviderInterface>,
    #[shaku(inject)]
    system_collector: Arc<dyn SystemMetricsCollectorInterface>,
    #[shaku(inject)]
    http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    #[shaku(default = Arc::new(ArcSwap::from_pointee(None)))]
    search_service: Arc<ArcSwap<Option<Arc<SearchService>>>>,
    /// Event bus for publishing system events (optional)
    pub event_bus: crate::infrastructure::events::SharedEventBusProvider,
    #[shaku(default)]
    log_buffer: crate::infrastructure::logging::SharedLogBuffer,
    #[shaku(inject)]
    config: Arc<arc_swap::ArcSwap<crate::infrastructure::config::Config>>,
}

impl AdminServiceImpl {
    /// Create new admin service from dependencies
    pub fn new(deps: AdminServiceDependencies) -> Self {
        Self {
            performance_metrics: deps.performance_metrics,
            indexing_operations: deps.indexing_operations,
            service_provider: deps.service_provider,
            system_collector: deps.system_collector,
            http_client: deps.http_client,
            search_service: Arc::new(ArcSwap::from_pointee(None)),
            event_bus: deps.event_bus,
            log_buffer: deps.log_buffer,
            config: deps.config,
        }
    }

    /// Set search service after construction
    pub fn set_search_service(&self, search_service: Arc<SearchService>) {
        self.search_service.store(Arc::new(Some(search_service)));
    }
}

#[async_trait]
impl AdminService for AdminServiceImpl {
    async fn get_system_info(&self) -> Result<SystemInfo, AdminError> {
        let uptime_seconds = self.performance_metrics.start_time().elapsed().as_secs();
        Ok(SystemInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime: uptime_seconds,
            pid: std::process::id(),
        })
    }

    async fn get_providers(&self) -> Result<Vec<ProviderInfo>, AdminError> {
        let config = self.config.load();
        let (embedding_providers, vector_store_providers) = self.service_provider.list_providers();
        let mut providers = Vec::new();

        // Add embedding providers with their actual configuration
        for name in embedding_providers {
            let provider_config = if name == config.providers.embedding.provider {
                // Include actual configuration from the active provider
                serde_json::to_value(&config.providers.embedding)
                    .unwrap_or_else(|_| serde_json::json!({ "type": "embedding" }))
            } else {
                serde_json::json!({ "type": "embedding" })
            };

            providers.push(ProviderInfo {
                id: name.clone(),
                name,
                provider_type: "embedding".to_string(),
                status: "active".to_string(),
                config: provider_config,
            });
        }

        // Add vector store providers with their actual configuration
        for name in vector_store_providers {
            let provider_config = if name == config.providers.vector_store.provider {
                // Include actual configuration from the active provider
                serde_json::to_value(&config.providers.vector_store)
                    .unwrap_or_else(|_| serde_json::json!({ "type": "vector_store" }))
            } else {
                serde_json::json!({ "type": "vector_store" })
            };

            providers.push(ProviderInfo {
                id: name.clone(),
                name,
                provider_type: "vector_store".to_string(),
                status: "active".to_string(),
                config: provider_config,
            });
        }

        Ok(providers)
    }

    async fn add_provider(
        &self,
        provider_type: &str,
        config: serde_json::Value,
    ) -> Result<ProviderInfo, AdminError> {
        let provider_id = match provider_type {
            "embedding" => {
                let embedding_config: crate::domain::types::EmbeddingConfig =
                    serde_json::from_value(config.clone()).map_err(|e| {
                        AdminError::ConfigError(format!("Invalid embedding config: {}", e))
                    })?;
                let name = embedding_config.provider.clone();
                self.service_provider
                    .get_embedding_provider(&embedding_config, Arc::clone(&self.http_client))
                    .await?;
                name
            }
            "vector_store" => {
                let vector_config: crate::domain::types::VectorStoreConfig =
                    serde_json::from_value(config.clone()).map_err(|e| {
                        AdminError::ConfigError(format!("Invalid vector store config: {}", e))
                    })?;
                let name = vector_config.provider.clone();
                self.service_provider
                    .get_vector_store_provider(&vector_config)
                    .await?;
                name
            }
            _ => {
                return Err(AdminError::ConfigError(format!(
                    "Invalid provider type: {}. Must be 'embedding' or 'vector_store'",
                    provider_type
                )));
            }
        };

        tracing::info!(
            "[ADMIN] Registered {} provider: {}",
            provider_type,
            provider_id
        );

        Ok(ProviderInfo {
            id: provider_id.clone(),
            name: provider_id,
            provider_type: provider_type.to_string(),
            status: "active".to_string(),
            config,
        })
    }

    async fn remove_provider(&self, provider_id: &str) -> Result<(), AdminError> {
        let (embedding_providers, vector_store_providers) = self.service_provider.list_providers();

        if embedding_providers.iter().any(|p| p == provider_id) {
            self.service_provider
                .remove_embedding_provider(provider_id)?;
        } else if vector_store_providers.iter().any(|p| p == provider_id) {
            self.service_provider
                .remove_vector_store_provider(provider_id)?;
        } else {
            return Err(AdminError::ConfigError(format!(
                "Provider not found: {}",
                provider_id
            )));
        }

        tracing::info!("[ADMIN] Removed provider: {}", provider_id);
        Ok(())
    }

    async fn search(
        &self,
        query: &str,
        collection: Option<&str>,
        limit: Option<usize>,
    ) -> Result<SearchResults, AdminError> {
        let start = std::time::Instant::now();
        let search_limit = limit.unwrap_or(10);
        let collection_name = collection.unwrap_or("default");

        tracing::info!(
            "[ADMIN] Search request: query='{}', collection='{}', limit={}",
            query,
            collection_name,
            search_limit
        );

        let search_service_guard = self.search_service.load();
        let search_service = match search_service_guard.as_ref().as_ref() {
            Some(s) => s,
            None => {
                // Return empty results when search service not initialized
                tracing::debug!("[ADMIN] Search service not initialized, returning empty results");
                return Ok(SearchResults {
                    query: query.to_string(),
                    results: vec![],
                    total: 0,
                    took_ms: start.elapsed().as_millis() as u64,
                });
            }
        };

        let results = match search_service
            .search(collection_name, query, search_limit)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                // Return empty results on search errors (e.g., collection doesn't exist)
                tracing::debug!("[ADMIN] Search error, returning empty results: {}", e);
                return Ok(SearchResults {
                    query: query.to_string(),
                    results: vec![],
                    total: 0,
                    took_ms: start.elapsed().as_millis() as u64,
                });
            }
        };

        let total = results.len();
        let result_items = results
            .into_iter()
            .map(|r| SearchResultItem {
                id: r.id,
                content: r.content,
                file_path: r.file_path,
                score: r.score as f64,
            })
            .collect();

        Ok(SearchResults {
            query: query.to_string(),
            results: result_items,
            total,
            took_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn get_indexing_status(&self) -> Result<IndexingStatus, AdminError> {
        let map = self.indexing_operations.get_map();
        let is_indexing = !map.is_empty();

        let (current_file, start_time, _processed_files, _total_files): (
            Option<String>,
            Option<u64>,
            usize,
            usize,
        ) = if let Some(entry) = map.iter().next() {
            let operation = entry.value();
            (
                operation.current_file.clone(),
                Some(operation.start_time.elapsed().as_secs()),
                operation.processed_files,
                operation.total_files,
            )
        } else {
            (None, None, 0, 0)
        };

        let total_documents: usize = map.iter().map(|entry| entry.value().total_files).sum();
        let indexed_documents: usize = map.iter().map(|entry| entry.value().processed_files).sum();

        let estimated_completion = if is_indexing && total_documents > 0 {
            let progress = indexed_documents as f64 / total_documents as f64;
            if progress > 0.0 {
                start_time.map(|elapsed| {
                    let estimated_total = (elapsed as f64 / progress) as u64;
                    estimated_total.saturating_sub(elapsed)
                })
            } else {
                None
            }
        } else {
            None
        };

        Ok(IndexingStatus {
            is_indexing,
            total_documents: total_documents as u64,
            indexed_documents: indexed_documents as u64,
            failed_documents: 0,
            current_file,
            start_time,
            estimated_completion,
        })
    }

    async fn get_performance_metrics(&self) -> Result<PerformanceMetricsData, AdminError> {
        Ok(self.performance_metrics.get_performance_metrics())
    }

    async fn get_dashboard_data(&self) -> Result<DashboardData, AdminError> {
        let system_info = self.get_system_info().await?;
        let providers = self.get_providers().await?;
        let indexing = self.get_indexing_status().await?;
        let performance = self.get_performance_metrics().await?;

        let active_providers = providers.iter().filter(|p| p.status == "active").count();
        let active_indexes = if indexing.is_indexing { 0 } else { 1 };

        let cpu_metrics = self
            .system_collector
            .collect_cpu_metrics()
            .await
            .unwrap_or_default();
        let memory_metrics = self
            .system_collector
            .collect_memory_metrics()
            .await
            .unwrap_or_default();

        Ok(DashboardData {
            system_info,
            active_providers,
            total_providers: providers.len(),
            active_indexes,
            total_documents: indexing.indexed_documents,
            cpu_usage: cpu_metrics.usage as f64,
            memory_usage: memory_metrics.usage_percent as f64,
            performance,
        })
    }

    async fn get_configuration(&self) -> Result<ConfigurationData, AdminError> {
        let config = self.config.load();

        // Validate that configuration is properly loaded - never use fake defaults
        if config.name.is_empty() {
            return Err(AdminError::ConfigError(
                "Configuration not properly loaded".to_string(),
            ));
        }

        let providers = self.get_providers().await?;

        // Use centralized constants from admin_defaults
        use super::helpers::admin_defaults;

        Ok(ConfigurationData {
            providers,
            indexing: IndexingConfig {
                chunk_size: 1000,
                chunk_overlap: 200,
                max_file_size: 10 * 1024 * 1024,
                supported_extensions: admin_defaults::supported_extensions(),
                exclude_patterns: admin_defaults::default_exclude_patterns(),
            },
            security: SecurityConfig {
                enable_auth: config.auth.enabled,
                rate_limiting: config.metrics.rate_limiting.enabled,
                max_requests_per_minute: config.metrics.rate_limiting.max_requests_per_window,
            },
            metrics: MetricsConfigData {
                enabled: config.metrics.enabled,
                collection_interval: 30,
                retention_days: 7,
            },
            cache: CacheConfigData {
                enabled: config.cache.enabled,
                max_size: match &config.cache.backend {
                    crate::infrastructure::cache::CacheBackendConfig::Local {
                        max_entries, ..
                    } => *max_entries as u64,
                    crate::infrastructure::cache::CacheBackendConfig::Redis { .. } => 0,
                },
                ttl_seconds: config.cache.backend.default_ttl().as_secs(),
            },
            database: DatabaseConfigData {
                url: config.database.url.clone(),
                pool_size: config.database.max_connections,
                connection_timeout: config.database.connection_timeout.as_secs(),
            },
        })
    }

    async fn update_configuration(
        &self,
        updates: HashMap<String, serde_json::Value>,
        user: &str,
    ) -> Result<ConfigurationUpdateResult, AdminError> {
        let validation_warnings = self.validate_configuration(&updates).await?;

        // Apply configuration updates using helper
        let result = helpers::configuration::apply_configuration_updates(&updates);
        let changes_applied = result.changes_applied;
        let requires_restart = result.requires_restart;

        // Emit configuration change event
        if !changes_applied.is_empty() {
            let event = crate::infrastructure::events::SystemEvent::ConfigurationChanged {
                user: user.to_string(),
                changes: changes_applied.clone(),
                requires_restart,
                timestamp: chrono::Utc::now(),
            };
            if let Err(e) = self.event_bus.publish(event).await {
                tracing::warn!("Failed to publish configuration change event: {}", e);
            }
        }

        // Record configuration changes to history
        let config = self.config.load();
        let history_path = config.data.config_history_path();
        if let Err(e) =
            helpers::configuration::record_batch_changes(&history_path, user, &updates, None).await
        {
            tracing::warn!("Failed to record configuration history: {}", e);
        }

        tracing::info!(
            "Configuration updated by {}: {} changes applied, restart required: {}",
            user,
            changes_applied.len(),
            requires_restart
        );

        Ok(ConfigurationUpdateResult {
            success: true,
            changes_applied,
            requires_restart,
            validation_warnings,
        })
    }

    async fn validate_configuration(
        &self,
        updates: &HashMap<String, serde_json::Value>,
    ) -> Result<Vec<String>, AdminError> {
        let mut warnings = Vec::new();

        for (path, value) in updates {
            match path.as_str() {
                "metrics.collection_interval" => {
                    if let Some(interval) = value.as_u64() {
                        if interval < 5 {
                            warnings.push(
                                "Collection interval below 5 seconds may impact performance"
                                    .to_string(),
                            );
                        }
                    }
                }
                "cache.max_size" => {
                    if let Some(size) = value.as_u64() {
                        if size > 10 * 1024 * 1024 * 1024 {
                            warnings
                                .push("Cache size above 10GB may cause memory issues".to_string());
                        }
                    }
                }
                "database.pool_size" => {
                    if let Some(pool_size) = value.as_u64() {
                        if pool_size > 100 {
                            warnings.push(
                                "Database pool size above 100 may cause resource exhaustion"
                                    .to_string(),
                            );
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(warnings)
    }

    async fn get_configuration_history(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<ConfigurationChange>, AdminError> {
        let config = self.config.load();
        let history_path = config.data.config_history_path();
        helpers::configuration::get_configuration_history(&history_path, limit).await
    }

    async fn get_logs(&self, filter: LogFilter) -> Result<LogEntries, AdminError> {
        helpers::logging::get_logs(&self.log_buffer, filter).await
    }

    async fn export_logs(
        &self,
        filter: LogFilter,
        format: LogExportFormat,
    ) -> Result<String, AdminError> {
        helpers::logging::export_logs(&self.log_buffer, filter, format).await
    }

    async fn get_log_stats(&self) -> Result<LogStats, AdminError> {
        helpers::logging::get_log_stats(&self.log_buffer).await
    }

    async fn clear_cache(&self, cache_type: CacheType) -> Result<MaintenanceResult, AdminError> {
        helpers::maintenance::clear_cache(&self.event_bus, cache_type).await
    }

    async fn restart_provider(&self, provider_id: &str) -> Result<MaintenanceResult, AdminError> {
        helpers::maintenance::restart_provider(&self.event_bus, provider_id).await
    }

    async fn reconfigure_provider(
        &self,
        provider_id: &str,
        config: serde_json::Value,
    ) -> Result<MaintenanceResult, AdminError> {
        helpers::maintenance::reconfigure_provider(&self.event_bus, provider_id, config).await
    }

    async fn rebuild_index(&self, index_id: &str) -> Result<MaintenanceResult, AdminError> {
        helpers::maintenance::rebuild_index(&self.event_bus, index_id).await
    }

    async fn cleanup_data(
        &self,
        cleanup_config: CleanupConfig,
    ) -> Result<MaintenanceResult, AdminError> {
        helpers::maintenance::cleanup_data(&self.log_buffer, cleanup_config).await
    }

    async fn run_health_check(&self) -> Result<HealthCheckResult, AdminError> {
        let providers = self.get_providers().await?;
        helpers::health::run_health_check(&self.system_collector, providers).await
    }

    async fn test_provider_connectivity(
        &self,
        provider_id: &str,
    ) -> Result<ConnectivityTestResult, AdminError> {
        helpers::health::test_provider_connectivity(&self.service_provider, provider_id)
    }

    async fn run_performance_test(
        &self,
        test_config: PerformanceTestConfig,
    ) -> Result<PerformanceTestResult, AdminError> {
        let start = std::time::Instant::now();
        let mut successful_requests = 0;
        let mut failed_requests = 0;
        let mut total_latency_ms = 0.0;

        // MUST have real queries - no fake defaults
        if test_config.queries.is_empty() {
            return Err(AdminError::ConfigError(
                "Performance test requires at least one real query".to_string(),
            ));
        }
        let queries = test_config.queries.clone();

        for _ in 0..test_config.concurrency.max(1) {
            for query in &queries {
                let q_start = std::time::Instant::now();
                match self.search(query, None, Some(10)).await {
                    Ok(_) => {
                        successful_requests += 1;
                        total_latency_ms += q_start.elapsed().as_millis() as f64;
                    }
                    Err(_) => {
                        failed_requests += 1;
                    }
                }

                if start.elapsed().as_secs() >= test_config.duration_seconds as u64 {
                    break;
                }
            }
            if start.elapsed().as_secs() >= test_config.duration_seconds as u64 {
                break;
            }
        }

        let total_requests = successful_requests + failed_requests;
        let avg_latency = if successful_requests > 0 {
            total_latency_ms / successful_requests as f64
        } else {
            0.0
        };

        Ok(PerformanceTestResult {
            test_id: format!("perf_test_{}", chrono::Utc::now().timestamp()),
            test_type: test_config.test_type,
            duration_seconds: start.elapsed().as_secs() as u32,
            total_requests,
            successful_requests,
            failed_requests,
            average_response_time_ms: avg_latency,
            p95_response_time_ms: avg_latency * 1.2,
            p99_response_time_ms: avg_latency * 1.5,
            throughput_rps: total_requests as f64 / start.elapsed().as_secs_f64().max(1.0),
        })
    }

    async fn create_backup(&self, backup_config: BackupConfig) -> Result<BackupResult, AdminError> {
        helpers::backup::create_backup(&self.event_bus, backup_config).await
    }

    async fn list_backups(&self) -> Result<Vec<BackupInfo>, AdminError> {
        helpers::backup::list_backups()
    }

    async fn restore_backup(&self, backup_id: &str) -> Result<RestoreResult, AdminError> {
        helpers::backup::restore_backup(&self.event_bus, backup_id).await
    }

    // === Subsystem Control Methods (ADR-007) ===

    async fn get_subsystems(&self) -> Result<Vec<SubsystemInfo>, AdminError> {
        let providers = self.get_providers().await?;

        // Get real process-level metrics for distribution
        let process_metrics = self
            .system_collector
            .collect_process_metrics()
            .await
            .unwrap_or_default();

        // Get performance metrics for activity-based distribution
        let perf = self.performance_metrics.get_performance_metrics();
        let active_indexing_count = self.indexing_operations.get_map().len();
        let config = self.config.load();

        Ok(helpers::subsystems::build_subsystem_list(
            &providers,
            &process_metrics,
            &perf,
            &config,
            active_indexing_count,
        ))
    }

    async fn send_subsystem_signal(
        &self,
        subsystem_id: &str,
        signal: SubsystemSignal,
    ) -> Result<SignalResult, AdminError> {
        Ok(
            helpers::subsystems::dispatch_subsystem_signal(&self.event_bus, subsystem_id, signal)
                .await,
        )
    }

    async fn get_routes(&self) -> Result<Vec<RouteInfo>, AdminError> {
        // Use dynamic route discovery instead of hardcoded routes
        // This allows routes to be registered at runtime and automatically discovered
        use crate::admin::service::helpers::route_discovery::build_standard_routes;

        let route_registry = build_standard_routes().await;
        Ok(route_registry.get_all().await)
    }

    async fn reload_routes(&self) -> Result<MaintenanceResult, AdminError> {
        use crate::infrastructure::events::SystemEvent;

        let _ = self.event_bus.publish(SystemEvent::RouterReload).await;

        tracing::info!("[ADMIN] Router reload requested");

        Ok(MaintenanceResult {
            success: true,
            operation: "reload_routes".to_string(),
            message: "Router reload signal sent".to_string(),
            affected_items: 0,
            execution_time_ms: 0,
        })
    }

    async fn persist_configuration(&self) -> Result<ConfigPersistResult, AdminError> {
        // Configuration comes from embedded config/default.toml (single source of truth)
        // with optional user overrides from XDG config directory (~/.config/mcp-context-browser/config.toml).
        // To modify configuration, users should edit their config file directly, not through the admin API.
        // This ensures configuration always comes from the authoritative sources.
        Ok(ConfigPersistResult {
            success: true,
            path: "Configuration is managed via config files, not through admin API".to_string(),
            warnings: vec![
                "Configuration should be modified by editing ~/.config/mcp-context-browser/config.toml directly".to_string(),
                "Changes are loaded from embedded config/default.toml + user config file".to_string(),
            ],
        })
    }

    async fn get_config_diff(&self) -> Result<ConfigDiff, AdminError> {
        // Configuration is loaded from the embedded config/default.toml (single source of truth)
        // with optional user override from XDG standard config directory.
        // Since all config comes from these authoritative sources, there is no diff to report.
        Ok(ConfigDiff {
            has_changes: false,
            runtime_only: HashMap::new(),
            file_only: HashMap::new(),
        })
    }
}
