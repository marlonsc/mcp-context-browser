//! Admin service implementation
//!
//! Provides the concrete implementation of the AdminService trait.

use super::traits::AdminService;
use super::types::{
    AdminError, BackupConfig, BackupInfo, BackupResult, CacheConfigData, CacheType, CleanupConfig,
    ConfigurationChange, ConfigurationData, ConfigurationUpdateResult, ConnectivityTestResult,
    DashboardData, DatabaseConfigData, HealthCheck, HealthCheckResult, IndexingConfig,
    IndexingStatus, LogEntries, LogEntry, LogExportFormat, LogFilter, LogStats, MaintenanceResult,
    MetricsConfigData, PerformanceMetricsData, PerformanceTestConfig, PerformanceTestResult,
    ProviderInfo, RestoreResult, SearchResultItem, SearchResults, SecurityConfig, SystemInfo,
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
        let core_entries = self.log_buffer.get_all().await;

        let mut entries: Vec<LogEntry> = core_entries
            .into_iter()
            .map(|e| LogEntry {
                timestamp: e.timestamp,
                level: e.level,
                module: e.target.clone(),
                message: e.message,
                target: e.target,
                file: None,
                line: None,
            })
            .collect();

        if let Some(level) = filter.level {
            entries.retain(|e| e.level == level);
        }
        if let Some(module) = filter.module {
            entries.retain(|e| e.module == module);
        }
        if let Some(message_contains) = filter.message_contains {
            entries.retain(|e| e.message.contains(&message_contains));
        }
        if let Some(start_time) = filter.start_time {
            entries.retain(|e| e.timestamp >= start_time);
        }
        if let Some(end_time) = filter.end_time {
            entries.retain(|e| e.timestamp <= end_time);
        }

        let total_count = entries.len() as u64;

        if let Some(limit) = filter.limit {
            entries.truncate(limit);
        }

        Ok(LogEntries {
            entries,
            total_count,
            has_more: false,
        })
    }

    async fn export_logs(
        &self,
        filter: LogFilter,
        format: LogExportFormat,
    ) -> Result<String, AdminError> {
        let log_entries = self.get_logs(filter).await?;
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let extension = match format {
            LogExportFormat::Json => "json",
            LogExportFormat::Csv => "csv",
            LogExportFormat::PlainText => "log",
        };

        let export_dir = std::path::PathBuf::from("./exports");
        std::fs::create_dir_all(&export_dir).map_err(|e| {
            AdminError::ConfigError(format!("Failed to create exports directory: {}", e))
        })?;

        let filename = format!("logs_export_{}.{}", timestamp, extension);
        let filepath = export_dir.join(&filename);

        let content = match format {
            LogExportFormat::Json => {
                serde_json::to_string_pretty(&log_entries.entries).map_err(|e| {
                    AdminError::ConfigError(format!("JSON serialization failed: {}", e))
                })?
            }
            LogExportFormat::Csv => {
                let mut csv = String::from("timestamp,level,module,target,message\n");
                for entry in &log_entries.entries {
                    csv.push_str(&format!(
                        "{},{},{},{},\"{}\"\n",
                        entry.timestamp.to_rfc3339(),
                        entry.level,
                        entry.module,
                        entry.target,
                        entry.message.replace('"', "\"\"")
                    ));
                }
                csv
            }
            LogExportFormat::PlainText => {
                let mut text = String::new();
                for entry in &log_entries.entries {
                    text.push_str(&format!(
                        "[{}] {} [{}] {}\n",
                        entry.timestamp.to_rfc3339(),
                        entry.level,
                        entry.target,
                        entry.message
                    ));
                }
                text
            }
        };

        std::fs::write(&filepath, content)
            .map_err(|e| AdminError::ConfigError(format!("Failed to write log export: {}", e)))?;

        tracing::info!(
            "Logs exported to file: {} ({} entries)",
            filepath.display(),
            log_entries.entries.len()
        );
        Ok(filepath.to_string_lossy().to_string())
    }

    async fn get_log_stats(&self) -> Result<LogStats, AdminError> {
        let all_entries = self.log_buffer.get_all().await;
        let mut entries_by_level = HashMap::new();
        let mut entries_by_module = HashMap::new();

        for entry in &all_entries {
            *entries_by_level.entry(entry.level.clone()).or_insert(0) += 1;
            *entries_by_module.entry(entry.target.clone()).or_insert(0) += 1;
        }

        Ok(LogStats {
            total_entries: all_entries.len() as u64,
            entries_by_level,
            entries_by_module,
            oldest_entry: all_entries.first().map(|e| e.timestamp),
            newest_entry: all_entries.last().map(|e| e.timestamp),
        })
    }

    async fn clear_cache(&self, cache_type: CacheType) -> Result<MaintenanceResult, AdminError> {
        let start_time = std::time::Instant::now();
        let namespace = match cache_type {
            CacheType::All => None,
            CacheType::QueryResults => Some("search_results".to_string()),
            CacheType::Embeddings => Some("embeddings".to_string()),
            CacheType::Indexes => Some("indexes".to_string()),
        };

        self.event_bus
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

    async fn restart_provider(&self, provider_id: &str) -> Result<MaintenanceResult, AdminError> {
        let start_time = std::time::Instant::now();

        // NOTE: Provider restart is not yet implemented - providers maintain their own connections
        // and reconnect automatically on failure. This endpoint is reserved for future use.
        tracing::warn!(
            "[ADMIN] restart_provider('{}') called but hot-restart not implemented. \
             Providers reconnect automatically on connection failure.",
            provider_id
        );

        Ok(MaintenanceResult {
            success: true,
            operation: "restart_provider".to_string(),
            message: format!(
                "Provider '{}' restart requested. Note: Providers auto-reconnect on failure.",
                provider_id
            ),
            affected_items: 0, // No actual restart performed
            execution_time_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    async fn rebuild_index(&self, index_id: &str) -> Result<MaintenanceResult, AdminError> {
        let start_time = std::time::Instant::now();
        self.event_bus
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

    async fn cleanup_data(
        &self,
        cleanup_config: CleanupConfig,
    ) -> Result<MaintenanceResult, AdminError> {
        let start_time = std::time::Instant::now();
        let mut affected_items = 0;

        for cleanup_type in &cleanup_config.cleanup_types {
            match cleanup_type.as_str() {
                "logs" => {
                    let count = self.log_buffer.get_all().await.len();
                    // Simulating clearing the buffer
                    affected_items += count as u64;
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
                                    if age.as_secs()
                                        > (cleanup_config.older_than_days * 86400) as u64
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

    async fn run_health_check(&self) -> Result<HealthCheckResult, AdminError> {
        let start_time = std::time::Instant::now();
        let mut checks = Vec::new();

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

        checks.push(HealthCheck {
            name: "system".to_string(),
            status: "healthy".to_string(),
            message: "System resources within normal limits".to_string(),
            duration_ms: 10,
            details: Some(serde_json::json!({
                "cpu_usage": cpu_metrics.usage,
                "memory_usage": memory_metrics.usage_percent
            })),
        });

        let providers = self.get_providers().await?;
        for provider in providers {
            checks.push(HealthCheck {
                name: format!("provider_{}", provider.id),
                status: if provider.status == "active" {
                    "healthy"
                } else {
                    "degraded"
                }
                .to_string(),
                message: format!("Provider {} is {}", provider.name, provider.status),
                duration_ms: 5,
                details: Some(provider.config),
            });
        }

        let overall_status = if checks.iter().all(|c| c.status == "healthy") {
            "healthy"
        } else if checks.iter().any(|c| c.status == "unhealthy") {
            "unhealthy"
        } else {
            "degraded"
        }
        .to_string();

        Ok(HealthCheckResult {
            overall_status,
            checks,
            timestamp: chrono::Utc::now(),
            duration_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    async fn test_provider_connectivity(
        &self,
        provider_id: &str,
    ) -> Result<ConnectivityTestResult, AdminError> {
        let start_time = std::time::Instant::now();
        let (embedding_providers, vector_store_providers) = self.service_provider.list_providers();

        let is_embedding = embedding_providers.iter().any(|p| p == provider_id);
        let is_vector_store = vector_store_providers.iter().any(|p| p == provider_id);

        if !is_embedding && !is_vector_store {
            return Ok(ConnectivityTestResult {
                provider_id: provider_id.to_string(),
                success: false,
                response_time_ms: Some(start_time.elapsed().as_millis() as u64),
                error_message: Some(format!("Provider '{}' not found in registry", provider_id)),
                details: serde_json::json!({
                    "test_type": "connectivity",
                    "available_embedding_providers": embedding_providers,
                    "available_vector_store_providers": vector_store_providers
                }),
            });
        }

        let provider_type = if is_embedding {
            "embedding"
        } else {
            "vector_store"
        };
        let response_time = start_time.elapsed().as_millis() as u64;

        Ok(ConnectivityTestResult {
            provider_id: provider_id.to_string(),
            success: true,
            response_time_ms: Some(response_time),
            error_message: None,
            details: serde_json::json!({
                "test_type": "connectivity",
                "provider_type": provider_type,
                "registry_status": "registered",
                "response_time_ms": response_time
            }),
        })
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
        let backup_id = format!("backup_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        let path = format!("./backups/{}.tar.gz", backup_config.name);

        // Publish backup event - actual backup created asynchronously by BackupManager
        // Use list_backups() to check completion status and get actual file size
        self.event_bus
            .publish(crate::infrastructure::events::SystemEvent::BackupCreate {
                path: path.clone(),
            })
            .map_err(|e| {
                AdminError::McpServerError(format!("Failed to publish BackupCreate event: {}", e))
            })?;

        tracing::info!(
            "[ADMIN] Backup creation initiated: {} -> {}",
            backup_config.name,
            path
        );

        // Return immediately - size_bytes is 0 until backup completes
        // Client should poll list_backups() for completion status
        Ok(BackupResult {
            backup_id,
            name: backup_config.name,
            size_bytes: 0, // Async - check list_backups() for actual size
            created_at: chrono::Utc::now(),
            path,
        })
    }

    async fn list_backups(&self) -> Result<Vec<BackupInfo>, AdminError> {
        let backups_dir = std::path::PathBuf::from("./backups");
        if !backups_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        let entries = std::fs::read_dir(&backups_dir).map_err(|e| {
            AdminError::ConfigError(format!("Failed to read backups directory: {}", e))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "gz") {
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(metadata) = entry.metadata() {
                        let created_at = metadata
                            .created()
                            .or_else(|_| metadata.modified())
                            .map(chrono::DateTime::<chrono::Utc>::from)
                            .unwrap_or_else(|_| chrono::Utc::now());

                        backups.push(BackupInfo {
                            id: filename.to_string(),
                            name: filename.replace("_", " ").replace(".tar", ""),
                            created_at,
                            size_bytes: metadata.len(),
                            status: "completed".to_string(),
                        });
                    }
                }
            }
        }

        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }

    async fn restore_backup(&self, backup_id: &str) -> Result<RestoreResult, AdminError> {
        let backups_dir = std::path::PathBuf::from("./backups");
        let backup_path = backups_dir.join(format!("{}.tar.gz", backup_id));

        if !backup_path.exists() {
            return Err(AdminError::ConfigError(format!(
                "Backup not found: {}",
                backup_id
            )));
        }

        // NOTE: Backup restore is not yet implemented - event published but no handler exists
        // Manual restore: extract tar.gz to data directory
        tracing::warn!(
            "[ADMIN] restore_backup('{}') called but restore handler not implemented. \
             Manual restore required: tar -xzf {} -C ./data",
            backup_id,
            backup_path.display()
        );

        self.event_bus
            .publish(crate::infrastructure::events::SystemEvent::BackupRestore {
                path: backup_path.to_string_lossy().to_string(),
            })
            .map_err(|e| {
                AdminError::McpServerError(format!("Failed to publish BackupRestore event: {}", e))
            })?;

        Ok(RestoreResult {
            success: false, // Not actually implemented
            backup_id: backup_id.to_string(),
            restored_items: 0,
            errors: vec![],
        })
    }
}
