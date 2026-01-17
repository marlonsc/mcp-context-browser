//! Service Lifecycle Management
//!
//! Provides centralized lifecycle management for all managed services.
//! The ServiceManager coordinates start/stop/restart operations and
//! publishes state change events via the EventBus.
//!
//! ## Architecture
//!
//! ```text
//!                    ┌─────────────────┐
//!                    │  ServiceManager │
//!                    └────────┬────────┘
//!                             │
//!        ┌────────────────────┼────────────────────┐
//!        │                    │                    │
//!        ▼                    ▼                    ▼
//! ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
//! │   Service A  │    │   Service B  │    │   Service C  │
//! │ (Embedding)  │    │ (VectorStore)│    │   (Cache)    │
//! └──────────────┘    └──────────────┘    └──────────────┘
//!        │                    │                    │
//!        └────────────────────┼────────────────────┘
//!                             │
//!                             ▼
//!                    ┌─────────────────┐
//!                    │    EventBus     │
//!                    │ (SSE/WebSocket) │
//!                    └─────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use mcb_infrastructure::infrastructure::lifecycle::ServiceManager;
//!
//! // Create manager with event bus
//! let manager = ServiceManager::new(event_bus);
//!
//! // Register services
//! manager.register(Arc::new(embedding_service));
//! manager.register(Arc::new(vector_store_service));
//!
//! // List all services
//! for info in manager.list() {
//!     println!("{}: {:?}", info.name, info.state);
//! }
//!
//! // Restart a specific service
//! manager.restart("embedding").await?;
//! ```

use dashmap::DashMap;
use mcb_application::ports::admin::{
    DependencyHealthCheck, LifecycleManaged, ServiceState as PortServiceState, ShutdownCoordinator,
};
use mcb_application::ports::infrastructure::EventBusProvider;
use mcb_domain::events::{DomainEvent, ServiceState as EventServiceState};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::{error, info, warn};

/// Information about a registered service
#[derive(Debug, Clone, Serialize)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,
    /// Current state
    pub state: PortServiceState,
}

/// Error type for service manager operations
#[derive(Debug, thiserror::Error)]
pub enum ServiceManagerError {
    /// Service not found
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    /// Service operation failed
    #[error("Service operation failed: {0}")]
    OperationFailed(#[from] mcb_domain::error::Error),
}

/// Central coordinator for service lifecycle management
///
/// The ServiceManager tracks all managed services and provides operations
/// to start, stop, and restart them. State changes are published as
/// domain events for real-time monitoring.
#[allow(dead_code)] // Will be used in admin API handlers
pub struct ServiceManager {
    /// Registered services by name
    services: DashMap<String, Arc<dyn LifecycleManaged>>,
    /// Event bus for publishing state changes
    event_bus: Arc<dyn EventBusProvider>,
}

#[allow(dead_code)] // Methods will be used in admin API handlers
impl ServiceManager {
    /// Create a new service manager with the given event bus
    pub fn new(event_bus: Arc<dyn EventBusProvider>) -> Self {
        Self {
            services: DashMap::new(),
            event_bus,
        }
    }

    /// Register a service for lifecycle management
    ///
    /// The service will be tracked and can be controlled via this manager.
    pub fn register(&self, service: Arc<dyn LifecycleManaged>) {
        let name = service.name().to_string();
        info!(service = %name, "Registering service for lifecycle management");
        self.services.insert(name, service);
    }

    /// Unregister a service from lifecycle management
    pub fn unregister(&self, name: &str) -> Option<Arc<dyn LifecycleManaged>> {
        info!(service = %name, "Unregistering service from lifecycle management");
        self.services.remove(name).map(|(_, v)| v)
    }

    /// Get information about all registered services
    pub fn list(&self) -> Vec<ServiceInfo> {
        self.services
            .iter()
            .map(|entry| ServiceInfo {
                name: entry.key().clone(),
                state: entry.value().state(),
            })
            .collect()
    }

    /// Get information about a specific service
    pub fn get(&self, name: &str) -> Option<ServiceInfo> {
        self.services.get(name).map(|entry| ServiceInfo {
            name: entry.key().clone(),
            state: entry.value().state(),
        })
    }

    /// Check if a service is registered
    pub fn contains(&self, name: &str) -> bool {
        self.services.contains_key(name)
    }

    /// Get the number of registered services
    pub fn count(&self) -> usize {
        self.services.len()
    }

    /// Start a specific service
    pub async fn start(&self, name: &str) -> std::result::Result<(), ServiceManagerError> {
        let service = self
            .services
            .get(name)
            .ok_or_else(|| ServiceManagerError::ServiceNotFound(name.to_string()))?;

        let previous_state = service.state();
        info!(service = %name, previous = ?previous_state, "Starting service");

        service.start().await?;

        let new_state = service.state();
        self.emit_state_change(name, new_state, Some(previous_state))
            .await;

        info!(service = %name, state = ?new_state, "Service started");
        Ok(())
    }

    /// Stop a specific service
    pub async fn stop(&self, name: &str) -> std::result::Result<(), ServiceManagerError> {
        let service = self
            .services
            .get(name)
            .ok_or_else(|| ServiceManagerError::ServiceNotFound(name.to_string()))?;

        let previous_state = service.state();
        info!(service = %name, previous = ?previous_state, "Stopping service");

        service.stop().await?;

        let new_state = service.state();
        self.emit_state_change(name, new_state, Some(previous_state))
            .await;

        info!(service = %name, state = ?new_state, "Service stopped");
        Ok(())
    }

    /// Restart a specific service
    pub async fn restart(&self, name: &str) -> std::result::Result<(), ServiceManagerError> {
        let service = self
            .services
            .get(name)
            .ok_or_else(|| ServiceManagerError::ServiceNotFound(name.to_string()))?;

        let previous_state = service.state();
        info!(service = %name, previous = ?previous_state, "Restarting service");

        service.restart().await?;

        let new_state = service.state();
        self.emit_state_change(name, new_state, Some(previous_state))
            .await;

        info!(service = %name, state = ?new_state, "Service restarted");
        Ok(())
    }

    /// Start all registered services
    pub async fn start_all(&self) -> Vec<(String, std::result::Result<(), ServiceManagerError>)> {
        let names: Vec<String> = self.services.iter().map(|e| e.key().clone()).collect();

        let mut results = Vec::with_capacity(names.len());
        for name in names {
            let result = self.start(&name).await;
            results.push((name, result));
        }
        results
    }

    /// Stop all registered services
    pub async fn stop_all(&self) -> Vec<(String, std::result::Result<(), ServiceManagerError>)> {
        let names: Vec<String> = self.services.iter().map(|e| e.key().clone()).collect();

        let mut results = Vec::with_capacity(names.len());
        for name in names {
            let result = self.stop(&name).await;
            results.push((name, result));
        }
        results
    }

    /// Perform health checks on all registered services
    pub async fn health_check_all(&self) -> Vec<DependencyHealthCheck> {
        // Clone the Arc pointers to avoid holding DashMap references across await points
        let services: Vec<Arc<dyn LifecycleManaged>> = self
            .services
            .iter()
            .map(|entry| Arc::clone(entry.value()))
            .collect();

        let mut checks = Vec::with_capacity(services.len());
        for service in services {
            let check = service.health_check().await;
            checks.push(check);
        }

        checks
    }

    /// Emit a service state change event
    async fn emit_state_change(
        &self,
        name: &str,
        state: PortServiceState,
        previous: Option<PortServiceState>,
    ) {
        let event_state = port_to_event_state(state);
        let event_previous = previous.map(port_to_event_state);

        let event = DomainEvent::ServiceStateChanged {
            name: name.to_string(),
            state: event_state,
            previous_state: event_previous,
        };

        let payload = match serde_json::to_vec(&event) {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to serialize state change event: {}", e);
                return;
            }
        };

        if let Err(e) = self.event_bus.publish("service.state", &payload).await {
            error!("Failed to publish state change event: {}", e);
        }
    }
}

/// Convert port ServiceState to domain event ServiceState
#[allow(dead_code)] // Used by ServiceManager::emit_state_change
fn port_to_event_state(state: PortServiceState) -> EventServiceState {
    match state {
        PortServiceState::Starting => EventServiceState::Starting,
        PortServiceState::Running => EventServiceState::Running,
        PortServiceState::Stopping => EventServiceState::Stopping,
        PortServiceState::Stopped => EventServiceState::Stopped,
    }
}

impl std::fmt::Debug for ServiceManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceManager")
            .field("service_count", &self.services.len())
            .field(
                "services",
                &self
                    .services
                    .iter()
                    .map(|e| e.key().clone())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

// ============================================================================
// Default Shutdown Coordinator
// ============================================================================

/// Default implementation of ShutdownCoordinator using atomics and Notify
///
/// This coordinator uses Tokio's Notify for efficient async waiting
/// and an AtomicBool for fast shutdown status checks.
pub struct DefaultShutdownCoordinator {
    /// Shutdown signal flag
    shutdown_signal: AtomicBool,
    /// Notification channel for async waiting
    notify: Notify,
}

impl DefaultShutdownCoordinator {
    /// Create a new shutdown coordinator
    pub fn new() -> Self {
        Self {
            shutdown_signal: AtomicBool::new(false),
            notify: Notify::new(),
        }
    }

    /// Wait asynchronously for shutdown signal
    ///
    /// This method blocks until `signal_shutdown()` is called.
    pub async fn wait_for_shutdown(&self) {
        // If already shutting down, return immediately
        if self.is_shutting_down() {
            return;
        }
        self.notify.notified().await;
    }
}

impl Default for DefaultShutdownCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for DefaultShutdownCoordinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultShutdownCoordinator")
            .field("is_shutting_down", &self.is_shutting_down())
            .finish()
    }
}

// Shaku Component implementation
impl<M: shaku::Module> shaku::Component<M> for DefaultShutdownCoordinator {
    type Interface = dyn ShutdownCoordinator;
    type Parameters = ();

    fn build(_: &mut shaku::ModuleBuildContext<M>, _: Self::Parameters) -> Box<Self::Interface> {
        Box::new(DefaultShutdownCoordinator::new())
    }
}

impl ShutdownCoordinator for DefaultShutdownCoordinator {
    fn signal_shutdown(&self) {
        info!("Shutdown signal received");
        self.shutdown_signal.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    fn is_shutting_down(&self) -> bool {
        self.shutdown_signal.load(Ordering::SeqCst)
    }
}
