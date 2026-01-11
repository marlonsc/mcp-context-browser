//! Basic web interface for admin operations

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;
use tera::{Context, Tera};

use crate::admin::models::AdminState;

/// Web interface manager
pub struct WebInterface {
    templates: Arc<Tera>,
}

impl WebInterface {
    /// Create a new web interface manager
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize Tera templates
        let tera = Tera::new("src/admin/web/templates/**/*.html")?;

        // Also register CSS as a template if needed, or we can serve it directly
        // For simplicity, we'll serve it via a dedicated route if it's in templates

        Ok(Self {
            templates: Arc::new(tera),
        })
    }

    /// Get the templates instance
    pub fn templates(&self) -> Arc<Tera> {
        Arc::clone(&self.templates)
    }

    /// Get the web routes
    pub fn routes(&self, state: AdminState) -> Router {
        Router::new()
            .route("/", get(dashboard_handler))
            .route("/dashboard", get(dashboard_handler))
            .route("/providers", get(providers_handler))
            .route("/indexes", get(indexes_handler))
            .route("/config", get(configuration_handler))
            .route("/logs", get(logs_handler))
            .route("/maintenance", get(maintenance_handler))
            .route("/diagnostics", get(diagnostics_handler))
            .route("/data", get(data_management_handler))
            .route("/login", get(login_handler))
            .route("/admin.css", get(css_handler))
            // HTMX partials
            .route(
                "/htmx/dashboard-metrics",
                get(htmx_dashboard_metrics_handler),
            )
            .route("/htmx/providers-list", get(htmx_providers_list_handler))
            .route("/htmx/indexes-list", get(htmx_indexes_list_handler))
            .with_state(state)
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
    // "indexes.html" was not in the list but "indexes_list.html" was in htmx/
    // Let's assume there is a top-level page or we use dashboard
    render_template(&state.templates, "dashboard.html", &context)
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
        .unwrap()
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
