//! Admin Web UI Module
//!
//! Provides an HTMX-powered web interface for the admin panel.
//! Templates are embedded at compile time for zero-dependency deployment.
//!
//! ## Pages
//!
//! - `/` or `/ui` - Dashboard with real-time metrics
//! - `/ui/config` - Configuration editor with live reload
//! - `/ui/health` - Health status and dependency monitoring
//! - `/ui/indexing` - Indexing operation progress

use axum::{
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

// Embed templates at compile time
const INDEX_HTML: &str = include_str!("templates/index.html");
const CONFIG_HTML: &str = include_str!("templates/config.html");
const HEALTH_HTML: &str = include_str!("templates/health.html");
const INDEXING_HTML: &str = include_str!("templates/indexing.html");

/// Dashboard page handler
pub async fn dashboard() -> Html<&'static str> {
    Html(INDEX_HTML)
}

/// Configuration page handler
pub async fn config_page() -> Html<&'static str> {
    Html(CONFIG_HTML)
}

/// Health status page handler
pub async fn health_page() -> Html<&'static str> {
    Html(HEALTH_HTML)
}

/// Indexing status page handler
pub async fn indexing_page() -> Html<&'static str> {
    Html(INDEXING_HTML)
}

/// Favicon handler - returns a simple SVG icon
pub async fn favicon() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/svg+xml")],
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><text y=".9em" font-size="90">📊</text></svg>"#,
    )
}

/// Create the admin web UI router
///
/// Routes:
/// - GET `/` - Dashboard
/// - GET `/ui` - Dashboard (alias)
/// - GET `/ui/config` - Configuration page
/// - GET `/ui/health` - Health status page
/// - GET `/ui/indexing` - Indexing status page
/// - GET `/favicon.ico` - Favicon
pub fn web_router() -> Router {
    Router::new()
        .route("/", get(dashboard))
        .route("/ui", get(dashboard))
        .route("/ui/config", get(config_page))
        .route("/ui/health", get(health_page))
        .route("/ui/indexing", get(indexing_page))
        .route("/favicon.ico", get(favicon))
}
