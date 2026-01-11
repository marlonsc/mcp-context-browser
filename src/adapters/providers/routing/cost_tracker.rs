//! Cost Tracking Module
//!
//! This module provides cost tracking capabilities using established patterns
//! and libraries, following SOLID principles with proper separation of concerns.

use crate::domain::error::{Error, Result};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Cost information for a provider operation
#[derive(Debug, Clone)]
pub struct ProviderCost {
    pub provider_id: String,
    pub operation_type: String,
    pub cost_per_unit: f64,
    pub unit_type: String, // "token", "request", "GB", etc.
    pub free_tier_limit: Option<u64>,
    pub currency: String,
}

impl ProviderCost {
    /// Calculate cost for given units
    pub fn calculate_cost(&self, units: u64) -> f64 {
        if let Some(free_limit) = self.free_tier_limit {
            if units <= free_limit {
                0.0
            } else {
                (units - free_limit) as f64 * self.cost_per_unit
            }
        } else {
            units as f64 * self.cost_per_unit
        }
    }

    /// Calculate efficiency score (0.0 = expensive, 1.0 = cheap)
    pub fn efficiency_score(&self) -> f64 {
        let max_reasonable_cost = match self.unit_type.as_str() {
            "token" => 0.0001, // $0.0001 per token (reasonable max)
            "request" => 1.0,  // $1 per request (reasonable max)
            "GB" => 1.0,       // $1 per GB (reasonable max)
            "second" => 0.1,   // $0.10 per second (reasonable max)
            _ => 1.0,          // Default
        };

        (max_reasonable_cost - self.cost_per_unit.min(max_reasonable_cost)) / max_reasonable_cost
    }
}

/// Usage metrics for tracking consumption
#[derive(Debug, Clone, Default)]
pub struct UsageMetrics {
    pub total_units: u64,
    pub total_cost: f64,
    pub current_period_units: u64,
    pub current_period_cost: f64,
    pub avg_cost_per_unit: f64,
    pub operation_count: u64,
    pub last_usage: Option<chrono::DateTime<chrono::Utc>>,
}

/// Cost tracking configuration
#[derive(Debug, Clone)]
pub struct CostTrackerConfig {
    pub enable_budget_limits: bool,
    pub default_currency: String,
}

impl Default for CostTrackerConfig {
    fn default() -> Self {
        Self {
            enable_budget_limits: true,
            default_currency: "USD".to_string(),
        }
    }
}

/// Trait for cost tracking
pub trait CostTrackerTrait: Send + Sync {
    fn record_usage(&self, provider_id: &str, units: u64) -> Result<f64>;
    fn get_usage_metrics(&self, provider_id: &str) -> Option<UsageMetrics>;
    fn set_budget(&self, provider_id: &str, budget: f64);
    fn check_budget(&self, provider_id: &str) -> bool;
    fn get_efficiency_score(&self, provider_id: &str) -> Option<f64>;
    fn register_provider_cost(&self, cost: ProviderCost);
    fn get_total_cost(&self) -> f64;
    fn get_current_period_cost(&self) -> f64;
}

/// Cost tracker for providers with thread-safe operations
pub struct CostTracker {
    /// Cost information for providers
    costs: Arc<DashMap<String, ProviderCost>>,
    /// Usage metrics for providers
    usage_metrics: Arc<DashMap<String, UsageMetrics>>,
    /// Budget limits for providers
    budgets: Arc<DashMap<String, f64>>,
    /// Configuration
    config: CostTrackerConfig,
}

impl CostTracker {
    /// Create a new cost tracker with default configuration
    pub fn new() -> Self {
        Self::with_config(CostTrackerConfig::default())
    }

    /// Create a new cost tracker with custom configuration
    pub fn with_config(config: CostTrackerConfig) -> Self {
        Self {
            costs: Arc::new(DashMap::new()),
            usage_metrics: Arc::new(DashMap::new()),
            budgets: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Get total cost across all providers
    pub fn get_total_cost(&self) -> f64 {
        self.usage_metrics.iter().map(|m| m.total_cost).sum()
    }

    /// Get current period cost across all providers
    pub fn get_current_period_cost(&self) -> f64 {
        self.usage_metrics
            .iter()
            .map(|m| m.current_period_cost)
            .sum()
    }
}

impl CostTrackerTrait for CostTracker {
    /// Register cost information for a provider
    fn register_provider_cost(&self, cost: ProviderCost) {
        info!(
            "Registered cost for provider {}: {} per {}",
            cost.provider_id, cost.cost_per_unit, cost.unit_type
        );
        self.costs.insert(cost.provider_id.clone(), cost);
    }

    /// Set monthly budget for a provider
    fn set_budget(&self, provider_id: &str, budget: f64) {
        info!("Set budget for provider {}: {}", provider_id, budget);
        self.budgets.insert(provider_id.to_string(), budget);
    }

    /// Record usage and calculate cost
    fn record_usage(&self, provider_id: &str, units: u64) -> Result<f64> {
        let cost_info = self
            .costs
            .get(provider_id)
            .ok_or_else(|| Error::not_found(format!("Cost info not found for {}", provider_id)))?;

        let cost = cost_info.calculate_cost(units);

        let mut metrics = self
            .usage_metrics
            .entry(provider_id.to_string())
            .or_default();
        metrics.total_units += units;
        metrics.total_cost += cost;
        metrics.current_period_units += units;
        metrics.current_period_cost += cost;
        metrics.operation_count += 1;
        metrics.last_usage = Some(chrono::Utc::now());

        if metrics.total_units > 0 {
            metrics.avg_cost_per_unit = metrics.total_cost / metrics.total_units as f64;
        }

        debug!(
            "Recorded {} units for provider {}, cost: {}",
            units, provider_id, cost
        );

        // Check budget
        if self.config.enable_budget_limits {
            if let Some(budget) = self.budgets.get(provider_id) {
                if metrics.current_period_cost > *budget {
                    warn!(
                        "Budget exceeded for provider {}: current={}, limit={}",
                        provider_id, metrics.current_period_cost, *budget
                    );
                }
            }
        }

        Ok(cost)
    }

    /// Check if provider is within budget
    fn check_budget(&self, provider_id: &str) -> bool {
        if !self.config.enable_budget_limits {
            return true;
        }

        let budget = match self.budgets.get(provider_id) {
            Some(b) => *b,
            None => return true, // No budget limit
        };

        let metrics = match self.usage_metrics.get(provider_id) {
            Some(m) => m.clone(),
            None => return true, // No usage yet
        };

        metrics.current_period_cost <= budget
    }

    /// Get usage metrics for a provider
    fn get_usage_metrics(&self, provider_id: &str) -> Option<UsageMetrics> {
        self.usage_metrics.get(provider_id).map(|m| m.clone())
    }

    /// Get efficiency score for a provider (0.0 = expensive, 1.0 = cheap)
    fn get_efficiency_score(&self, provider_id: &str) -> Option<f64> {
        self.costs.get(provider_id).map(|c| c.efficiency_score())
    }

    fn get_total_cost(&self) -> f64 {
        self.get_total_cost()
    }

    fn get_current_period_cost(&self) -> f64 {
        self.get_current_period_cost()
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculation() {
        let cost_info = ProviderCost {
            provider_id: "test".to_string(),
            operation_type: "embedding".to_string(),
            cost_per_unit: 0.01,
            unit_type: "token".to_string(),
            free_tier_limit: Some(100),
            currency: "USD".to_string(),
        };

        assert_eq!(cost_info.calculate_cost(50), 0.0);
        assert_eq!(cost_info.calculate_cost(150), 0.5);
    }

    #[test]
    fn test_cost_tracking() {
        let tracker = CostTracker::new();
        let cost_info = ProviderCost {
            provider_id: "test".to_string(),
            operation_type: "embedding".to_string(),
            cost_per_unit: 0.01,
            unit_type: "token".to_string(),
            free_tier_limit: None,
            currency: "USD".to_string(),
        };

        tracker.register_provider_cost(cost_info);
        let cost = tracker.record_usage("test", 100).unwrap();
        assert_eq!(cost, 1.0);

        let metrics = tracker.get_usage_metrics("test").unwrap();
        assert_eq!(metrics.total_units, 100);
        assert_eq!(metrics.total_cost, 1.0);
    }

    #[test]
    fn test_budget_enforcement() {
        let tracker = CostTracker::new();
        tracker.set_budget("test", 5.0);

        let cost_info = ProviderCost {
            provider_id: "test".to_string(),
            operation_type: "embedding".to_string(),
            cost_per_unit: 1.0,
            unit_type: "request".to_string(),
            free_tier_limit: None,
            currency: "USD".to_string(),
        };
        tracker.register_provider_cost(cost_info);

        assert!(tracker.check_budget("test"));
        let _ = tracker.record_usage("test", 4);
        assert!(tracker.check_budget("test"));
        let _ = tracker.record_usage("test", 2);
        assert!(!tracker.check_budget("test"));
    }

    #[test]
    fn test_cost_efficiency_ranking() {
        let expensive = ProviderCost {
            provider_id: "exp".to_string(),
            operation_type: "token".to_string(),
            cost_per_unit: 0.0001,
            unit_type: "token".to_string(),
            free_tier_limit: None,
            currency: "USD".to_string(),
        };

        let cheap = ProviderCost {
            provider_id: "cheap".to_string(),
            operation_type: "token".to_string(),
            cost_per_unit: 0.00001,
            unit_type: "token".to_string(),
            free_tier_limit: None,
            currency: "USD".to_string(),
        };

        assert!(cheap.efficiency_score() > expensive.efficiency_score());
    }

    #[test]
    fn test_budget_utilization() {
        let tracker = CostTracker::new();
        tracker.set_budget("test", 10.0);

        let cost_info = ProviderCost {
            provider_id: "test".to_string(),
            operation_type: "request".to_string(),
            cost_per_unit: 1.0,
            unit_type: "request".to_string(),
            free_tier_limit: None,
            currency: "USD".to_string(),
        };
        tracker.register_provider_cost(cost_info);

        let _ = tracker.record_usage("test", 7);
        let score = tracker.get_efficiency_score("test").unwrap();
        assert!(score >= 0.0);
    }
}
