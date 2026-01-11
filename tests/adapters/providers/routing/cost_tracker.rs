//! Tests for cost tracking module
//!
//! Tests for cost tracking capabilities with thread-safe operations.

use mcp_context_browser::adapters::providers::routing::cost_tracker::{
    CostTracker, CostTrackerTrait, ProviderCost,
};

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
fn test_cost_tracking() -> std::result::Result<(), Box<dyn std::error::Error>> {
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
    let cost = tracker.record_usage("test", 100)?;
    assert_eq!(cost, 1.0);

    let metrics = tracker
        .get_usage_metrics("test")
        .ok_or("Usage metrics not found")?;
    assert_eq!(metrics.total_units, 100);
    assert_eq!(metrics.total_cost, 1.0);
    Ok(())
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
fn test_budget_utilization() -> std::result::Result<(), Box<dyn std::error::Error>> {
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
    let score = tracker
        .get_efficiency_score("test")
        .ok_or("Efficiency score not found")?;
    assert!(score >= 0.0);
    Ok(())
}
