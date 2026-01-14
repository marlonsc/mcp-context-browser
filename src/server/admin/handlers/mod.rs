//! HTTP handlers for admin API endpoints
//!
//! This module is organized by domain:
//! - `config` - Configuration management handlers
//! - `providers` - Provider management handlers
//! - `indexes` - Index management handlers
//! - `system` - System status and dashboard handlers
//! - `maintenance` - Maintenance operation handlers
//! - `diagnostics` - Health check and testing handlers
//! - `data` - Backup and restore handlers
//! - `subsystems` - Subsystem control handlers
//! - `htmx` - HTMX partial response handlers

mod config;
/// Data management handlers for backups and restoration
mod data;
/// System diagnostics and health check handlers
mod diagnostics;
/// Shared handler utilities and helpers
mod helpers;
pub mod htmx;
/// Index management and monitoring handlers
mod indexes;
/// System maintenance and cleanup handlers
mod maintenance;
/// Provider management and configuration handlers
mod providers;
/// Subsystem control and monitoring handlers
mod subsystems;
/// System status and dashboard handlers
mod system;

// Re-export all handlers for backwards compatibility
pub use config::*;
pub use data::*;
pub use diagnostics::*;
pub use helpers::*;
pub use htmx::*;
pub use indexes::*;
pub use maintenance::*;
pub use providers::*;
pub use subsystems::*;
pub use system::*;

// Common imports used across handlers
pub(crate) mod common {
    pub use axum::{
        extract::{Extension, Path, Query, State},
        http::StatusCode,
        response::{Html, Json},
    };

    pub use crate::server::admin::models::{
        AdminState, ApiResponse, IndexInfo, IndexOperationRequest, ProviderConfigRequest,
        ProviderInfo, SystemConfig,
    };
    pub use crate::server::admin::service::MaintenanceResult;
}

// Query parameter structures shared across handlers
use serde::Deserialize;

/// Query parameters for search
#[derive(Deserialize)]
pub struct SearchQuery {
    /// Q
    pub q: String,
    /// Optional limit value
    pub limit: Option<usize>,
}

/// Query parameters for history endpoints
#[derive(Deserialize)]
pub struct HistoryQuery {
    /// Optional limit value
    pub limit: Option<usize>,
}

/// Query parameters for log export
#[derive(Deserialize)]
pub struct ExportQuery {
    /// Format
    pub format: crate::server::admin::service::LogExportFormat,
}

/// Request body for sending signals to subsystems
#[derive(Deserialize)]
pub struct SubsystemSignalRequest {
    /// Signal
    pub signal: crate::server::admin::service::SubsystemSignal,
}

/// Request for cleanup operation
#[derive(Debug, Deserialize)]
pub struct CleanupRequest {
    /// Optional older_than_days value
    pub older_than_days: Option<u32>,
}
