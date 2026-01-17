//! Provider Router Implementations
//!
//! Provides routing logic for selecting providers based on health and context.

use async_trait::async_trait;
use mcb_application::ports::infrastructure::routing::{
    ProviderContext, ProviderHealthStatus, ProviderRouter,
};
use mcb_domain::error::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;

use super::health::HealthMonitor;

/// Default provider router with health-aware selection
///
/// Selects providers based on health status and context preferences.
/// Must be constructed via DI - never instantiate directly.
pub struct DefaultProviderRouter {
    /// Health monitor for tracking provider status
    health_monitor: Arc<dyn HealthMonitor>,
    /// Available embedding providers
    embedding_providers: Vec<String>,
    /// Available vector store providers
    vector_store_providers: Vec<String>,
}

impl DefaultProviderRouter {
    /// Create a new router with injected dependencies
    ///
    /// This should only be called by the DI container.
    pub fn new(
        health_monitor: Arc<dyn HealthMonitor>,
        embedding_providers: Vec<String>,
        vector_store_providers: Vec<String>,
    ) -> Self {
        Self {
            health_monitor,
            embedding_providers,
            vector_store_providers,
        }
    }

    /// Select the best provider from a list based on health and preferences
    fn select_best_provider(
        &self,
        providers: &[String],
        context: &ProviderContext,
    ) -> Result<String> {
        // Filter out excluded providers
        let available: Vec<_> = providers
            .iter()
            .filter(|p| !context.excluded_providers.contains(p))
            .collect();

        if available.is_empty() {
            return Err(Error::infrastructure("No providers available after exclusions"));
        }

        // Try preferred providers first (if healthy)
        for preferred in &context.preferred_providers {
            if available.contains(&preferred) {
                let health = self.health_monitor.get_health(preferred);
                if health != ProviderHealthStatus::Unhealthy {
                    return Ok(preferred.clone());
                }
            }
        }

        // Find the healthiest available provider
        let mut best_provider: Option<&String> = None;
        let mut best_health = ProviderHealthStatus::Unhealthy;

        for provider in &available {
            let health = self.health_monitor.get_health(provider);

            // Prefer healthy over degraded over unhealthy
            let is_better = match (health, best_health) {
                (ProviderHealthStatus::Healthy, _) => best_health != ProviderHealthStatus::Healthy,
                (ProviderHealthStatus::Degraded, ProviderHealthStatus::Unhealthy) => true,
                (ProviderHealthStatus::Degraded, _) => best_provider.is_none(),
                (ProviderHealthStatus::Unhealthy, _) => best_provider.is_none(),
            };

            if is_better {
                best_provider = Some(provider);
                best_health = health;
            }
        }

        best_provider
            .cloned()
            .ok_or_else(|| Error::infrastructure("No healthy providers available"))
    }
}

#[async_trait]
impl ProviderRouter for DefaultProviderRouter {
    async fn select_embedding_provider(&self, context: &ProviderContext) -> Result<String> {
        self.select_best_provider(&self.embedding_providers, context)
    }

    async fn select_vector_store_provider(&self, context: &ProviderContext) -> Result<String> {
        self.select_best_provider(&self.vector_store_providers, context)
    }

    async fn get_provider_health(&self, provider_id: &str) -> Result<ProviderHealthStatus> {
        Ok(self.health_monitor.get_health(provider_id))
    }

    async fn report_failure(&self, provider_id: &str, _error: &str) -> Result<()> {
        self.health_monitor.record_failure(provider_id);
        Ok(())
    }

    async fn report_success(&self, provider_id: &str) -> Result<()> {
        self.health_monitor.record_success(provider_id);
        Ok(())
    }

    async fn get_all_health(&self) -> Result<HashMap<String, ProviderHealthStatus>> {
        Ok(self.health_monitor.get_all_health())
    }

    async fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        stats.insert(
            "provider".to_string(),
            serde_json::json!("default"),
        );
        stats.insert(
            "embedding_providers".to_string(),
            serde_json::json!(self.embedding_providers),
        );
        stats.insert(
            "vector_store_providers".to_string(),
            serde_json::json!(self.vector_store_providers),
        );
        stats.insert(
            "health_summary".to_string(),
            serde_json::json!(self.health_monitor.get_all_health()),
        );
        stats
    }
}

impl std::fmt::Debug for DefaultProviderRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultProviderRouter")
            .field("embedding_providers", &self.embedding_providers)
            .field("vector_store_providers", &self.vector_store_providers)
            .finish()
    }
}

/// Null provider router for testing
///
/// Always returns "null" as the selected provider.
/// Does not track health or apply any routing logic.
pub struct NullProviderRouter;

impl NullProviderRouter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullProviderRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProviderRouter for NullProviderRouter {
    async fn select_embedding_provider(&self, _context: &ProviderContext) -> Result<String> {
        Ok("null".to_string())
    }

    async fn select_vector_store_provider(&self, _context: &ProviderContext) -> Result<String> {
        Ok("null".to_string())
    }

    async fn get_provider_health(&self, _provider_id: &str) -> Result<ProviderHealthStatus> {
        Ok(ProviderHealthStatus::Healthy)
    }

    async fn report_failure(&self, _provider_id: &str, _error: &str) -> Result<()> {
        Ok(())
    }

    async fn report_success(&self, _provider_id: &str) -> Result<()> {
        Ok(())
    }

    async fn get_all_health(&self) -> Result<HashMap<String, ProviderHealthStatus>> {
        Ok(HashMap::new())
    }

    async fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        stats.insert("provider".to_string(), serde_json::json!("null"));
        stats
    }
}

// Shaku Component implementation for NullProviderRouter
impl<M: shaku::Module> shaku::Component<M> for NullProviderRouter {
    type Interface = dyn ProviderRouter;
    type Parameters = ();

    fn build(_: &mut shaku::ModuleBuildContext<M>, _: Self::Parameters) -> Box<Self::Interface> {
        Box::new(NullProviderRouter::new())
    }
}
