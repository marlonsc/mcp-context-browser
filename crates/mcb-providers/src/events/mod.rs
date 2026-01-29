//! Event Bus Provider Implementations
//!
//! Provides event bus backends for domain events.
//!
//! ## Available Providers
//!
//! | Provider | Type | Description |
//! |----------|------|-------------|
//! | NullEventBusProvider | Testing | Discards all events |
//! | TokioEventBusProvider | In-Process | Tokio broadcast channels |
//! | NatsEventBusProvider | Distributed | NATS for multi-process systems |
//!
//! ## Provider Selection Guide
//!
//! - **Testing**: Use `NullEventBusProvider` to discard events
//! - **Single Instance**: Use `TokioEventBusProvider` for in-process events
//! - **Distributed**: Use `NatsEventBusProvider` for multi-process/node systems

#[cfg(feature = "events-nats")]
pub mod nats;
pub mod null;
pub mod tokio;

// Re-export providers
#[cfg(feature = "events-nats")]
pub use nats::{NatsEventBusProvider, NatsEventPublisher};
pub use null::{NullEventBusProvider, NullEventPublisher};
pub use tokio::{TokioEventBusProvider, TokioEventPublisher};

// Re-export port trait from application layer
pub use mcb_application::ports::infrastructure::{DomainEventStream, EventBusProvider};

// Re-export domain event types
pub use mcb_domain::events::DomainEvent;
