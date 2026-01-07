//! Cost Tracking Module
//!
//! This module provides cost tracking capabilities using established patterns
//! and libraries, following SOLID principles with proper separation of concerns.

use crate::core::error::{Error, Result};
use dashmap::DashMap;
use std::collections::HashMap;
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

    /// Register cost information for a provider
    pub fn register_provider_cost(&self, cost: ProviderCost) {
        let key = format!("{}:{}", cost.provider_id, cost.operation_type);
        self.costs.insert(key, cost.clone());
        info!(
            "Registered cost for provider {} operation {}",
            cost.provider_id, cost.operation_type
        );
    }

    /// Set budget limit for a provider
    pub fn set_budget(&self, provider_id: &str, budget: f64) {
        self.budgets.insert(provider_id.to_string(), budget);
        info!(
            "Set budget limit of {} for provider {}",
            budget, provider_id
        );
    }

    /// Track operation usage and cost
    pub fn track_operation(
        &self,
        provider_id: &str,
        operation_type: &str,
        units: u64,
    ) -> Result<f64> {
        let cost_key = format!("{}:{}", provider_id, operation_type);
        let cost_info = self.costs.get(&cost_key).ok_or_else(|| {
            Error::not_found(format!(
                "Cost info for provider: {} operation: {}",
                provider_id, operation_type
            ))
        })?;

        let cost = cost_info.calculate_cost(units);

        // Check budget limits if enabled
        if self.config.enable_budget_limits {
            if let Some(budget_limit) = self.budgets.get(provider_id) {
                let current_metrics = self
                    .usage_metrics
                    .get(provider_id)
                    .map(|m| m.clone())
                    .unwrap_or_default();
                let new_total_cost = current_metrics.current_period_cost + cost;

                if new_total_cost > *budget_limit {
                    warn!(
                        "Budget limit exceeded for provider: {} (current: {}, budget: {})",
                        provider_id, new_total_cost, *budget_limit
                    );
                    return Err(Error::generic(format!(
                        "Budget limit exceeded for provider: {} (limit: {}, would be: {})",
                        provider_id, *budget_limit, new_total_cost
                    )));
                }
            }
        }

        // Update usage metrics
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

        // Update average cost
        if metrics.total_units > 0 {
            metrics.avg_cost_per_unit = metrics.total_cost / metrics.total_units as f64;
        }

        debug!(
            "Tracked operation: {}:{} units={} cost={}",
            provider_id, operation_type, units, cost
        );

        Ok(cost)
    }

    /// Get usage metrics for a provider
    pub fn get_usage_metrics(&self, provider_id: &str) -> Option<UsageMetrics> {
        self.usage_metrics.get(provider_id).map(|m| m.clone())
    }

    /// Get cost information for a provider operation
    pub fn get_provider_cost(
        &self,
        provider_id: &str,
        operation_type: &str,
    ) -> Option<ProviderCost> {
        let key = format!("{}:{}", provider_id, operation_type);
        self.costs.get(&key).map(|c| c.clone())
    }

    /// Get total cost across all providers
    pub fn get_total_cost(&self) -> f64 {
        self.usage_metrics.iter().map(|m| m.total_cost).sum()
    }

    /// Get current billing period cost across all providers
    pub fn get_current_period_cost(&self) -> f64 {
        self.usage_metrics
            .iter()
            .map(|m| m.current_period_cost)
            .sum()
    }

    /// Reset billing period for all providers
    pub fn reset_billing_period(&self) {
        self.usage_metrics.iter_mut().for_each(|mut metrics| {
            metrics.current_period_units = 0;
            metrics.current_period_cost = 0.0;
        });
        info!("Reset billing period for all providers");
    }

    /// Get cost efficiency ranking of providers
    pub fn get_cost_efficiency_ranking(&self) -> Vec<(String, f64)> {
        let mut rankings: Vec<(String, f64)> = self
            .costs
            .iter()
            .map(|entry| {
                let provider_id = entry.key().split(':').next().unwrap_or("").to_string();
                let efficiency = entry.value().efficiency_score();
                (provider_id, efficiency)
            })
            .collect();

        // Remove duplicates and sort by efficiency (higher is better)
        rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        rankings.dedup_by(|a, b| a.0 == b.0);

        rankings
    }

    /// Get budget utilization for all providers
    pub fn get_budget_utilization(&self) -> HashMap<String, f64> {
        let mut utilization = HashMap::new();

        for budget_entry in self.budgets.iter() {
            let provider_id = budget_entry.key();
            let budget_limit = *budget_entry.value();

            if let Some(metrics) = self.usage_metrics.get(provider_id) {
                let utilization_rate = if budget_limit > 0.0 {
                    metrics.current_period_cost / budget_limit
                } else {
                    0.0
                };
                utilization.insert(provider_id.clone(), utilization_rate);
            } else {
                utilization.insert(provider_id.clone(), 0.0);
            }
        }

        utilization
    }

    /// Get efficiency score for a provider (0.0 = expensive, 1.0 = cheap)
    /// Higher scores indicate more cost-effective providers
    pub fn get_efficiency_score(&self, provider_id: &str) -> Option<f64> {
        // Find the provider with the best cost efficiency
        let mut best_cost = f64::INFINITY;
        let mut provider_cost = None;

        for cost_entry in self.costs.iter() {
            if cost_entry.provider_id == provider_id && cost_entry.cost_per_unit < best_cost {
                best_cost = cost_entry.cost_per_unit;
                provider_cost = Some(cost_entry.value().clone());
            }
        }

        provider_cost.map(|cost| cost.efficiency_score())
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
        let cost = ProviderCost {
            provider_id: "test-provider".to_string(),
            operation_type: "embedding".to_string(),
            cost_per_unit: 0.0001,
            unit_type: "token".to_string(),
            free_tier_limit: Some(1000000),
            currency: "USD".to_string(),
        };

        // Within free tier
        assert_eq!(cost.calculate_cost(500000), 0.0);

        // Exceeding free tier
        assert_eq!(cost.calculate_cost(1500000), 50.0); // 500k * $0.0001

        // No free tier
        let cost_no_free = ProviderCost {
            free_tier_limit: None,
            ..cost
        };
        assert_eq!(cost_no_free.calculate_cost(1000000), 100.0);
    }

    #[test]
    fn test_cost_tracking() {
        let tracker = CostTracker::new();

        let cost = ProviderCost {
            provider_id: "test-provider".to_string(),
            operation_type: "embedding".to_string(),
            cost_per_unit: 0.0001,
            unit_type: "token".to_string(),
            free_tier_limit: None,
            currency: "USD".to_string(),
        };

        tracker.register_provider_cost(cost);

        // Track some usage
        let cost1 = tracker
            .track_operation("test-provider", "embedding", 1000)
            .unwrap();
        assert_eq!(cost1, 0.1); // 1000 * 0.0001

        let cost2 = tracker
            .track_operation("test-provider", "embedding", 2000)
            .unwrap();
        assert_eq!(cost2, 0.2);

        // Check metrics
        if let Some(metrics) = tracker.get_usage_metrics("test-provider") {
            assert_eq!(metrics.total_units, 3000);
            assert!((metrics.total_cost - 0.3).abs() < f64::EPSILON);
            assert_eq!(metrics.operation_count, 2);
        } else {
            panic!("Metrics not found");
        }
    }

    #[test]
    fn test_budget_enforcement() {
        let tracker = CostTracker::new();

        let cost = ProviderCost {
            provider_id: "test-provider".to_string(),
            operation_type: "embedding".to_string(),
            cost_per_unit: 0.01, // High cost to trigger budget limits quickly
            unit_type: "request".to_string(),
            free_tier_limit: None,
            currency: "USD".to_string(),
        };

        let budget = 1.0; // $1 budget

        tracker.register_provider_cost(cost);
        tracker.set_budget("test-provider", budget);

        // First operation should succeed
        let result1 = tracker.track_operation("test-provider", "embedding", 50);
        assert!(result1.is_ok()); // $0.50

        // Second operation should exceed budget
        let result2 = tracker.track_operation("test-provider", "embedding", 60);
        assert!(result2.is_err()); // Would be $0.60, total $1.10 > $1.00
    }

    #[test]
    fn test_cost_efficiency_ranking() {
        let tracker = CostTracker::new();

        // Add expensive provider
        let expensive_cost = ProviderCost {
            provider_id: "expensive".to_string(),
            operation_type: "embedding".to_string(),
            cost_per_unit: 0.001,
            unit_type: "token".to_string(),
            free_tier_limit: None,
            currency: "USD".to_string(),
        };

        // Add cheap provider
        let cheap_cost = ProviderCost {
            provider_id: "cheap".to_string(),
            operation_type: "embedding".to_string(),
            cost_per_unit: 0.00001,
            unit_type: "token".to_string(),
            free_tier_limit: None,
            currency: "USD".to_string(),
        };

        tracker.register_provider_cost(expensive_cost);
        tracker.register_provider_cost(cheap_cost);

        let ranking = tracker.get_cost_efficiency_ranking();
        assert!(!ranking.is_empty());

        // Cheap provider should be ranked higher (higher efficiency score)
        let cheap_index = ranking.iter().position(|(id, _)| id == "cheap");
        let expensive_index = ranking.iter().position(|(id, _)| id == "expensive");

        assert!(cheap_index.is_some());
        assert!(expensive_index.is_some());

        if let (Some(cheap_pos), Some(expensive_pos)) = (cheap_index, expensive_index) {
            assert!(
                cheap_pos < expensive_pos,
                "Cheap provider should rank higher than expensive provider"
            );
        }
    }

    #[test]
    fn test_budget_utilization() {
        let tracker = CostTracker::new();

        tracker.set_budget("provider1", 100.0);
        tracker.set_budget("provider2", 200.0);

        let utilization = tracker.get_budget_utilization();
        assert_eq!(utilization.get("provider1"), Some(&0.0));
        assert_eq!(utilization.get("provider2"), Some(&0.0));
    }
}
