//! Dependency Injection System - Shaku-based Architecture
//!
//! This module implements a strict Shaku-based dependency injection system
//! following Clean Architecture and Domain-Driven Design principles.
//!
//! ## Architecture Overview
//!
//! The DI system is organized into hierarchical modules by domain:
//!
//! ```text
//! ApplicationModule (Root)
//! ├── InfrastructureModule
//! ├── ServerModule
//! ├── AdaptersModule
//! └── AdminModule (depends on all above)
//! ```
//!
//! ## Shaku Best Practices Implemented
//!
//! 1. **Module Traits**: Each domain has a trait defining its interface
//! 2. **Submodules**: Modules use `use dyn ModuleTrait` for composition
//! 3. **Component Registration**: Only concrete implementations in `module!` macros
//! 4. **Provider Pattern**: Null providers as defaults, overridden at runtime
//! 5. **Builder Pattern**: `Module::builder(submodules...).build()` for construction
//!
//! ## Key Principles
//!
//! - **Trait-based DI**: All dependencies injected as `Arc<dyn Trait>`
//! - **Domain Separation**: Infrastructure concerns separate from business logic
//! - **Testability**: Null providers enable isolated testing
//! - **Runtime Configuration**: Component overrides for production providers
//!
//! **ARCHITECTURE**: This module contains ONLY wiring logic via Shaku.
//! Business logic and adapters are in their respective domain crates.

pub mod bootstrap;
pub mod dispatch;
pub mod modules;
pub mod resolver;

pub use bootstrap::*;
pub use dispatch::*;
pub use resolver::{resolve_providers, ResolvedProviders};
