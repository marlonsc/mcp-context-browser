//! Provider Routing Port
//!
//! Defines the contract for provider routing and selection services.
//! Provider routing enables intelligent selection of embedding providers,
//! vector stores, and other backend services based on health, cost,
//! and quality requirements.

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health status for a provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProviderHealthStatus {
    /// Provider is functioning normally
    #[default]
    Healthy,
    /// Provider is experiencing issues but still usable
    Degraded,
    /// Provider is not available
    Unhealthy,
}

/// Context for provider selection decisions
///
/// This structure carries information about the operation being performed
/// and any preferences or constraints that should influence provider selection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderContext {
    /// Type of operation being performed (e.g., "embedding", "search", "index")
    pub operation_type: String,
    /// Cost sensitivity (0.0 = ignore cost, 1.0 = prioritize low cost)
    pub cost_sensitivity: f64,
    /// Quality requirement (0.0 = any quality, 1.0 = highest quality only)
    pub quality_requirement: f64,
    /// Latency sensitivity (0.0 = ignore latency, 1.0 = prioritize low latency)
    pub latency_sensitivity: f64,
    /// Preferred providers to try first (if healthy)
    pub preferred_providers: Vec<String>,
    /// Providers to exclude from selection
    pub excluded_providers: Vec<String>,
}

impl ProviderContext {
    /// Create a new provider context with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a context optimized for low cost
    pub fn cost_optimized() -> Self {
        Self {
            cost_sensitivity: 1.0,
            quality_requirement: 0.3,
            latency_sensitivity: 0.3,
            ..Default::default()
        }
    }

    /// Create a context optimized for high quality
    pub fn quality_optimized() -> Self {
        Self {
            cost_sensitivity: 0.3,
            quality_requirement: 1.0,
            latency_sensitivity: 0.3,
            ..Default::default()
        }
    }

    /// Create a context optimized for low latency
    pub fn latency_optimized() -> Self {
        Self {
            cost_sensitivity: 0.3,
            quality_requirement: 0.3,
            latency_sensitivity: 1.0,
            ..Default::default()
        }
    }

    /// Set the operation type
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation_type = operation.into();
        self
    }

    /// Add a preferred provider
    pub fn prefer(mut self, provider: impl Into<String>) -> Self {
        self.preferred_providers.push(provider.into());
        self
    }

    /// Exclude a provider
    pub fn exclude(mut self, provider: impl Into<String>) -> Self {
        self.excluded_providers.push(provider.into());
        self
    }
}

/// Provider routing interface
///
/// Provides intelligent routing and selection of backend providers
/// based on health status, cost, quality, and operational requirements.
#[async_trait]
pub trait ProviderRouter: Send + Sync {
    /// Select the best embedding provider based on context
    ///
    /// Returns the identifier of the selected provider.
    async fn select_embedding_provider(&self, context: &ProviderContext) -> Result<String>;

    /// Select the best vector store provider based on context
    ///
    /// Returns the identifier of the selected provider.
    async fn select_vector_store_provider(&self, context: &ProviderContext) -> Result<String>;

    /// Get the current health status of a provider
    async fn get_provider_health(&self, provider_id: &str) -> Result<ProviderHealthStatus>;

    /// Report a provider failure for health tracking
    ///
    /// This should be called when a provider operation fails to update
    /// the health monitoring system.
    async fn report_failure(&self, provider_id: &str, error: &str) -> Result<()>;

    /// Report a provider success for health tracking
    ///
    /// This should be called when a provider operation succeeds to update
    /// the health monitoring system.
    async fn report_success(&self, provider_id: &str) -> Result<()>;

    /// Get health status of all known providers
    async fn get_all_health(&self) -> Result<HashMap<String, ProviderHealthStatus>>;

    /// Get router statistics for monitoring
    async fn get_stats(&self) -> HashMap<String, serde_json::Value>;
}
