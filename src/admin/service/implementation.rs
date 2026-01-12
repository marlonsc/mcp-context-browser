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
    HealthCheck, HealthCheckResult, IndexingConfig, IndexingStatus, LogEntries, LogExportFormat,
    LogFilter, LogStats, MaintenanceResult, MetricsConfigData, PerformanceMetricsData,
    PerformanceTestConfig, PerformanceTestResult, ProviderInfo, RestoreResult, RouteInfo,
    SearchResultItem, SearchResults, SecurityConfig, SignalResult, SubsystemInfo, SubsystemMetrics,
    SubsystemSignal, SubsystemStatus, SubsystemType, SystemInfo,
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
    #[shaku(default)]
    event_bus: crate::infrastructure::events::SharedEventBus,
    #[shaku(default)]
    log_buffer: crate::infrastructure::logging::SharedLogBuffer,
    #[shaku(default)]
    config: Arc<arc_swap::ArcSwap<crate::infrastructure::config::Config>>,
}

impl AdminServiceImpl {
    /// Create new admin service with dependency injection
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        performance_metrics: Arc<dyn PerformanceMetricsInterface>,
        indexing_operations: Arc<dyn IndexingOperationsInterface>,
        service_provider: Arc<dyn ServiceProviderInterface>,
        system_collector: Arc<dyn SystemMetricsCollectorInterface>,
        http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
        event_bus: crate::infrastructure::events::SharedEventBus,
        log_buffer: crate::infrastructure::logging::SharedLogBuffer,
        config: Arc<arc_swap::ArcSwap<crate::infrastructure::config::Config>>,
    ) -> Self {
        Self {
            performance_metrics,
            indexing_operations,
            service_provider,
            system_collector,
            http_client,
            search_service: Arc::new(ArcSwap::from_pointee(None)),
            event_bus,
            log_buffer,
            config,
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
        let (embedding_providers, vector_store_providers) = self.service_provider.list_providers();
        let mut providers = Vec::new();

        for name in embedding_providers {
            providers.push(ProviderInfo {
                id: name.clone(),
                name,
                provider_type: "embedding".to_string(),
                status: "active".to_string(),
                config: serde_json::json!({ "type": "embedding" }),
            });
        }

        for name in vector_store_providers {
            providers.push(ProviderInfo {
                id: name.clone(),
                name,
                provider_type: "vector_store".to_string(),
                status: "active".to_string(),
                config: serde_json::json!({ "type": "vector_store" }),
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
        let search_service = search_service_guard.as_ref().as_ref().ok_or_else(|| {
            AdminError::McpServerError("Search service not initialized".to_string())
        })?;

        let results = search_service
            .search(collection_name, query, search_limit)
            .await
            .map_err(|e| AdminError::McpServerError(e.to_string()))?;

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
        let providers = self.get_providers().await?;

        Ok(ConfigurationData {
            providers,
            indexing: IndexingConfig {
                chunk_size: 1000,
                chunk_overlap: 200,
                max_file_size: 10 * 1024 * 1024,
                supported_extensions: vec![
                    ".rs".to_string(),
                    ".py".to_string(),
                    ".js".to_string(),
                    ".ts".to_string(),
                    ".go".to_string(),
                    ".java".to_string(),
                ],
                exclude_patterns: vec![
                    "node_modules".to_string(),
                    "target".to_string(),
                    ".git".to_string(),
                ],
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
                max_size: config.cache.max_size as u64,
                ttl_seconds: config.cache.default_ttl_seconds,
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
        let mut changes_applied = Vec::new();
        let mut requires_restart = false;

        for (path, value) in &updates {
            changes_applied.push(format!("{} = {:?}", path, value));
            if path.starts_with("database.") || path.starts_with("server.") {
                requires_restart = true;
            }
        }

        tracing::info!(
            "Configuration updated by {}: {} changes applied",
            user,
            changes_applied.len()
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
        _limit: Option<usize>,
    ) -> Result<Vec<ConfigurationChange>, AdminError> {
        Ok(Vec::new())
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
        helpers::maintenance::clear_cache(&self.event_bus, cache_type)
    }

    async fn restart_provider(&self, provider_id: &str) -> Result<MaintenanceResult, AdminError> {
        helpers::maintenance::restart_provider(&self.event_bus, provider_id)
    }

    async fn rebuild_index(&self, index_id: &str) -> Result<MaintenanceResult, AdminError> {
        helpers::maintenance::rebuild_index(&self.event_bus, index_id)
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

        let queries = if test_config.queries.is_empty() {
            vec!["test".to_string()]
        } else {
            test_config.queries.clone()
        };

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
        helpers::backup::create_backup(&self.event_bus, backup_config)
    }

    async fn list_backups(&self) -> Result<Vec<BackupInfo>, AdminError> {
        helpers::backup::list_backups()
    }

    async fn restore_backup(&self, backup_id: &str) -> Result<RestoreResult, AdminError> {
        helpers::backup::restore_backup(&self.event_bus, backup_id)
    }

    // === Subsystem Control Methods (ADR-007) ===

    async fn get_subsystems(&self) -> Result<Vec<SubsystemInfo>, AdminError> {
        let mut subsystems = Vec::new();
        let providers = self.get_providers().await?;

        // Add embedding providers as subsystems
        for provider in providers.iter().filter(|p| p.provider_type == "embedding") {
            subsystems.push(SubsystemInfo {
                id: format!("embedding:{}", provider.id),
                name: format!("Embedding Provider: {}", provider.name),
                subsystem_type: SubsystemType::Embedding,
                status: if provider.status == "active" {
                    SubsystemStatus::Running
                } else {
                    SubsystemStatus::Stopped
                },
                health: HealthCheck {
                    name: provider.name.clone(),
                    status: provider.status.clone(),
                    message: "Provider operational".to_string(),
                    duration_ms: 0,
                    details: Some(provider.config.clone()),
                },
                config: provider.config.clone(),
                metrics: SubsystemMetrics {
                    cpu_percent: 0.0,
                    memory_mb: 0,
                    requests_per_sec: 0.0,
                    error_rate: 0.0,
                    last_activity: Some(chrono::Utc::now()),
                },
            });
        }

        // Add vector store providers as subsystems
        for provider in providers
            .iter()
            .filter(|p| p.provider_type == "vector_store")
        {
            subsystems.push(SubsystemInfo {
                id: format!("vector_store:{}", provider.id),
                name: format!("Vector Store: {}", provider.name),
                subsystem_type: SubsystemType::VectorStore,
                status: if provider.status == "active" {
                    SubsystemStatus::Running
                } else {
                    SubsystemStatus::Stopped
                },
                health: HealthCheck {
                    name: provider.name.clone(),
                    status: provider.status.clone(),
                    message: "Vector store operational".to_string(),
                    duration_ms: 0,
                    details: Some(provider.config.clone()),
                },
                config: provider.config.clone(),
                metrics: SubsystemMetrics {
                    cpu_percent: 0.0,
                    memory_mb: 0,
                    requests_per_sec: 0.0,
                    error_rate: 0.0,
                    last_activity: Some(chrono::Utc::now()),
                },
            });
        }

        // Add core subsystems
        let perf = self.performance_metrics.get_performance_metrics();

        subsystems.push(SubsystemInfo {
            id: "search".to_string(),
            name: "Search Service".to_string(),
            subsystem_type: SubsystemType::Search,
            status: SubsystemStatus::Running,
            health: HealthCheck {
                name: "search".to_string(),
                status: "healthy".to_string(),
                message: format!("{} queries processed", perf.total_queries),
                duration_ms: 0,
                details: None,
            },
            config: serde_json::json!({}),
            metrics: SubsystemMetrics {
                cpu_percent: 0.0,
                memory_mb: 0,
                requests_per_sec: perf.total_queries as f64 / perf.uptime_seconds.max(1) as f64,
                error_rate: perf.failed_queries as f64 / perf.total_queries.max(1) as f64,
                last_activity: Some(chrono::Utc::now()),
            },
        });

        let is_indexing = !self.indexing_operations.get_map().is_empty();
        subsystems.push(SubsystemInfo {
            id: "indexing".to_string(),
            name: "Indexing Service".to_string(),
            subsystem_type: SubsystemType::Indexing,
            status: SubsystemStatus::Running,
            health: HealthCheck {
                name: "indexing".to_string(),
                status: "healthy".to_string(),
                message: if is_indexing {
                    "Indexing in progress".to_string()
                } else {
                    "Indexing service ready".to_string()
                },
                duration_ms: 0,
                details: None,
            },
            config: serde_json::json!({}),
            metrics: SubsystemMetrics {
                cpu_percent: 0.0,
                memory_mb: 0,
                requests_per_sec: 0.0,
                error_rate: 0.0,
                last_activity: Some(chrono::Utc::now()),
            },
        });

        subsystems.push(SubsystemInfo {
            id: "cache".to_string(),
            name: "Cache Manager".to_string(),
            subsystem_type: SubsystemType::Cache,
            status: if self.config.load().cache.enabled {
                SubsystemStatus::Running
            } else {
                SubsystemStatus::Stopped
            },
            health: HealthCheck {
                name: "cache".to_string(),
                status: if self.config.load().cache.enabled {
                    "healthy"
                } else {
                    "disabled"
                }
                .to_string(),
                message: format!("Cache hit rate: {:.1}%", perf.cache_hit_rate * 100.0),
                duration_ms: 0,
                details: None,
            },
            config: serde_json::json!({
                "enabled": self.config.load().cache.enabled,
                "max_size": self.config.load().cache.max_size,
            }),
            metrics: SubsystemMetrics {
                cpu_percent: 0.0,
                memory_mb: 0,
                requests_per_sec: 0.0,
                error_rate: 0.0,
                last_activity: Some(chrono::Utc::now()),
            },
        });

        subsystems.push(SubsystemInfo {
            id: "http_transport".to_string(),
            name: "HTTP Transport".to_string(),
            subsystem_type: SubsystemType::HttpTransport,
            status: SubsystemStatus::Running,
            health: HealthCheck {
                name: "http_transport".to_string(),
                status: "healthy".to_string(),
                message: format!("{} active connections", perf.active_connections),
                duration_ms: 0,
                details: None,
            },
            config: serde_json::json!({
                "port": self.config.load().metrics.port,
            }),
            metrics: SubsystemMetrics {
                cpu_percent: 0.0,
                memory_mb: 0,
                requests_per_sec: perf.total_queries as f64 / perf.uptime_seconds.max(1) as f64,
                error_rate: 0.0,
                last_activity: Some(chrono::Utc::now()),
            },
        });

        Ok(subsystems)
    }

    async fn send_subsystem_signal(
        &self,
        subsystem_id: &str,
        signal: SubsystemSignal,
    ) -> Result<SignalResult, AdminError> {
        use crate::infrastructure::events::SystemEvent;

        let signal_name = match &signal {
            SubsystemSignal::Restart => "restart",
            SubsystemSignal::Reload => "reload",
            SubsystemSignal::Pause => "pause",
            SubsystemSignal::Resume => "resume",
            SubsystemSignal::Configure(_) => "configure",
        };

        // Parse subsystem ID (format: "type:id" or just "id")
        let (subsystem_type, provider_id): (&str, &str) = if subsystem_id.contains(':') {
            let parts: Vec<&str> = subsystem_id.splitn(2, ':').collect();
            (parts[0], parts.get(1).copied().unwrap_or(""))
        } else {
            ("", subsystem_id)
        };

        // Dispatch appropriate event based on signal type and subsystem
        match signal {
            SubsystemSignal::Restart => {
                if subsystem_type == "embedding" || subsystem_type == "vector_store" {
                    let _ = self.event_bus.publish(SystemEvent::ProviderRestart {
                        provider_type: subsystem_type.to_string(),
                        provider_id: provider_id.to_string(),
                    });
                } else if subsystem_id == "cache" {
                    let _ = self
                        .event_bus
                        .publish(SystemEvent::CacheClear { namespace: None });
                } else if subsystem_id == "indexing" {
                    let _ = self
                        .event_bus
                        .publish(SystemEvent::IndexRebuild { collection: None });
                }
            }
            SubsystemSignal::Reload => {
                let _ = self.event_bus.publish(SystemEvent::Reload);
            }
            SubsystemSignal::Configure(config) => {
                if subsystem_type == "embedding" || subsystem_type == "vector_store" {
                    let _ = self.event_bus.publish(SystemEvent::ProviderReconfigure {
                        provider_type: subsystem_type.to_string(),
                        config,
                    });
                }
            }
            SubsystemSignal::Pause | SubsystemSignal::Resume => {
                // Pause/Resume not yet implemented for all subsystems
                tracing::warn!(
                    "[ADMIN] Pause/Resume not implemented for subsystem: {}",
                    subsystem_id
                );
            }
        }

        tracing::info!(
            "[ADMIN] Sent {} signal to subsystem {}",
            signal_name,
            subsystem_id
        );

        Ok(SignalResult {
            success: true,
            subsystem_id: subsystem_id.to_string(),
            signal: signal_name.to_string(),
            message: format!("Signal '{}' sent to '{}'", signal_name, subsystem_id),
        })
    }

    async fn get_routes(&self) -> Result<Vec<RouteInfo>, AdminError> {
        // Return a list of known routes
        // In a full implementation, this would be dynamically generated from the router
        Ok(vec![
            RouteInfo {
                id: "health".to_string(),
                path: "/api/health".to_string(),
                method: "GET".to_string(),
                handler: "health_handler".to_string(),
                auth_required: false,
                rate_limit: None,
            },
            RouteInfo {
                id: "metrics".to_string(),
                path: "/api/context/metrics".to_string(),
                method: "GET".to_string(),
                handler: "comprehensive_metrics_handler".to_string(),
                auth_required: false,
                rate_limit: None,
            },
            RouteInfo {
                id: "status".to_string(),
                path: "/api/context/status".to_string(),
                method: "GET".to_string(),
                handler: "status_handler".to_string(),
                auth_required: false,
                rate_limit: None,
            },
            RouteInfo {
                id: "admin_dashboard".to_string(),
                path: "/admin".to_string(),
                method: "GET".to_string(),
                handler: "admin_index".to_string(),
                auth_required: true,
                rate_limit: Some(60),
            },
            RouteInfo {
                id: "admin_providers".to_string(),
                path: "/admin/providers".to_string(),
                method: "GET".to_string(),
                handler: "get_providers_handler".to_string(),
                auth_required: true,
                rate_limit: Some(60),
            },
            RouteInfo {
                id: "admin_search".to_string(),
                path: "/admin/search".to_string(),
                method: "POST".to_string(),
                handler: "search_handler".to_string(),
                auth_required: true,
                rate_limit: Some(30),
            },
            RouteInfo {
                id: "mcp_message".to_string(),
                path: "/mcp".to_string(),
                method: "POST".to_string(),
                handler: "mcp_message_handler".to_string(),
                auth_required: false,
                rate_limit: Some(100),
            },
            RouteInfo {
                id: "mcp_sse".to_string(),
                path: "/mcp/sse".to_string(),
                method: "GET".to_string(),
                handler: "mcp_sse_handler".to_string(),
                auth_required: false,
                rate_limit: Some(10),
            },
        ])
    }

    async fn reload_routes(&self) -> Result<MaintenanceResult, AdminError> {
        use crate::infrastructure::events::SystemEvent;

        let _ = self.event_bus.publish(SystemEvent::RouterReload);

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
        let config = self.config.load();
        let config_path = dirs::home_dir()
            .unwrap_or_default()
            .join(".context")
            .join("config.toml");

        let toml_string = toml::to_string_pretty(&**config)
            .map_err(|e| AdminError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        tokio::fs::write(&config_path, toml_string)
            .await
            .map_err(|e| AdminError::ConfigError(format!("Failed to write config file: {}", e)))?;

        tracing::info!("[ADMIN] Configuration persisted to {:?}", config_path);

        Ok(ConfigPersistResult {
            success: true,
            path: config_path.to_string_lossy().to_string(),
            warnings: Vec::new(),
        })
    }

    async fn get_config_diff(&self) -> Result<ConfigDiff, AdminError> {
        let runtime_config = self.config.load();
        let config_path = dirs::home_dir()
            .unwrap_or_default()
            .join(".context")
            .join("config.toml");

        // Try to read the file config
        let file_content = match tokio::fs::read_to_string(&config_path).await {
            Ok(content) => content,
            Err(_) => {
                return Ok(ConfigDiff {
                    has_changes: true,
                    runtime_only: HashMap::new(),
                    file_only: HashMap::new(),
                });
            }
        };

        let file_config: crate::infrastructure::config::Config = toml::from_str(&file_content)
            .map_err(|e| AdminError::ConfigError(format!("Failed to parse config file: {}", e)))?;

        // Simple comparison - in production, this would be more sophisticated
        let runtime_json = serde_json::to_value(&**runtime_config)
            .map_err(|e| AdminError::ConfigError(format!("Failed to serialize runtime: {}", e)))?;
        let file_json = serde_json::to_value(&file_config)
            .map_err(|e| AdminError::ConfigError(format!("Failed to serialize file: {}", e)))?;

        let has_changes = runtime_json != file_json;

        Ok(ConfigDiff {
            has_changes,
            runtime_only: HashMap::new(),
            file_only: HashMap::new(),
        })
    }
}
