//! Provider Router Module
//!
//! This module provides intelligent provider routing using established patterns
//! and dependency injection, following SOLID principles with proper separation of concerns.

use crate::adapters::providers::routing::circuit_breaker::CircuitBreakerConfig;
use crate::domain::error::{Error, Result};
use crate::domain::ports::{EmbeddingProvider, VectorStoreProvider};
use crate::infrastructure::di::registry::{ProviderRegistry, ProviderRegistryTrait};
use std::sync::Arc;
use tracing::{debug, info, instrument};

// Import types from modules
use crate::adapters::providers::routing::circuit_breaker::CircuitBreaker;
use crate::adapters::providers::routing::cost_tracker::{CostTracker, CostTrackerTrait};
use crate::adapters::providers::routing::failover::{FailoverContext, FailoverManager};
use crate::adapters::providers::routing::health::{HealthMonitor, HealthMonitorTrait};
use crate::adapters::providers::routing::metrics::{
    ProviderMetricsCollector, ProviderMetricsCollectorTrait,
};

/// Context for provider selection decisions
#[derive(Debug, Clone)]
pub struct ProviderContext {
    /// Operation type (embedding, search, etc.)
    pub operation_type: String,
    /// Expected load level
    pub expected_load: LoadLevel,
    /// Cost sensitivity (0.0 = cost insensitive, 1.0 = cost critical)
    pub cost_sensitivity: f64,
    /// Quality requirements (0.0 = any quality, 1.0 = highest quality)
    pub quality_requirement: f64,
    /// Latency sensitivity (0.0 = latency insensitive, 1.0 = real-time)
    pub latency_sensitivity: f64,
    /// Preferred provider types
    pub preferred_providers: Vec<String>,
    /// Excluded providers
    pub excluded_providers: Vec<String>,
    /// Maximum budget for operation
    pub max_budget: Option<f64>,
    /// User ID for personalization
    pub user_id: Option<String>,
    /// Geographic region preference
    pub region: Option<String>,
}

impl Default for ProviderContext {
    fn default() -> Self {
        Self {
            operation_type: "general".to_string(),
            expected_load: LoadLevel::Medium,
            cost_sensitivity: 0.5,
            quality_requirement: 0.5,
            latency_sensitivity: 0.5,
            preferred_providers: Vec::new(),
            excluded_providers: Vec::new(),
            max_budget: None,
            user_id: None,
            region: None,
        }
    }
}

/// Load levels for operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Strategy for provider selection
#[async_trait::async_trait]
pub trait ProviderSelectionStrategy: Send + Sync {
    /// Select a provider from the given candidates based on context
    async fn select_provider(
        &self,
        candidates: &[String],
        context: &ProviderContext,
        health_monitor: &HealthMonitor,
        cost_tracker: &CostTracker,
    ) -> Result<String>;
}

/// Contextual provider selection strategy
pub struct ContextualStrategy {
    /// Weights for different scoring factors
    health_weight: f64,
    cost_weight: f64,
    quality_weight: f64,
    latency_weight: f64,
    load_weight: f64,
}

impl ContextualStrategy {
    /// Create a new contextual strategy with default weights
    pub fn new() -> Self {
        Self {
            health_weight: 0.3,
            cost_weight: 0.25,
            quality_weight: 0.2,
            latency_weight: 0.15,
            load_weight: 0.1,
        }
    }

    /// Create a new contextual strategy with custom weights
    pub fn with_weights(
        health_weight: f64,
        cost_weight: f64,
        quality_weight: f64,
        latency_weight: f64,
        load_weight: f64,
    ) -> Self {
        assert!(
            (health_weight + cost_weight + quality_weight + latency_weight + load_weight - 1.0)
                .abs()
                < f64::EPSILON,
            "Weights must sum to 1.0"
        );

        Self {
            health_weight,
            cost_weight,
            quality_weight,
            latency_weight,
            load_weight,
        }
    }

    /// Calculate quality score for a provider
    fn calculate_quality_score(&self, provider_id: &str) -> f64 {
        match provider_id {
            id if id.contains("openai") => 0.9,
            id if id.contains("anthropic") => 0.95,
            id if id.contains("gemini") => 0.85,
            id if id.contains("ollama") => 0.75,
            _ => 0.6, // Default quality score
        }
    }

    /// Calculate latency score for a provider
    fn calculate_latency_score(&self, provider_id: &str) -> f64 {
        match provider_id {
            id if id.contains("ollama") => 0.9, // Local, fast
            id if id.contains("openai") => 0.7, // Cloud, variable
            id if id.contains("gemini") => 0.8, // Cloud, good
            _ => 0.75,                          // Default latency score
        }
    }

    /// Calculate load compatibility score
    fn calculate_load_score(&self, provider_id: &str, load_level: LoadLevel) -> f64 {
        match load_level {
            LoadLevel::Low => 1.0,
            LoadLevel::Medium => 0.9,
            LoadLevel::High => match provider_id {
                id if id.contains("ollama") => 0.9, // Local handles high load well
                _ => 0.6,                           // Cloud providers may have limits
            },
            LoadLevel::Critical => match provider_id {
                id if id.contains("ollama") => 0.8,
                _ => 0.4,
            },
        }
    }
}

impl Default for ContextualStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ProviderSelectionStrategy for ContextualStrategy {
    async fn select_provider(
        &self,
        candidates: &[String],
        context: &ProviderContext,
        health_monitor: &HealthMonitor,
        cost_tracker: &CostTracker,
    ) -> Result<String> {
        let mut scored_providers: Vec<(String, f64)> = Vec::new();

        for provider_id in candidates {
            if !health_monitor.is_healthy(provider_id).await {
                continue;
            }

            let mut score = 0.0;

            // Health score
            score += self.health_weight;

            // Cost score - use actual cost tracking data
            let cost_score = cost_tracker
                .get_efficiency_score(provider_id)
                .unwrap_or(0.5);
            score += cost_score * context.cost_sensitivity * self.cost_weight;

            // Quality score
            let quality_score = self.calculate_quality_score(provider_id);
            score += quality_score * context.quality_requirement * self.quality_weight;

            // Latency score
            let latency_score = self.calculate_latency_score(provider_id);
            score += latency_score * context.latency_sensitivity * self.latency_weight;

            // Load compatibility score
            let load_score = self.calculate_load_score(provider_id, context.expected_load);
            score += load_score * self.load_weight;

            // Bonus for preferred providers
            if context.preferred_providers.contains(provider_id) {
                score += 0.1;
            }

            scored_providers.push((provider_id.clone(), score));
        }

        if scored_providers.is_empty() {
            return Err(Error::not_found("No suitable providers available"));
        }

        // Select highest scoring provider
        scored_providers
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(id, score)| {
                debug!("Selected provider {} with score {:.3}", id, score);
                id
            })
            .ok_or_else(|| Error::not_found("No providers available"))
    }
}

/// Dependencies required by the ProviderRouter
#[derive(Clone)]
pub struct ProviderRouterDeps {
    /// Provider registry
    pub registry: Arc<ProviderRegistry>,
    /// Health monitoring
    pub health_monitor: Arc<HealthMonitor>,
    /// Circuit breaker for resilience
    pub circuit_breaker: Arc<CircuitBreaker>,
    /// Metrics collection
    pub metrics: Arc<ProviderMetricsCollector>,
    /// Cost tracking
    pub cost_tracker: Arc<CostTracker>,
    /// Failover management
    pub failover_manager: Arc<FailoverManager>,
}

impl ProviderRouterDeps {
    /// Create dependencies with default implementations
    pub async fn with_defaults(registry: Arc<ProviderRegistry>) -> Result<Self> {
        let health_monitor = Arc::new(HealthMonitor::with_registry(Arc::clone(&registry)));
        let circuit_breaker =
            Arc::new(CircuitBreaker::with_config("global", CircuitBreakerConfig::default()).await);
        let metrics = Arc::new(ProviderMetricsCollector::new()?);
        let cost_tracker = Arc::new(CostTracker::new());
        let failover_manager = Arc::new(FailoverManager::new(Arc::clone(&health_monitor)));

        Ok(Self {
            registry,
            health_monitor,
            circuit_breaker,
            metrics,
            cost_tracker,
            failover_manager,
        })
    }
}

/// Advanced provider router with dependency injection
pub struct ProviderRouter {
    /// Injected dependencies
    deps: ProviderRouterDeps,
    /// Current selection strategy
    selection_strategy: Box<dyn ProviderSelectionStrategy>,
}

impl ProviderRouter {
    /// Create a new provider router with dependency injection
    pub fn new(deps: ProviderRouterDeps) -> Self {
        Self {
            deps,
            selection_strategy: Box::new(ContextualStrategy::new()),
        }
    }

    /// Create a new provider router with default dependencies
    pub async fn with_defaults(registry: Arc<ProviderRegistry>) -> Result<Self> {
        let deps = ProviderRouterDeps::with_defaults(registry).await?;
        Ok(Self::new(deps))
    }

    /// Set the provider selection strategy
    pub fn set_selection_strategy(&mut self, strategy: Box<dyn ProviderSelectionStrategy>) {
        self.selection_strategy = strategy;
        info!("Provider selection strategy updated");
    }

    /// Select an embedding provider based on context
    #[instrument(skip(self, context), fields(operation_type = %context.operation_type))]
    pub async fn select_embedding_provider(&self, context: &ProviderContext) -> Result<String> {
        let available_providers = self.deps.registry.list_embedding_providers();

        if available_providers.is_empty() {
            return Err(Error::not_found("No embedding providers available"));
        }

        // Filter out excluded providers
        let candidates: Vec<String> = available_providers
            .into_iter()
            .filter(|id| !context.excluded_providers.contains(id))
            .collect();

        if candidates.is_empty() {
            return Err(Error::not_found(
                "No eligible embedding providers available",
            ));
        }

        // Apply selection strategy
        let selected = self
            .selection_strategy
            .select_provider(
                &candidates,
                context,
                &self.deps.health_monitor,
                &self.deps.cost_tracker,
            )
            .await?;

        // Record selection metrics
        self.deps
            .metrics
            .record_provider_selection(&selected, "contextual");

        debug!(
            "Selected embedding provider: {} for operation: {}",
            selected, context.operation_type
        );

        Ok(selected)
    }

    /// Select a vector store provider based on context
    #[instrument(skip(self, context), fields(operation_type = %context.operation_type))]
    pub async fn select_vector_store_provider(&self, context: &ProviderContext) -> Result<String> {
        let available_providers = self.deps.registry.list_vector_store_providers();

        if available_providers.is_empty() {
            return Err(Error::not_found("No vector store providers available"));
        }

        // Filter out excluded providers
        let candidates: Vec<String> = available_providers
            .into_iter()
            .filter(|id| !context.excluded_providers.contains(id))
            .collect();

        if candidates.is_empty() {
            return Err(Error::not_found(
                "No eligible vector store providers available",
            ));
        }

        // Apply selection strategy
        let selected = self
            .selection_strategy
            .select_provider(
                &candidates,
                context,
                &self.deps.health_monitor,
                &self.deps.cost_tracker,
            )
            .await?;

        // Record selection metrics
        self.deps
            .metrics
            .record_provider_selection(&selected, "contextual");

        debug!(
            "Selected vector store provider: {} for operation: {}",
            selected, context.operation_type
        );

        Ok(selected)
    }

    /// Get an embedding provider with circuit breaker protection and failover
    pub async fn get_embedding_provider(
        &self,
        context: &ProviderContext,
    ) -> Result<Arc<dyn EmbeddingProvider>> {
        let candidates = self.deps.registry.list_embedding_providers();

        let failover_context = FailoverContext {
            operation_type: context.operation_type.clone(),
            preferred_providers: context.preferred_providers.clone(),
            excluded_providers: context.excluded_providers.clone(),
            max_attempts: 3,
            current_attempt: 0,
        };

        self.deps
            .failover_manager
            .execute_with_failover(&candidates, &failover_context, |provider_id: String| {
                let registry = Arc::clone(&self.deps.registry);
                let metrics = Arc::clone(&self.deps.metrics);

                async move {
                    let start_time = std::time::Instant::now();
                    let provider = registry.get_embedding_provider(&provider_id)?;

                    let response_time = start_time.elapsed().as_secs_f64();
                    metrics.record_response_time(&provider_id, "get_provider", response_time);
                    metrics.record_request(&provider_id, "get_provider", "success");

                    Ok(provider)
                }
            })
            .await
    }

    /// Get a vector store provider with circuit breaker protection and failover
    pub async fn get_vector_store_provider(
        &self,
        context: &ProviderContext,
    ) -> Result<Arc<dyn VectorStoreProvider>> {
        let candidates = self.deps.registry.list_vector_store_providers();

        let failover_context = FailoverContext {
            operation_type: context.operation_type.clone(),
            preferred_providers: context.preferred_providers.clone(),
            excluded_providers: context.excluded_providers.clone(),
            max_attempts: 3,
            current_attempt: 0,
        };

        self.deps
            .failover_manager
            .execute_with_failover(&candidates, &failover_context, |provider_id: String| {
                let registry = Arc::clone(&self.deps.registry);
                let metrics = Arc::clone(&self.deps.metrics);

                async move {
                    let start_time = std::time::Instant::now();
                    let provider = registry.get_vector_store_provider(&provider_id)?;

                    let response_time = start_time.elapsed().as_secs_f64();
                    metrics.record_response_time(&provider_id, "get_provider", response_time);
                    metrics.record_request(&provider_id, "get_provider", "success");

                    Ok(provider)
                }
            })
            .await
    }

    /// Record successful operation
    pub async fn record_success(&self, provider_id: &str, response_time_ms: f64) {
        self.deps
            .health_monitor
            .check_provider(provider_id)
            .await
            .ok();
        self.deps
            .metrics
            .record_response_time(provider_id, "operation", response_time_ms / 1000.0);
        self.deps
            .metrics
            .record_request(provider_id, "operation", "success");
    }

    /// Record failed operation
    pub async fn record_failure(&self, provider_id: &str, error: &Error) {
        self.deps
            .health_monitor
            .check_provider(provider_id)
            .await
            .ok();
        self.deps
            .metrics
            .record_request(provider_id, "operation", "error");
        self.deps
            .metrics
            .record_error(provider_id, &error.to_string());
    }

    /// Get router statistics
    pub async fn get_statistics(&self) -> RouterStatistics {
        let embedding_providers = self.deps.registry.list_embedding_providers().len();
        let vector_store_providers = self.deps.registry.list_vector_store_providers().len();
        let total_providers = embedding_providers + vector_store_providers;

        let _all_provider_ids: Vec<String> = self
            .deps
            .registry
            .list_embedding_providers()
            .into_iter()
            .chain(self.deps.registry.list_vector_store_providers())
            .collect();

        let healthy_providers = self
            .deps
            .health_monitor
            .list_healthy_providers()
            .await
            .len();

        let total_cost = self.deps.cost_tracker.get_total_cost();
        let current_period_cost = self.deps.cost_tracker.get_current_period_cost();

        RouterStatistics {
            total_providers,
            healthy_providers,
            total_cost,
            current_period_cost,
            strategy_name: "Contextual".to_string(),
        }
    }

    /// Get access to individual components for advanced usage
    pub fn deps(&self) -> &ProviderRouterDeps {
        &self.deps
    }
}

/// Router statistics
#[derive(Debug, Clone)]
pub struct RouterStatistics {
    /// Total number of registered providers
    pub total_providers: usize,
    /// Number of healthy providers
    pub healthy_providers: usize,
    /// Total cost across all providers
    pub total_cost: f64,
    /// Current billing period cost
    pub current_period_cost: f64,
    /// Current strategy name
    pub strategy_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::providers::routing::health::{HealthCheckResult, ProviderHealthStatus};
    use crate::infrastructure::di::registry::ProviderRegistry;

    #[tokio::test]
    async fn test_provider_router_creation() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let registry = Arc::new(ProviderRegistry::new());
        let router = ProviderRouter::with_defaults(Arc::clone(&registry)).await?;

        let stats = router.get_statistics().await;
        assert_eq!(stats.total_providers, 0);
        assert_eq!(stats.healthy_providers, 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_provider_selection_with_no_providers(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let registry = Arc::new(ProviderRegistry::new());
        let router = ProviderRouter::with_defaults(Arc::clone(&registry)).await?;

        let context = ProviderContext::default();
        let result = router.select_embedding_provider(&context).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_contextual_strategy() {
        let registry = Arc::new(ProviderRegistry::new());
        let strategy = ContextualStrategy::new();
        let health_monitor = Arc::new(HealthMonitor::with_registry(Arc::clone(&registry)));
        let cost_tracker = Arc::new(CostTracker::new());

        let candidates = vec!["ollama".to_string(), "openai".to_string()];

        // Register providers as healthy (unknown providers are now considered unhealthy)
        for provider in &candidates {
            health_monitor
                .record_result(HealthCheckResult {
                    provider_id: provider.clone(),
                    status: ProviderHealthStatus::Healthy,
                    response_time: std::time::Duration::from_millis(10),
                    error_message: None,
                })
                .await;
        }

        let context = ProviderContext {
            cost_sensitivity: 1.0, // High cost sensitivity
            ..Default::default()
        };

        // Providers are registered as healthy, selection will succeed
        let result = strategy
            .select_provider(&candidates, &context, &health_monitor, &cost_tracker)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_provider_context_default() {
        let context = ProviderContext::default();
        assert_eq!(context.operation_type, "general");
        assert_eq!(context.cost_sensitivity, 0.5);
        assert_eq!(context.quality_requirement, 0.5);
        assert!(context.excluded_providers.is_empty());
    }
}
