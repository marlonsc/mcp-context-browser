//! Dependency Injection layer
//!
//! Uses Shaku for dependency injection to wire together infrastructure components.
//! This follows Clean Architecture principles by providing a composition root
//! that resolves dependencies and creates the application container.

pub mod bootstrap;
pub mod dispatch;
pub mod factory;
pub mod modules;
pub mod registry;

pub use bootstrap::*;
pub use dispatch::*;
pub use factory::*;
pub use modules::*;
pub use registry::*;