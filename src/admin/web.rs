//! Web interface module using Tera templates and HTMX
//!
//! This module provides a complete web interface generated server-side
//! using Tera templates and HTMX for dynamic interactions.

use std::sync::Arc;
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tera::{Context, Tera, Function as TeraFunction};
use async_trait::async_trait;

use crate::admin::models::{AdminState, ApiResponse};
use crate::admin::service::{AdminService, AdminServiceImpl};
use crate::admin::{AdminApi, AdminConfig};
use crate::metrics::http_server::{HealthResponse, MetricsApiServer};

/// Trait para componentes web reutilizáveis
#[async_trait]
pub trait WebComponent {
    /// Nome do componente
    fn name(&self) -> &'static str;

    /// Renderiza o componente com dados específicos
    async fn render(&self, context: &mut Context) -> crate::core::error::Result<String>;
}

/// Gerenciador de componentes web
pub struct ComponentManager {
    components: HashMap<&'static str, Box<dyn WebComponent + Send + Sync>>,
}

impl ComponentManager {
    pub fn new() -> Self {
        let mut manager = Self {
            components: HashMap::new(),
        };

        // Registrar componentes padrão
        manager.register(Box::new(MetricCardComponent));
        manager.register(Box::new(ProviderCardComponent));
        manager.register(Box::new(NotificationComponent));

        manager
    }

    pub fn register(&mut self, component: Box<dyn WebComponent + Send + Sync>) {
        self.components.insert(component.name(), component);
    }

    pub async fn render_component(
        &self,
        name: &str,
        context: &mut Context,
    ) -> crate::core::error::Result<String> {
        if let Some(component) = self.components.get(name) {
            component.render(context).await
        } else {
            Err(format!("Component '{}' not found", name).into())
        }
    }
}

/// Componente de cartão de métrica
pub struct MetricCardComponent;

#[async_trait]
impl WebComponent for MetricCardComponent {
    fn name(&self) -> &'static str {
        "metric_card"
    }

    async fn render(&self, context: &mut Context) -> crate::core::error::Result<String> {
        // Dados do componente
        let icon = context.get("icon").and_then(|v| v.as_str()).unwrap_or("");
        let title = context.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let value = context.get("value").and_then(|v| v.as_str()).unwrap_or("");
        let subtitle = context.get("subtitle").and_then(|v| v.as_str()).unwrap_or("");
        let color = context.get("color").and_then(|v| v.as_str()).unwrap_or("blue");

        let html = format!(r#"
<div class="bg-white dark:bg-gray-800 overflow-hidden shadow-sm rounded-lg border border-gray-200 dark:border-gray-700 hover:shadow-md transition-all duration-200">
    <div class="p-6">
        <div class="flex items-center">
            <div class="flex-shrink-0">
                <div class="w-12 h-12 bg-{}-100 dark:bg-{}-900 rounded-xl flex items-center justify-center">
                    <svg class="w-6 h-6 text-{}-600 dark:text-{}-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="{}"></path>
                    </svg>
                </div>
            </div>
            <div class="ml-5 w-0 flex-1">
                <dl>
                    <dt class="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">{}</dt>
                    <dd class="text-3xl font-bold text-gray-900 dark:text-white">{}</dd>
                    <dd class="text-sm text-gray-500 dark:text-gray-400 mt-1">{}</dd>
                </dl>
            </div>
        </div>
    </div>
</div>"#, color, color, color, color, icon, title, value, subtitle);

        Ok(html)
    }
}

/// Componente de cartão de provider
pub struct ProviderCardComponent;

#[async_trait]
impl WebComponent for ProviderCardComponent {
    fn name(&self) -> &'static str {
        "provider_card"
    }

    async fn render(&self, context: &mut Context) -> crate::core::error::Result<String> {
        let provider = context.get("provider").and_then(|v| v.as_object());

        if let Some(provider) = provider {
            let id = provider.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let name = provider.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let provider_type = provider.get("provider_type").and_then(|v| v.as_str()).unwrap_or("");
            let status = provider.get("status").and_then(|v| v.as_str()).unwrap_or("");

            let status_classes = match status {
                "active" => "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300",
                "inactive" => "bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-300",
                _ => "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300"
            };

            let html = format!(r#"
<div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6 hover:shadow-lg hover:-translate-y-0.5 transition-all duration-200">
    <div class="flex items-start justify-between">
        <div class="flex items-start space-x-4">
            <div class="flex-shrink-0">
                <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-blue-500 via-purple-500 to-pink-500 flex items-center justify-center">
                    <svg class="w-6 h-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"></path>
                    </svg>
                </div>
            </div>
            <div class="min-w-0 flex-1">
                <div class="flex items-center justify-between">
                    <h3 class="text-lg font-semibold text-gray-900 dark:text-white truncate">{}</h3>
                    <span class="inline-flex items-center px-3 py-1 rounded-full text-xs font-medium {} ml-2">
                        {}
                    </span>
                </div>
                <p class="text-sm text-gray-500 dark:text-gray-400 mt-1 capitalize">{}</p>
                <div class="mt-3 flex items-center space-x-3">
                    <button class="inline-flex items-center px-3 py-1.5 border border-gray-300 dark:border-gray-600 rounded-lg text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors duration-200">
                        <svg class="w-4 h-4 mr-1.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path>
                        </svg>
                        Configure
                    </button>
                    <button class="inline-flex items-center px-3 py-1.5 border border-red-300 dark:border-red-600 rounded-lg text-sm font-medium text-red-700 dark:text-red-300 bg-white dark:bg-gray-800 hover:bg-red-50 dark:hover:bg-red-900 transition-colors duration-200">
                        <svg class="w-4 h-4 mr-1.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
                        </svg>
                        Remove
                    </button>
                </div>
            </div>
        </div>
    </div>
</div>"#, name, status_classes, status, provider_type);

            Ok(html)
        } else {
            Err("Provider data not found in context".into())
        }
    }
}

/// Componente de notificação
pub struct NotificationComponent;

#[async_trait]
impl WebComponent for NotificationComponent {
    fn name(&self) -> &'static str {
        "notification"
    }

    async fn render(&self, context: &mut Context) -> crate::core::error::Result<String> {
        let message = context.get("message").and_then(|v| v.as_str()).unwrap_or("");
        let type_ = context.get("type").and_then(|v| v.as_str()).unwrap_or("info");

        let (bg_class, text_class, icon) = match type_ {
            "success" => ("bg-green-50 border-green-200", "text-green-800", "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"),
            "error" => ("bg-red-50 border-red-200", "text-red-800", "M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"),
            "warning" => ("bg-yellow-50 border-yellow-200", "text-yellow-800", "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"),
            _ => ("bg-blue-50 border-blue-200", "text-blue-800", "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z")
        };

        let html = format!(r#"
<div class="fixed top-4 right-4 z-50 max-w-sm w-full {} border rounded-lg p-4 shadow-lg transform transition-all duration-300 ease-in-out" x-data="{{ show: true }}" x-show="show" x-transition:enter="translate-x-0 opacity-100" x-transition:enter-start="translate-x-full opacity-0" x-transition:leave="translate-x-full opacity-0" x-transition:leave-end="translate-x-0 opacity-100" style="display: none;">
    <div class="flex items-start">
        <div class="flex-shrink-0">
            <svg class="h-6 w-6 {}" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="{}"></path>
            </svg>
        </div>
        <div class="ml-3 w-0 flex-1 pt-0.5">
            <p class="text-sm font-medium {}">{}</p>
        </div>
        <div class="ml-4 flex-shrink-0 flex">
            <button @click="show = false" class="inline-flex {} hover:{} focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-{}-500 rounded-md p-1.5">
                <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                </svg>
            </button>
        </div>
    </div>
</div>"#, bg_class, text_class, icon, text_class, message, text_class, text_class, type_);

        Ok(html)
    }
}
use crate::core::error::Result;

/// Web interface manager with advanced components
pub struct WebInterface {
    tera: Tera,
    component_manager: ComponentManager,
    theme_manager: ThemeManager,
    admin_service: Arc<dyn crate::admin::service::AdminService>,
}

impl WebInterface {
    /// Create a new web interface manager
    pub fn new() -> Result<Self> {
        let mut tera = Tera::new("src/admin/web/templates/**/*.html")?;

        // Add custom functions and filters
        tera.register_function("format_bytes", format_bytes);
        tera.register_function("format_duration", format_duration);
        tera.register_function("format_percentage", format_percentage);
        tera.register_function("render_component", render_component_function);

        // Add custom filters
        tera.register_filter("number_format", number_format);
        tera.register_filter("theme_class", theme_class_filter);

        // Initialize managers
        let component_manager = ComponentManager::new();
        let theme_manager = ThemeManager::new();

        // Create admin service - will be injected at runtime
        let admin_service: Arc<dyn AdminService> = Arc::new(crate::admin::service::AdminServiceImpl::new(std::sync::Arc::new(crate::server::McpServer::new(None).unwrap())));

        Ok(Self {
            tera,
            component_manager,
            theme_manager,
            admin_service,
        })
    }

    /// Get the web routes
    pub fn routes(&self, mcp_server: Arc<crate::server::McpServer>) -> Router {
        let admin_api = Arc::new(AdminApi::new(AdminConfig::default()));
        let admin_service: Arc<dyn AdminService> = Arc::new(AdminServiceImpl::new(Arc::clone(&mcp_server)));
        let state = AdminState {
            admin_api,
            admin_service,
            mcp_server,
        };
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
            .route("/static/admin.css", get(Self::admin_css))
            .route("/static/admin.js", get(Self::admin_js))
            .with_state(state)
    }

    /// Render a template with context
    fn render(&self, template: &str, context: &Context) -> Result<String> {
        Ok(self.tera.render(template, context)?)
    }

    // Page handlers

    async fn dashboard_page(State(state): State<AdminState>) -> impl IntoResponse {
        let interface = WebInterface::new().unwrap();

        // Get dashboard data through admin service (real data)
        let dashboard_data = match interface.admin_service.get_dashboard_data().await {
            Ok(data) => data,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to get dashboard data: {}", e),
                )
                    .into_response();
            }
        };

        // Create health response
        let health = HealthResponse {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            service: "mcp-context-browser".to_string(),
            version: dashboard_data.system_info.version,
            uptime: dashboard_data.system_info.uptime,
            pid: dashboard_data.system_info.pid,
            status: "healthy".to_string(),
        };

        let mut context = Context::new();
        context.insert("title", "Dashboard");
        context.insert("page", "dashboard");
        context.insert("health", &health);
        context.insert("active_providers", &dashboard_data.active_providers);
        context.insert("total_providers", &dashboard_data.total_providers);
        context.insert("active_indexes", &dashboard_data.active_indexes);
        context.insert("total_documents", &dashboard_data.total_documents);
        context.insert("cpu_usage", &format!("{:.1}", dashboard_data.cpu_usage));
        context.insert("memory_usage", &format!("{:.1}", dashboard_data.memory_usage));

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

        // Get real provider data
        let providers = state.mcp_server.get_registered_providers();

        let mut context = Context::new();
        context.insert("title", "Providers");
        context.insert("page", "providers");
        context.insert("providers", &providers);

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

        // Get real indexing data
        let indexing_status = state.mcp_server.get_indexing_status_admin();

        let indexes = vec![serde_json::json!({
            "id": "main-index",
            "name": "Main Codebase Index",
            "status": if indexing_status.is_indexing { "indexing" } else { "active" },
            "document_count": indexing_status.indexed_documents,
            "created_at": indexing_status.start_time.unwrap_or(1640995200),
            "updated_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        })];

        let mut context = Context::new();
        context.insert("title", "Indexes");
        context.insert("page", "indexes");
        context.insert("indexes", &indexes);

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

        // Get real system metrics
        let system_info = state.mcp_server.get_system_info();
        let health = HealthResponse {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            service: "mcp-context-browser".to_string(),
            version: system_info.version,
            uptime: system_info.uptime,
            pid: system_info.pid,
            status: "healthy".to_string(),
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

        // Get providers through admin service (real data)
        let providers = match interface.admin_service.get_providers().await {
            Ok(providers) => providers,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to get providers: {}", e),
                )
                    .into_response();
            }
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

        // Get indexing status through admin service (real data)
        let indexing_status = match interface.admin_service.get_indexing_status().await {
            Ok(status) => status,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to get indexing status: {}", e),
                )
                    .into_response();
            }
        };

        let indexes = vec![
            crate::admin::models::IndexInfo {
                id: "main-index".to_string(),
                name: "Main Codebase Index".to_string(),
                status: if indexing_status.is_indexing { "indexing".to_string() } else { "active".to_string() },
                document_count: indexing_status.indexed_documents,
                created_at: indexing_status.start_time.unwrap_or(1640995200),
                updated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            },
        ];

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

    async fn admin_css() -> impl IntoResponse {
        let css = include_str!("web/templates/admin.css");

        (
            [
                (header::CONTENT_TYPE, "text/css"),
                (header::CACHE_CONTROL, "public, max-age=3600"),
            ],
            css,
        )
    }

    async fn admin_js() -> impl IntoResponse {
        let js = r#"
// Admin interface JavaScript
console.log('MCP Context Browser Admin Interface loaded');
"#;

        (
            [
                (header::CONTENT_TYPE, "application/javascript"),
                (header::CACHE_CONTROL, "public, max-age=3600"),
            ],
            js,
        )
    }
}

/// Gerenciador de temas avançado
pub struct ThemeManager {
    themes: HashMap<String, ThemeConfig>,
}

#[derive(Clone, Debug)]
pub struct ThemeConfig {
    pub name: String,
    pub colors: HashMap<String, String>,
    pub fonts: HashMap<String, String>,
    pub spacing: HashMap<String, String>,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut themes = HashMap::new();

        // Tema padrão (light)
        let mut light_theme = ThemeConfig {
            name: "light".to_string(),
            colors: HashMap::new(),
            fonts: HashMap::new(),
            spacing: HashMap::new(),
        };

        light_theme.colors.insert("bg-primary".to_string(), "white".to_string());
        light_theme.colors.insert("bg-secondary".to_string(), "gray-50".to_string());
        light_theme.colors.insert("text-primary".to_string(), "gray-900".to_string());
        light_theme.colors.insert("text-secondary".to_string(), "gray-600".to_string());
        light_theme.colors.insert("border".to_string(), "gray-200".to_string());

        // Tema dark
        let mut dark_theme = ThemeConfig {
            name: "dark".to_string(),
            colors: HashMap::new(),
            fonts: HashMap::new(),
            spacing: HashMap::new(),
        };

        dark_theme.colors.insert("bg-primary".to_string(), "gray-900".to_string());
        dark_theme.colors.insert("bg-secondary".to_string(), "gray-800".to_string());
        dark_theme.colors.insert("text-primary".to_string(), "white".to_string());
        dark_theme.colors.insert("text-secondary".to_string(), "gray-300".to_string());
        dark_theme.colors.insert("border".to_string(), "gray-700".to_string());

        themes.insert("light".to_string(), light_theme);
        themes.insert("dark".to_string(), dark_theme);

        Self { themes }
    }

    pub fn get_theme(&self, name: &str) -> Option<&ThemeConfig> {
        self.themes.get(name)
    }

    pub fn get_css_variables(&self, theme_name: &str) -> String {
        if let Some(theme) = self.themes.get(theme_name) {
            let mut css = String::new();
            for (key, value) in &theme.colors {
                css.push_str(&format!("--color-{}: {};", key, value));
            }
            css
        } else {
            String::new()
        }
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

fn number_format(value: &serde_json::Value, _args: &HashMap<String, serde_json::Value>) -> tera::Result<serde_json::Value> {
    let num = match value {
        serde_json::Value::Number(n) => n.as_u64().unwrap_or(0),
        serde_json::Value::String(s) => s.parse().unwrap_or(0),
        _ => 0,
    };

    Ok(serde_json::Value::String(num.to_string()))
}

fn render_component_function(args: &HashMap<String, serde_json::Value>) -> tera::Result<serde_json::Value> {
    let component_name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::msg("Component name is required"))?;

    // This would need access to the ComponentManager instance
    // For now, return a placeholder
    Ok(serde_json::Value::String(format!("<!-- Component: {} -->", component_name)))
}

fn theme_class_filter(value: &serde_json::Value, args: &HashMap<String, serde_json::Value>) -> tera::Result<serde_json::Value> {
    let theme = args
        .get("theme")
        .and_then(|v| v.as_str())
        .unwrap_or("light");

    let class_name = value.as_str().unwrap_or("");

    // Apply theme-specific modifications
    let themed_class = match theme {
        "dark" => class_name.replace("bg-white", "bg-gray-800")
                           .replace("text-gray-900", "text-white")
                           .replace("border-gray-200", "border-gray-700"),
        _ => class_name.to_string(),
    };

    Ok(serde_json::Value::String(themed_class))
}