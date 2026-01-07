//! Web interface module using Tera templates and HTMX
//!
//! This module provides a complete web interface generated server-side
//! using Tera templates and HTMX for dynamic interactions.

use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tera::{Context, Tera};

use crate::admin::models::{AdminState, ApiResponse};
use crate::core::error::Result;

/// Web interface manager
pub struct WebInterface {
    tera: Tera,
}

impl WebInterface {
    /// Create a new web interface manager
    pub fn new() -> Result<Self> {
        let mut tera = Tera::new("src/admin/web/templates/**/*")?;

        // Add custom functions
        tera.register_function("format_bytes", format_bytes);
        tera.register_function("format_duration", format_duration);
        tera.register_function("format_percentage", format_percentage);

        Ok(Self { tera })
    }

    /// Get the web routes
    pub fn routes(&self, state: AdminState) -> Router {
        Router::new()
            .route("/", get(Self::dashboard_page))
            .route("/login", get(Self::login_page))
            .route("/login", post(Self::login_post))
            .route("/logout", post(Self::logout))
            .route("/dashboard", get(Self::dashboard_page))
            .route("/providers", get(Self::providers_page))
            .route("/indexes", get(Self::indexes_page))
            .route("/config", get(Self::config_page))
            .route("/search", get(Self::search_page))
            // HTMX endpoints
            .route("/htmx/dashboard/metrics", get(Self::htmx_dashboard_metrics))
            .route("/htmx/providers/list", get(Self::htmx_providers_list))
            .route("/htmx/indexes/list", get(Self::htmx_indexes_list))
            // Static assets
            .route("/static/bootstrap.css", get(Self::bootstrap_css))
            .route("/static/bootstrap.js", get(Self::bootstrap_js))
            .route("/static/htmx.js", get(Self::htmx_js))
            .with_state(state)
    }

    /// Render a template with context
    fn render(&self, template: &str, context: &Context) -> Result<String> {
        Ok(self.tera.render(template, context)?)
    }

    // Page handlers

    async fn dashboard_page(State(state): State<AdminState>) -> impl IntoResponse {
        let interface = WebInterface::new().unwrap();

        // Get dashboard data
        let health = match crate::admin::api::API.get_health().await {
            Ok(data) => data.data,
            Err(_) => crate::admin::models::HealthResponse {
                timestamp: 0,
                service: "unknown".to_string(),
                version: "unknown".to_string(),
                uptime: 0,
                pid: 0,
                status: "unknown".to_string(),
            },
        };

        let mut context = Context::new();
        context.insert("title", "Dashboard");
        context.insert("page", "dashboard");
        context.insert("health", &health);

        match interface.render("dashboard.html", &context) {
            Ok(html) => Html(html).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            )
                .into_response(),
        }
    }

    async fn login_page() -> impl IntoResponse {
        let interface = WebInterface::new().unwrap();

        let mut context = Context::new();
        context.insert("title", "Admin Login");

        match interface.render("login.html", &context) {
            Ok(html) => Html(html).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            )
                .into_response(),
        }
    }

    async fn login_post() -> impl IntoResponse {
        // TODO: Implement login logic
        "Login not implemented yet".into_response()
    }

    async fn logout() -> impl IntoResponse {
        // TODO: Implement logout logic
        "Logout not implemented yet".into_response()
    }

    async fn providers_page(State(state): State<AdminState>) -> impl IntoResponse {
        let interface = WebInterface::new().unwrap();

        let mut context = Context::new();
        context.insert("title", "Providers");
        context.insert("page", "providers");

        match interface.render("providers.html", &context) {
            Ok(html) => Html(html).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            )
                .into_response(),
        }
    }

    async fn indexes_page(State(state): State<AdminState>) -> impl IntoResponse {
        let interface = WebInterface::new().unwrap();

        let mut context = Context::new();
        context.insert("title", "Indexes");
        context.insert("page", "indexes");

        match interface.render("indexes.html", &context) {
            Ok(html) => Html(html).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            )
                .into_response(),
        }
    }

    async fn config_page(State(state): State<AdminState>) -> impl IntoResponse {
        let interface = WebInterface::new().unwrap();

        let mut context = Context::new();
        context.insert("title", "Configuration");
        context.insert("page", "config");

        match interface.render("config.html", &context) {
            Ok(html) => Html(html).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            )
                .into_response(),
        }
    }

    async fn search_page(State(state): State<AdminState>) -> impl IntoResponse {
        let interface = WebInterface::new().unwrap();

        let mut context = Context::new();
        context.insert("title", "Search");
        context.insert("page", "search");

        match interface.render("search.html", &context) {
            Ok(html) => Html(html).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            )
                .into_response(),
        }
    }

    // HTMX handlers

    async fn htmx_dashboard_metrics(State(state): State<AdminState>) -> impl IntoResponse {
        let interface = WebInterface::new().unwrap();

        // Get fresh metrics
        let health = match crate::admin::api::API.get_health().await {
            Ok(data) => data.data,
            Err(_) => crate::admin::models::HealthResponse {
                timestamp: 0,
                service: "unknown".to_string(),
                version: "unknown".to_string(),
                uptime: 0,
                pid: 0,
                status: "unknown".to_string(),
            },
        };

        let mut context = Context::new();
        context.insert("health", &health);

        match interface.render("htmx/dashboard_metrics.html", &context) {
            Ok(html) => Html(html).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            )
                .into_response(),
        }
    }

    async fn htmx_providers_list(State(state): State<AdminState>) -> impl IntoResponse {
        let interface = WebInterface::new().unwrap();

        // Get providers
        let providers = match crate::admin::api::API.get_providers().await {
            Ok(data) => data.data,
            Err(_) => vec![],
        };

        let mut context = Context::new();
        context.insert("providers", &providers);

        match interface.render("htmx/providers_list.html", &context) {
            Ok(html) => Html(html).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            )
                .into_response(),
        }
    }

    async fn htmx_indexes_list(State(state): State<AdminState>) -> impl IntoResponse {
        let interface = WebInterface::new().unwrap();

        // Get indexes
        let indexes = match crate::admin::api::API.get_indexes().await {
            Ok(data) => data.data,
            Err(_) => vec![],
        };

        let mut context = Context::new();
        context.insert("indexes", &indexes);

        match interface.render("htmx/indexes_list.html", &context) {
            Ok(html) => Html(html).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            )
                .into_response(),
        }
    }

    // Static assets

    async fn bootstrap_css() -> impl IntoResponse {
        let css = r#"
        /* Bootstrap CSS served from Rust */
        @import url('https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css');
        "#;

        (
            [(header::CONTENT_TYPE, "text/css")],
            css,
        )
    }

    async fn bootstrap_js() -> impl IntoResponse {
        let js = r#"
// Bootstrap JS served from Rust - redirect to CDN
import('https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/js/bootstrap.bundle.min.js');
"#;

        (
            [(header::CONTENT_TYPE, "application/javascript")],
            js,
        )
    }

    async fn htmx_js() -> impl IntoResponse {
        let js = r#"
// HTMX JS served from Rust - redirect to CDN
import('https://unpkg.com/htmx.org@1.9.10');
"#;

        (
            [(header::CONTENT_TYPE, "application/javascript")],
            js,
        )
    }
}

// Tera custom functions

fn format_bytes(args: &HashMap<String, serde_json::Value>) -> tera::Result<serde_json::Value> {
    let bytes = args
        .get("bytes")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let formatted = if bytes >= 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    };

    Ok(serde_json::Value::String(formatted))
}

fn format_duration(args: &HashMap<String, serde_json::Value>) -> tera::Result<serde_json::Value> {
    let seconds = args
        .get("seconds")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    let formatted = if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    };

    Ok(serde_json::Value::String(formatted))
}

fn format_percentage(args: &HashMap<String, serde_json::Value>) -> tera::Result<serde_json::Value> {
    let value = args
        .get("value")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    Ok(serde_json::Value::String(format!("{:.1}%", value)))
}