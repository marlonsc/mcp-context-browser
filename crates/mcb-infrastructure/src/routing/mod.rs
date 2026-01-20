//! Provider Routing Infrastructure
//!
//! Provides intelligent routing and selection of backend providers
//! based on health status, cost, quality, and operational requirements.
//!
//! ## Components
//!
//! - [`NullProviderRouter`] - No-op router for testing
//! - [`DefaultProviderRouter`] - Production router with health tracking
//!
//! ## Usage via DI
//!
//! ```no_run
//! // Providers are obtained via DI container
//! // let router: Arc<dyn ProviderRouter> = container.resolve();
//! // let provider = router.select_embedding_provider(&context).await?;
//! ```

mod health;
mod router;

// Re-export for DI registration
pub use health::{HealthMonitor, InMemoryHealthMonitor, NullHealthMonitor};
pub use router::{DefaultProviderRouter, NullProviderRouter};
