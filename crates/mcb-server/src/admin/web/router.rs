//! Web Router Module
//!
//! Router configuration for the admin web interface.
//!
//! Migrated from Axum to Rocket in v0.1.2 (ADR-026).

use rocket::{Build, Rocket, routes};

use super::handlers;

/// Create the admin web UI rocket instance
///
/// Routes:
/// - GET `/` - Dashboard
/// - GET `/ui` - Dashboard (alias)
/// - GET `/ui/config` - Configuration page
/// - GET `/ui/health` - Health status page
/// - GET `/ui/indexing` - Indexing status page
/// - GET `/favicon.ico` - Favicon
pub fn web_rocket() -> Rocket<Build> {
    rocket::build().mount(
        "/",
        routes![
            handlers::dashboard,
            handlers::dashboard_ui,
            handlers::config_page,
            handlers::health_page,
            handlers::indexing_page,
            handlers::favicon,
        ],
    )
}

/// Get routes for mounting in a parent Rocket instance
pub fn web_routes() -> Vec<rocket::Route> {
    routes![
        handlers::dashboard,
        handlers::dashboard_ui,
        handlers::config_page,
        handlers::health_page,
        handlers::indexing_page,
        handlers::favicon,
    ]
}
