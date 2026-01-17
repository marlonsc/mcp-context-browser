//! Routing Module Implementation - Provider Selection with DI
//!
//! This module provides provider routing and selection services.
//! Uses NullProviderRouter as default, override for production use.
//!
//! ## Services Provided:
//!
//! | Service | Default | Use Case |
//! |---------|---------|----------|
//! | ProviderRouter | `NullProviderRouter` | Override with real router in production |
//!
//! ## Production Override Example:
//!
//! ```ignore
//! use mcb_infrastructure::routing::DefaultProviderRouter;
//!
//! let module = RoutingModuleImpl::builder()
//!     .with_component_override::<dyn ProviderRouter>(Box::new(router))
//!     .build();
//! ```

use shaku::module;

// Import null implementation as default
use crate::routing::NullProviderRouter;

// Import trait
use super::traits::RoutingModule;

module! {
    pub RoutingModuleImpl: RoutingModule {
        components = [
            NullProviderRouter  // Testing default - override in production
        ],
        providers = []
    }
}
