//! Basic web interface for admin operations
//!
//! This module provides the web UI for the admin dashboard.
//! It uses server-side rendering with Tera templates, composing
//! data from AdminService via ViewModelBuilder.

pub mod builders;
pub mod html_helpers;
pub mod view_model_builders;
pub mod view_models;

use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;
use tera::{Context, Tera};

use crate::admin::auth::web_auth_middleware;
use crate::admin::models::AdminState;
use crate::admin::web::html_helpers::htmx_error;

use self::builders::ViewModelBuilder;

// Embed all templates at compile time so binary is self-contained
const TPL_BASE: &str = include_str!("templates/base.html");
const TPL_DASHBOARD: &str = include_str!("templates/dashboard.html");
const TPL_PROVIDERS: &str = include_str!("templates/providers.html");
const TPL_INDEXES: &str = include_str!("templates/indexes.html");
const TPL_CONFIGURATION: &str = include_str!("templates/configuration.html");
const TPL_LOGS: &str = include_str!("templates/logs.html");
const TPL_MAINTENANCE: &str = include_str!("templates/maintenance.html");
const TPL_DIAGNOSTICS: &str = include_str!("templates/diagnostics.html");
const TPL_DATA_MANAGEMENT: &str = include_str!("templates/data_management.html");
const TPL_LOGIN: &str = include_str!("templates/login.html");
const TPL_ICONS: &str = include_str!("templates/icons.html");
const TPL_ADMIN_CSS: &str = include_str!("templates/admin.css");
const TPL_ADMIN_JS: &str = include_str!("templates/admin.js");
// HTMX partials
const TPL_HTMX_DASHBOARD_METRICS: &str = include_str!("templates/htmx/dashboard_metrics.html");
const TPL_HTMX_PROVIDERS_LIST: &str = include_str!("templates/htmx/providers_list.html");
const TPL_HTMX_INDEXES_LIST: &str = include_str!("templates/htmx/indexes_list.html");
const TPL_HTMX_SUBSYSTEMS_LIST: &str = include_str!("templates/htmx/subsystems_list.html");
const TPL_HTMX_CONFIG_DIFF: &str = include_str!("templates/htmx/config_diff.html");
const TPL_ERROR: &str = include_str!("templates/error.html");

/// Web interface manager
pub struct WebInterface {
    templates: Arc<Tera>,
}

impl WebInterface {
    /// Create a new web interface manager with embedded templates
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize Tera with embedded templates (no filesystem access needed)
        let mut tera = Tera::default();

        // Add all embedded templates
        // Important: icons.html MUST be registered FIRST since base.html imports macros from it
        tera.add_raw_template("icons.html", TPL_ICONS)?;
        tera.add_raw_template("base.html", TPL_BASE)?;
        tera.add_raw_template("dashboard.html", TPL_DASHBOARD)?;
        tera.add_raw_template("providers.html", TPL_PROVIDERS)?;
        tera.add_raw_template("indexes.html", TPL_INDEXES)?;
        tera.add_raw_template("configuration.html", TPL_CONFIGURATION)?;
        tera.add_raw_template("logs.html", TPL_LOGS)?;
        tera.add_raw_template("maintenance.html", TPL_MAINTENANCE)?;
        tera.add_raw_template("diagnostics.html", TPL_DIAGNOSTICS)?;
        tera.add_raw_template("data_management.html", TPL_DATA_MANAGEMENT)?;
        tera.add_raw_template("login.html", TPL_LOGIN)?;
        // HTMX partials
        tera.add_raw_template("htmx/dashboard_metrics.html", TPL_HTMX_DASHBOARD_METRICS)?;
        tera.add_raw_template("htmx/providers_list.html", TPL_HTMX_PROVIDERS_LIST)?;
        tera.add_raw_template("htmx/indexes_list.html", TPL_HTMX_INDEXES_LIST)?;
        tera.add_raw_template("htmx/subsystems_list.html", TPL_HTMX_SUBSYSTEMS_LIST)?;
        tera.add_raw_template("htmx/config_diff.html", TPL_HTMX_CONFIG_DIFF)?;
        tera.add_raw_template("error.html", TPL_ERROR)?;

        Ok(Self {
            templates: Arc::new(tera),
        })
    }

    /// Get the templates instance
    pub fn templates(&self) -> Arc<Tera> {
        Arc::clone(&self.templates)
    }

    /// Get the web routes
    ///
    /// Protected routes require authentication via cookie.
    /// Public routes (login, CSS) are accessible without authentication.
    pub fn routes(&self, state: AdminState) -> Router {
        // Protected routes - require authentication
        let protected_routes = Router::new()
            .route("/", get(dashboard_handler))
            .route("/dashboard", get(dashboard_handler))
            .route("/providers", get(providers_handler))
            .route("/indexes", get(indexes_handler))
            .route("/config", get(configuration_handler))
            .route("/logs", get(logs_handler))
            .route("/maintenance", get(maintenance_handler))
            .route("/diagnostics", get(diagnostics_handler))
            .route("/data", get(data_management_handler))
            // HTMX partials (also protected as they contain sensitive data)
            .route(
                "/htmx/dashboard-metrics",
                get(htmx_dashboard_metrics_handler),
            )
            .route("/htmx/providers-list", get(htmx_providers_list_handler))
            .route("/htmx/indexes-list", get(htmx_indexes_list_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                web_auth_middleware,
            ));

        // Public routes - no authentication required
        let public_routes = Router::new()
            .route("/login", get(login_handler))
            .route("/static/admin.css", get(css_handler))
            .route("/static/admin.js", get(js_handler));

        // Merge routes
        protected_routes.merge(public_routes).with_state(state)
    }
}

// --- Context Creation Helpers ---

/// Create Tera context for a page with view model
/// Automatically inserts "page" and "vm" keys.
///
/// Reduces boilerplate from 3 lines to 1 line per handler.
#[inline]
fn create_page_context<T: serde::Serialize>(page: &str, view_model: &T) -> Context {
    let mut context = Context::new();
    context.insert("page", page);
    context.insert("vm", view_model);
    context
}

/// Create Tera context with JSON-serialized view model
/// Useful for pages that need client-side JavaScript access to data.
///
/// Automatically inserts "page", "vm", and "vm_json" keys.
/// Reduces boilerplate from 5-6 lines to 2 lines per handler.
#[inline]
fn create_page_context_with_json<T: serde::Serialize>(page: &str, view_model: &T) -> Context {
    let vm_json = serde_json::to_string(view_model).unwrap_or_else(|_| "{}".to_string());
    let mut context = Context::new();
    context.insert("page", page);
    context.insert("vm", view_model);
    context.insert("vm_json", &vm_json);
    context
}

// --- Handlers ---

async fn dashboard_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let builder = ViewModelBuilder::new(&state);

    match builder.build_dashboard().await {
        Ok(view_model) => render_template(
            &state.templates,
            "dashboard.html",
            &create_page_context_with_json(view_model.page, &view_model),
        ),
        Err(e) => {
            tracing::error!("Failed to build dashboard view model: {}", e);
            render_error_page(&state.templates, "Dashboard Error", &e.to_string())
        }
    }
}

async fn providers_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let builder = ViewModelBuilder::new(&state);

    match builder.build_providers_page().await {
        Ok(view_model) => render_template(
            &state.templates,
            "providers.html",
            &create_page_context(view_model.page, &view_model),
        ),
        Err(e) => {
            tracing::error!("Failed to build providers view model: {}", e);
            render_error_page(&state.templates, "Providers Error", &e.to_string())
        }
    }
}

async fn indexes_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let builder = ViewModelBuilder::new(&state);

    match builder.build_indexes_page().await {
        Ok(view_model) => render_template(
            &state.templates,
            "indexes.html",
            &create_page_context(view_model.page, &view_model),
        ),
        Err(e) => {
            tracing::error!("Failed to build indexes view model: {}", e);
            render_error_page(&state.templates, "Indexes Error", &e.to_string())
        }
    }
}

async fn configuration_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let builder = ViewModelBuilder::new(&state);

    match builder.build_configuration_page().await {
        Ok(view_model) => {
            let mut context = Context::new();
            context.insert("page", &view_model.page);
            context.insert("page_description", &view_model.page_description);
            context.insert("vm", &view_model);
            render_template(&state.templates, "configuration.html", &context)
        }
        Err(e) => {
            tracing::error!("Failed to build configuration view model: {}", e);
            render_error_page(&state.templates, "Configuration Error", &e.to_string())
        }
    }
}

async fn logs_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let builder = ViewModelBuilder::new(&state);

    match builder.build_logs_page().await {
        Ok(view_model) => {
            let mut context = Context::new();
            context.insert("page", &view_model.page);
            context.insert("page_description", &view_model.page_description);
            context.insert("vm", &view_model);
            render_template(&state.templates, "logs.html", &context)
        }
        Err(e) => {
            tracing::error!("Failed to build logs view model: {}", e);
            render_error_page(&state.templates, "Logs Error", &e.to_string())
        }
    }
}

async fn maintenance_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("page", "maintenance");
    render_template(&state.templates, "maintenance.html", &context)
}

async fn diagnostics_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("page", "diagnostics");
    render_template(&state.templates, "diagnostics.html", &context)
}

async fn data_management_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("page", "data");
    render_template(&state.templates, "data_management.html", &context)
}

async fn login_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let context = Context::new();
    render_template(&state.templates, "login.html", &context)
}

async fn css_handler() -> impl IntoResponse {
    Response::builder()
        .header("Content-Type", "text/css")
        .body(TPL_ADMIN_CSS.to_string())
        .map_err(|e| {
            tracing::error!("Failed to build CSS response: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

async fn js_handler() -> impl IntoResponse {
    Response::builder()
        .header("Content-Type", "application/javascript")
        .body(TPL_ADMIN_JS.to_string())
        .map_err(|e| {
            tracing::error!("Failed to build JS response: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

// --- HTMX Handlers ---

async fn htmx_dashboard_metrics_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let builder = ViewModelBuilder::new(&state);

    match builder.build_dashboard().await {
        Ok(view_model) => {
            let mut context = Context::new();
            context.insert("vm", &view_model);
            render_template(&state.templates, "htmx/dashboard_metrics.html", &context)
        }
        Err(_) => htmx_error("Failed to load metrics").into_response(),
    }
}

async fn htmx_providers_list_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let builder = ViewModelBuilder::new(&state);

    match builder.build_providers_page().await {
        Ok(view_model) => {
            let mut context = Context::new();
            context.insert("providers", &view_model.providers);
            render_template(&state.templates, "htmx/providers_list.html", &context)
        }
        Err(_) => htmx_error("Failed to load providers").into_response(),
    }
}

async fn htmx_indexes_list_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let builder = ViewModelBuilder::new(&state);

    match builder.build_indexes_page().await {
        Ok(view_model) => {
            let mut context = Context::new();
            context.insert("indexes", &view_model.indexes);
            render_template(&state.templates, "htmx/indexes_list.html", &context)
        }
        Err(_) => htmx_error("Failed to load indexes").into_response(),
    }
}

// --- Helper ---

fn render_template(tera: &Tera, name: &str, context: &Context) -> Response {
    match tera.render(name, context) {
        Ok(html) => Html(html).into_response(),
        Err(err) => {
            tracing::error!("Template error: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", err),
            )
                .into_response()
        }
    }
}

fn render_error_page(tera: &Tera, title: &str, message: &str) -> Response {
    let error_vm = ViewModelBuilder::build_error(title, message, None);
    let mut context = Context::new();
    context.insert("error", &error_vm);
    context.insert("page", "error");

    match tera.render("error.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}: {}", title, message),
        )
            .into_response(),
    }
}
