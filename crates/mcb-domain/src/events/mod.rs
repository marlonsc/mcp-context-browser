//! Domain Events
//!
//! Domain events represent significant business occurrences that have happened
//! within the domain. Events are immutable facts that other parts of the system
//! can react to.
//!
//! ## Domain Events
//!
//! | Event | Description |
//! |-------|-------------|
//! | [`DomainEvent`] | Base trait for all domain events |
//! | [`EventPublisher`] | Interface for publishing domain events |

/// Domain event definitions and publisher interface
pub mod domain_events;

// Re-export domain event types
pub use domain_events::{DomainEvent, EventPublisher};
