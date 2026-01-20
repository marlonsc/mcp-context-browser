//! Unit tests for service lifecycle management
//!
//! Tests service state tracking, lifecycle operations, and the LifecycleManaged trait.
//! Includes a test implementation to verify trait behavior.

use async_trait::async_trait;
use mcb_application::ports::admin::{
    DependencyHealth, DependencyHealthCheck, LifecycleManaged, PortServiceState,
};
use mcb_domain::error::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

// =============================================================================
// PortServiceState Enum Tests
// =============================================================================

/// Test service state enum values
#[test]
fn test_service_state_values() {
    // Verify all states are distinct
    assert_ne!(PortServiceState::Starting, PortServiceState::Running);
    assert_ne!(PortServiceState::Running, PortServiceState::Stopping);
    assert_ne!(PortServiceState::Stopping, PortServiceState::Stopped);
    assert_ne!(PortServiceState::Stopped, PortServiceState::Starting);
}

/// Test service state debug representation
#[test]
fn test_service_state_debug() {
    let state = PortServiceState::Running;
    let debug_str = format!("{:?}", state);
    assert!(debug_str.contains("Running"));
}

/// Test service state clone (Copy)
#[test]
fn test_service_state_clone() {
    let state = PortServiceState::Running;
    let cloned = state;
    assert_eq!(state, cloned);
}

/// Test all service states can be created
#[test]
fn test_all_service_states_creation() {
    let states = [
        PortServiceState::Starting,
        PortServiceState::Running,
        PortServiceState::Stopping,
        PortServiceState::Stopped,
    ];

    assert_eq!(states.len(), 4);

    // Verify each state is distinct
    for (i, state_a) in states.iter().enumerate() {
        for (j, state_b) in states.iter().enumerate() {
            if i != j {
                assert_ne!(state_a, state_b, "States at {} and {} should differ", i, j);
            }
        }
    }
}

/// Test default state is Stopped
#[test]
fn test_service_state_default() {
    let state: PortServiceState = Default::default();
    assert_eq!(state, PortServiceState::Stopped);
}

/// Test service state serialization
#[test]
fn test_service_state_serialization() {
    let state = PortServiceState::Running;
    let json = serde_json::to_string(&state).expect("serialization failed");
    assert!(json.contains("Running"));

    let deserialized: PortServiceState =
        serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deserialized, PortServiceState::Running);
}

// =============================================================================
// DependencyHealth Tests
// =============================================================================

/// Test DependencyHealth enum values
#[test]
fn test_dependency_health_values() {
    assert_ne!(DependencyHealth::Healthy, DependencyHealth::Degraded);
    assert_ne!(DependencyHealth::Degraded, DependencyHealth::Unhealthy);
    assert_ne!(DependencyHealth::Unhealthy, DependencyHealth::Unknown);
}

/// Test DependencyHealth default is Unknown
#[test]
fn test_dependency_health_default() {
    let health: DependencyHealth = Default::default();
    assert_eq!(health, DependencyHealth::Unknown);
}

/// Test DependencyHealth serialization
#[test]
fn test_dependency_health_serialization() {
    let health = DependencyHealth::Healthy;
    let json = serde_json::to_string(&health).expect("serialization failed");
    let deserialized: DependencyHealth =
        serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deserialized, DependencyHealth::Healthy);
}

// =============================================================================
// DependencyHealthCheck Tests
// =============================================================================

/// Test DependencyHealthCheck default values
#[test]
fn test_health_check_default() {
    let check: DependencyHealthCheck = Default::default();
    assert_eq!(check.name, "");
    assert_eq!(check.status, DependencyHealth::Unknown);
    assert!(check.message.is_none());
    assert!(check.latency_ms.is_none());
    assert_eq!(check.last_check, 0);
}

/// Test DependencyHealthCheck with values
#[test]
fn test_health_check_with_values() {
    let check = DependencyHealthCheck {
        name: "database".to_string(),
        status: DependencyHealth::Healthy,
        message: Some("Connected".to_string()),
        latency_ms: Some(15),
        last_check: 1234567890,
    };

    assert_eq!(check.name, "database");
    assert_eq!(check.status, DependencyHealth::Healthy);
    assert_eq!(check.message.as_deref(), Some("Connected"));
    assert_eq!(check.latency_ms, Some(15));
}

/// Test DependencyHealthCheck serialization
#[test]
fn test_health_check_serialization() {
    let check = DependencyHealthCheck {
        name: "redis".to_string(),
        status: DependencyHealth::Degraded,
        message: Some("High latency".to_string()),
        latency_ms: Some(500),
        last_check: 1234567890,
    };

    let json = serde_json::to_string(&check).expect("serialization failed");
    assert!(json.contains("redis"));
    assert!(json.contains("Degraded"));
    assert!(json.contains("High latency"));

    let deserialized: DependencyHealthCheck =
        serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deserialized.name, "redis");
    assert_eq!(deserialized.status, DependencyHealth::Degraded);
}

// =============================================================================
// Test Implementation of LifecycleManaged
// =============================================================================

/// Test implementation of LifecycleManaged for verifying trait behavior
struct TestService {
    name: String,
    state: AtomicU32,
    start_count: AtomicU32,
    stop_count: AtomicU32,
    healthy: bool,
}

impl TestService {
    fn new(name: &str, healthy: bool) -> Self {
        Self {
            name: name.to_string(),
            state: AtomicU32::new(Self::state_to_u32(PortServiceState::Stopped)),
            start_count: AtomicU32::new(0),
            stop_count: AtomicU32::new(0),
            healthy,
        }
    }

    fn state_to_u32(state: PortServiceState) -> u32 {
        match state {
            PortServiceState::Starting => 0,
            PortServiceState::Running => 1,
            PortServiceState::Stopping => 2,
            PortServiceState::Stopped => 3,
        }
    }

    fn u32_to_state(value: u32) -> PortServiceState {
        match value {
            0 => PortServiceState::Starting,
            1 => PortServiceState::Running,
            2 => PortServiceState::Stopping,
            3 => PortServiceState::Stopped,
            _ => PortServiceState::Stopped,
        }
    }
}

#[async_trait]
impl LifecycleManaged for TestService {
    fn name(&self) -> &str {
        &self.name
    }

    fn state(&self) -> PortServiceState {
        Self::u32_to_state(self.state.load(Ordering::SeqCst))
    }

    async fn start(&self) -> Result<()> {
        self.state.store(
            Self::state_to_u32(PortServiceState::Starting),
            Ordering::SeqCst,
        );
        self.start_count.fetch_add(1, Ordering::SeqCst);
        // Simulate startup
        self.state.store(
            Self::state_to_u32(PortServiceState::Running),
            Ordering::SeqCst,
        );
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        self.state.store(
            Self::state_to_u32(PortServiceState::Stopping),
            Ordering::SeqCst,
        );
        self.stop_count.fetch_add(1, Ordering::SeqCst);
        // Simulate shutdown
        self.state.store(
            Self::state_to_u32(PortServiceState::Stopped),
            Ordering::SeqCst,
        );
        Ok(())
    }

    async fn health_check(&self) -> DependencyHealthCheck {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if self.healthy {
            DependencyHealthCheck {
                name: self.name.clone(),
                status: DependencyHealth::Healthy,
                message: Some("Service is healthy".to_string()),
                latency_ms: Some(5),
                last_check: now,
            }
        } else {
            DependencyHealthCheck {
                name: self.name.clone(),
                status: DependencyHealth::Unhealthy,
                message: Some("Service check failed".to_string()),
                latency_ms: None,
                last_check: now,
            }
        }
    }
}

// =============================================================================
// LifecycleManaged Trait Tests
// =============================================================================

/// Test service starts in Stopped state
#[tokio::test]
async fn test_service_initial_state() {
    let service = TestService::new("test-service", true);
    assert_eq!(service.state(), PortServiceState::Stopped);
}

/// Test service name retrieval
#[tokio::test]
async fn test_service_name() {
    let service = TestService::new("my-awesome-service", true);
    assert_eq!(service.name(), "my-awesome-service");
}

/// Test service start transitions to Running
#[tokio::test]
async fn test_service_start() {
    let service = TestService::new("test-service", true);
    assert_eq!(service.state(), PortServiceState::Stopped);

    service.start().await.expect("start should succeed");
    assert_eq!(service.state(), PortServiceState::Running);
}

/// Test service stop transitions to Stopped
#[tokio::test]
async fn test_service_stop() {
    let service = TestService::new("test-service", true);

    // Start first
    service.start().await.expect("start should succeed");
    assert_eq!(service.state(), PortServiceState::Running);

    // Then stop
    service.stop().await.expect("stop should succeed");
    assert_eq!(service.state(), PortServiceState::Stopped);
}

/// Test service restart calls stop then start
#[tokio::test]
async fn test_service_restart() {
    let service = TestService::new("test-service", true);

    // Start initially
    service.start().await.expect("start should succeed");
    let initial_starts = service.start_count.load(Ordering::SeqCst);
    let initial_stops = service.stop_count.load(Ordering::SeqCst);

    // Restart
    service.restart().await.expect("restart should succeed");

    // Should have incremented both counters
    assert_eq!(
        service.start_count.load(Ordering::SeqCst),
        initial_starts + 1
    );
    assert_eq!(service.stop_count.load(Ordering::SeqCst), initial_stops + 1);
    assert_eq!(service.state(), PortServiceState::Running);
}

/// Test healthy service returns healthy check
#[tokio::test]
async fn test_healthy_service_health_check() {
    let service = TestService::new("healthy-service", true);
    let check = service.health_check().await;

    assert_eq!(check.name, "healthy-service");
    assert_eq!(check.status, DependencyHealth::Healthy);
    assert!(check.message.is_some());
    assert!(check.latency_ms.is_some());
    assert!(check.last_check > 0);
}

/// Test unhealthy service returns unhealthy check
#[tokio::test]
async fn test_unhealthy_service_health_check() {
    let service = TestService::new("unhealthy-service", false);
    let check = service.health_check().await;

    assert_eq!(check.name, "unhealthy-service");
    assert_eq!(check.status, DependencyHealth::Unhealthy);
    assert!(check.message.is_some());
    assert!(check.latency_ms.is_none()); // Unhealthy doesn't report latency
}

/// Test service can be used through trait object
#[tokio::test]
async fn test_service_as_trait_object() {
    let service: Arc<dyn LifecycleManaged> = Arc::new(TestService::new("dynamic-service", true));

    assert_eq!(service.name(), "dynamic-service");
    assert_eq!(service.state(), PortServiceState::Stopped);

    service.start().await.expect("start should succeed");
    assert_eq!(service.state(), PortServiceState::Running);

    let health = service.health_check().await;
    assert_eq!(health.status, DependencyHealth::Healthy);

    service.stop().await.expect("stop should succeed");
    assert_eq!(service.state(), PortServiceState::Stopped);
}

/// Test multiple services can operate independently
#[tokio::test]
async fn test_multiple_services() {
    let service_a = TestService::new("service-a", true);
    let service_b = TestService::new("service-b", false);

    // Start only service A
    service_a.start().await.expect("start A should succeed");

    assert_eq!(service_a.state(), PortServiceState::Running);
    assert_eq!(service_b.state(), PortServiceState::Stopped);

    // Health checks reflect individual states
    let health_a = service_a.health_check().await;
    let health_b = service_b.health_check().await;

    assert_eq!(health_a.status, DependencyHealth::Healthy);
    assert_eq!(health_b.status, DependencyHealth::Unhealthy);
}
