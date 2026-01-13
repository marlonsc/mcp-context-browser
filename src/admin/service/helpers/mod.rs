//! Helper modules for AdminService implementation
//!
//! Splits the AdminService implementation into focused domains:
//! - `logging` - Log retrieval, filtering, export, and statistics
//! - `maintenance` - Cache clearing, provider restart, index rebuild, cleanup
//! - `health` - Health checks, connectivity tests, performance tests
//! - `backup` - Backup creation, listing, and restoration
//! - `configuration` - Configuration history tracking and persistence
//! - `activity` - Activity feed tracking from EventBus
//! - `admin_defaults` - Default configuration values for admin operations
//! - `runtime_config` - Dynamic configuration values from running subsystems
//! - `route_discovery` - Dynamic route registration and discovery
//! - `subsystems` - Subsystem information building

pub mod activity;
pub mod admin_defaults;
pub mod backup;
pub mod configuration;
pub mod defaults;
pub mod health;
pub mod logging;
pub mod maintenance;
pub mod route_discovery;
pub mod runtime_config;
pub mod subsystems;
