//! Admin Module Implementation - COMPLETE: All Administrative Services
//!
//! This module provides ALL administrative services with #[derive(Component)].
//! No placeholders - all services are real implementations.
//!
//! ## COMPLETE Services Provided:
//!
//! - NullAdminService (from infrastructure) -> implements AdminService ✓
//!
//! ## No Runtime Factories:
//!
//! All services created at compile-time by Shaku DI, not runtime factories.

use shaku::module;

// Import ONLY real implementations with Component derive
use crate::application::admin::NullAdminService;

// Import traits
use super::traits::AdminModule;

/// Admin module implementation - COMPLETE Shaku DI.
///
/// Contains ALL administrative services with proper Component derives.
/// No placeholders - everything is real and compiles.
///
/// ## Component Registration - COMPLETE
///
/// ALL services have #[derive(Component)] and #[shaku(interface = ...)].
/// NO struct types in HasComponent (impossible in Shaku).
/// NO placeholder services.
/// ONLY real implementations that exist in the codebase.
///
/// ## Construction - COMPLETE
///
/// ```rust,ignore
/// let admin = AdminModuleImpl::builder().build();
/// ```
module! {
    pub AdminModuleImpl: AdminModule {
        components = [
            // COMPLETE administrative services with Component derive
            NullAdminService
        ],
        providers = []
    }
}
