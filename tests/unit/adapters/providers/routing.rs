//! Routing provider tests

#[path = "routing/circuit_breaker.rs"]
mod circuit_breaker;

#[path = "routing/cost_tracker.rs"]
mod cost_tracker;

#[path = "routing/failover.rs"]
mod failover;

#[path = "routing/health.rs"]
mod health;

#[path = "routing/metrics.rs"]
mod metrics;

#[path = "routing/router.rs"]
mod router;
