//! Event Publisher Implementations
//!
//! Provides event bus backends for domain events.
//!
//! ## Available Providers
//!
//! | Provider | Type | Description |
//! |----------|------|-------------|
//! | [`NullEventPublisher`] | Testing | Discards all events |
//! | [`TokioEventPublisher`] | In-Process | Tokio broadcast channels |
//!
//! ## Provider Selection Guide
//!
//! - **Testing**: Use `NullEventPublisher` to discard events
//! - **Single Instance**: Use `TokioEventPublisher` for in-process events

pub mod null;
pub mod tokio;

// Re-export for convenience
pub use null::NullEventPublisher;
pub use tokio::TokioEventPublisher;

// Re-export domain types
pub use mcb_domain::events::domain_events::{DomainEvent, EventPublisher};
