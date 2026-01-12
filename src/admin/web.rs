//! Basic web interface for admin operations

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

// Embed all templates at compile time so binary is self-contained
const TPL_BASE: &str = include_str!("web/templates/base.html");
const TPL_DASHBOARD: &str = include_str!("web/templates/dashboard.html");
const TPL_PROVIDERS: &str = include_str!("web/templates/providers.html");
const TPL_INDEXES: &str = include_str!("web/templates/indexes.html");
const TPL_CONFIGURATION: &str = include_str!("web/templates/configuration.html");
const TPL_LOGS: &str = include_str!("web/templates/logs.html");
const TPL_MAINTENANCE: &str = include_str!("web/templates/maintenance.html");
const TPL_DIAGNOSTICS: &str = include_str!("web/templates/diagnostics.html");
const TPL_DATA_MANAGEMENT: &str = include_str!("web/templates/data_management.html");
const TPL_LOGIN: &str = include_str!("web/templates/login.html");
// HTMX partials
const TPL_HTMX_DASHBOARD_METRICS: &str = include_str!("web/templates/htmx/dashboard_metrics.html");
const TPL_HTMX_PROVIDERS_LIST: &str = include_str!("web/templates/htmx/providers_list.html");
const TPL_HTMX_INDEXES_LIST: &str = include_str!("web/templates/htmx/indexes_list.html");
const TPL_HTMX_SUBSYSTEMS_LIST: &str = include_str!("web/templates/htmx/subsystems_list.html");
const TPL_HTMX_CONFIG_DIFF: &str = include_str!("web/templates/htmx/config_diff.html");

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
            .route("/admin.css", get(css_handler));

        // Merge routes
        protected_routes.merge(public_routes).with_state(state)
    }
}

// --- Handlers ---

async fn dashboard_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("page", "dashboard");
    render_template(&state.templates, "dashboard.html", &context)
}

async fn providers_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("page", "providers");
    render_template(&state.templates, "providers.html", &context)
}

async fn indexes_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("page", "indexes");
    render_template(&state.templates, "indexes.html", &context)
}

async fn configuration_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("page", "config");
    render_template(&state.templates, "configuration.html", &context)
}

async fn logs_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("page", "logs");
    render_template(&state.templates, "logs.html", &context)
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
    let css = include_str!("web/templates/admin.css");
    Response::builder()
        .header("Content-Type", "text/css")
        .body(css.to_string())
        .map_err(|e| {
            tracing::error!("Failed to build CSS response: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

// --- HTMX Handlers ---

async fn htmx_dashboard_metrics_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let context = Context::new();
    // In a real app, we'd fetch actual metrics here
    render_template(&state.templates, "htmx/dashboard_metrics.html", &context)
}

async fn htmx_providers_list_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let context = Context::new();
    render_template(&state.templates, "htmx/providers_list.html", &context)
}

async fn htmx_indexes_list_handler(State(state): State<AdminState>) -> impl IntoResponse {
    let context = Context::new();
    render_template(&state.templates, "htmx/indexes_list.html", &context)
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
