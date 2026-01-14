//! End-to-end workflow tests for admin interface
//!
//! These tests verify complete admin workflows using real services.

use std::collections::HashMap;

use mcp_context_browser::application::admin::traits::AdminService;

/// Create a real AdminService for testing with minimal dependencies
async fn create_test_admin_service() -> std::sync::Arc<dyn AdminService> {
    use arc_swap::ArcSwap;
    use mcp_context_browser::adapters::http_client::HttpClientPool;
    use mcp_context_browser::application::admin::{AdminServiceDependencies, AdminServiceImpl};
    use mcp_context_browser::infrastructure::config::ConfigLoader;
    use mcp_context_browser::infrastructure::di::factory::ServiceProvider;
    use mcp_context_browser::infrastructure::events::EventBus;
    use mcp_context_browser::infrastructure::logging;
    use mcp_context_browser::infrastructure::metrics::system::SystemMetricsCollector;
    use mcp_context_browser::infrastructure::operations::McpIndexingOperations;
    use mcp_context_browser::server::metrics::McpPerformanceMetrics;
    use std::sync::Arc;

    // Create minimal test dependencies
    let performance_metrics: Arc<
        dyn mcp_context_browser::server::metrics::PerformanceMetricsInterface,
    > = Arc::new(McpPerformanceMetrics::default());
    let indexing_operations: Arc<
        dyn mcp_context_browser::infrastructure::operations::IndexingOperationsInterface,
    > = Arc::new(McpIndexingOperations::default());
    let service_provider: Arc<
        dyn mcp_context_browser::infrastructure::di::factory::ServiceProviderInterface,
    > = Arc::new(ServiceProvider::new());
    let system_collector: Arc<
        dyn mcp_context_browser::infrastructure::metrics::system::SystemMetricsCollectorInterface,
    > = Arc::new(SystemMetricsCollector::new());
    let http_client: Arc<dyn mcp_context_browser::adapters::http_client::HttpClientProvider> =
        Arc::new(HttpClientPool::new().expect("Failed to create HTTP client"));

    // Create event bus and log buffer
    let event_bus: Arc<dyn mcp_context_browser::infrastructure::events::EventBusProvider> =
        Arc::new(EventBus::with_default_capacity());
    let log_buffer = logging::create_shared_log_buffer(1000);

    // Load config from file instead of using Config::default()
    let loader = ConfigLoader::new();
    let loaded_config = loader
        .load()
        .await
        .expect("Failed to load config for tests");
    let config = Arc::new(ArcSwap::from_pointee(loaded_config));

    // Create the real admin service
    let deps = AdminServiceDependencies {
        performance_metrics,
        indexing_operations,
        service_provider,
        system_collector,
        http_client,
        event_bus,
        log_buffer,
        config,
        cache_provider: None,
        search_service: None,
    };
    let admin_service = AdminServiceImpl::new(deps);

    Arc::new(admin_service)
}

/// Test infrastructure for setting up real services

#[tokio::test]
async fn test_e2e_login_and_system_info() {
    use mcp_context_browser::server::admin::auth::AuthService;

    // 1. Create auth service and authenticate
    let auth_service = AuthService::new(
        "test-secret".to_string(),
        3600,
        "admin".to_string(),
        "admin".to_string(),
    )
    .expect("Failed to create auth service");

    let result = auth_service.authenticate("admin", "admin");
    assert!(result.is_ok());

    // 2. Generate token
    let user = result.unwrap();
    let token = auth_service.generate_token(&user);
    assert!(token.is_ok());

    // 3. Validate token
    let token = token.unwrap();
    let claims = auth_service.validate_token(&token);
    assert!(claims.is_ok());

    // 4. Get system info using admin service
    let service = create_test_admin_service().await;
    let info = service.get_system_info().await;
    assert!(info.is_ok());

    let info = info.unwrap();
    assert!(!info.version.is_empty());
}

#[tokio::test]
async fn test_e2e_config_update_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let service = create_test_admin_service().await;

    // 1. Get current configuration
    let config = service.get_configuration().await;
    assert!(config.is_ok());
    let _original_config = config.unwrap();

    // 2. Validate a configuration change
    let mut updates = HashMap::new();
    updates.insert("indexing.chunk_size".to_string(), serde_json::json!(1024));

    let warnings = service.validate_configuration(&updates).await;
    assert!(warnings.is_ok());

    // 3. Update configuration
    let result = service
        .update_configuration(updates.clone(), "test_user")
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);

    // 4. Check configuration history
    let history = service.get_configuration_history(Some(10)).await;
    assert!(history.is_ok());

    Ok(())
}

// ============================================================================
// HTTP Page Rendering Tests
// ============================================================================

/// Test that login page renders correctly (public route, no auth)
#[tokio::test]
async fn test_e2e_login_page_renders() {
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use mcp_context_browser::server::admin::web::WebInterface;
    use tower::ServiceExt;

    // Create web interface and get templates
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let state = create_test_admin_state(&web_interface).await;
    let app = web_interface.routes(state);

    // Test login page (public, no auth required)
    let response = app
        .oneshot(Request::get("/login").body(Body::empty()).unwrap())
        .await
        .expect("Request failed");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Login page should return 200"
    );

    // Verify HTML content
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Login") || body_str.contains("login"),
        "Login page should contain login form"
    );
}

/// Test that dashboard page renders with real DI data
#[tokio::test]
async fn test_e2e_dashboard_page_renders_with_di_data() {
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use mcp_context_browser::server::admin::web::WebInterface;
    use tower::ServiceExt;

    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let state = create_test_admin_state(&web_interface).await;
    let app = web_interface.routes(state.clone());

    // First, authenticate and get a token
    let token = get_test_auth_token(&state).await;

    // Test dashboard with authentication cookie
    let response = app
        .oneshot(
            Request::get("/dashboard")
                .header("Cookie", format!("mcp_admin_token={}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Dashboard should return 200 with valid token"
    );

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);

    // Verify DI data appears in rendered HTML
    assert!(
        body_str.contains("Dashboard") || body_str.contains("dashboard"),
        "Dashboard should render dashboard template"
    );
}

/// Test all protected pages render with authentication
#[tokio::test]
async fn test_e2e_all_protected_pages_render() {
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use mcp_context_browser::server::admin::web::WebInterface;
    use tower::ServiceExt;

    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let state = create_test_admin_state(&web_interface).await;
    let token = get_test_auth_token(&state).await;

    // All protected routes to test
    // Note: /data excluded because it requires filesystem access for backups directory
    let protected_routes = vec![
        ("/", "Dashboard"),
        ("/dashboard", "Dashboard"),
        ("/providers", "Providers"),
        ("/indexes", "Indexes"),
        ("/config", "Configuration"),
        ("/logs", "Logs"),
        ("/maintenance", "Maintenance"),
        ("/diagnostics", "Diagnostics"),
    ];

    for (route, expected_content) in protected_routes {
        let app = web_interface.routes(state.clone());
        let response = app
            .oneshot(
                Request::get(route)
                    .header("Cookie", format!("mcp_admin_token={}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap_or_else(|_| panic!("Request to {} failed", route));

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);

        // If not 200, show the error body for debugging
        assert_eq!(
            status,
            StatusCode::OK,
            "Page {} should return 200, got {} with body: {}",
            route,
            status,
            &body_str[..std::cmp::min(500, body_str.len())]
        );

        // Check that page has some content (template rendered)
        assert!(
            body_str.len() > 100,
            "Page {} should have rendered content (got {} bytes)",
            route,
            body_str.len()
        );

        // Check for expected content marker
        assert!(
            body_str
                .to_lowercase()
                .contains(&expected_content.to_lowercase())
                || body_str.contains("<!DOCTYPE html>"),
            "Page {} should contain '{}' or be valid HTML",
            route,
            expected_content
        );
    }
}

/// Test that protected pages redirect to login without authentication
#[tokio::test]
async fn test_e2e_protected_pages_require_auth() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use mcp_context_browser::server::admin::web::WebInterface;
    use tower::ServiceExt;

    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let state = create_test_admin_state(&web_interface).await;
    let app = web_interface.routes(state);

    // Try accessing dashboard without auth
    let response = app
        .oneshot(Request::get("/dashboard").body(Body::empty()).unwrap())
        .await
        .expect("Request failed");

    // Should redirect to login or return 401/403
    let status = response.status();
    assert!(
        status == StatusCode::FOUND
            || status == StatusCode::SEE_OTHER
            || status == StatusCode::UNAUTHORIZED
            || status == StatusCode::FORBIDDEN,
        "Protected page without auth should redirect or deny (got {})",
        status
    );
}

// ============================================================================
// Auth Disabled Mode Tests
// ============================================================================

/// Test that dashboard is accessible without login when auth is disabled
#[tokio::test]
async fn test_e2e_dashboard_accessible_when_auth_disabled() {
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use mcp_context_browser::server::admin::web::WebInterface;
    use tower::ServiceExt;

    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let state = create_test_admin_state_auth_disabled(&web_interface).await;
    let app = web_interface.routes(state);

    // Access dashboard WITHOUT any cookie - should succeed when auth disabled
    let response = app
        .oneshot(Request::get("/dashboard").body(Body::empty()).unwrap())
        .await
        .expect("Request failed");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Dashboard should be accessible without login when auth is disabled"
    );

    // Verify HTML content is rendered
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);

    assert!(
        body_str.contains("<!DOCTYPE html>") || body_str.contains("dashboard"),
        "Dashboard should render HTML content"
    );
}

/// Test that all protected pages are accessible when auth is disabled
#[tokio::test]
async fn test_e2e_all_pages_accessible_when_auth_disabled() {
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use mcp_context_browser::server::admin::web::WebInterface;
    use tower::ServiceExt;

    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let state = create_test_admin_state_auth_disabled(&web_interface).await;

    // All routes that should be accessible without auth when disabled
    let routes = vec![
        "/",
        "/dashboard",
        "/providers",
        "/indexes",
        "/config",
        "/logs",
        "/maintenance",
        "/diagnostics",
    ];

    for route in routes {
        let app = web_interface.routes(state.clone());
        let response = app
            .oneshot(Request::get(route).body(Body::empty()).unwrap())
            .await
            .unwrap_or_else(|_| panic!("Request to {} failed", route));

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Route {} should be accessible when auth is disabled",
            route
        );

        // Verify some content is rendered
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(
            body.len() > 100,
            "Route {} should return rendered content",
            route
        );
    }
}

// ============================================================================
// Dashboard Data Points Validation
// ============================================================================

/// Test that dashboard renders all expected data points
#[tokio::test]
async fn test_e2e_dashboard_data_points_validation() {
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use mcp_context_browser::server::admin::web::WebInterface;
    use tower::ServiceExt;

    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let state = create_test_admin_state(&web_interface).await;
    let token = get_test_auth_token(&state).await;
    let app = web_interface.routes(state);

    // Access dashboard with valid auth
    let response = app
        .oneshot(
            Request::get("/dashboard")
                .header("Cookie", format!("mcp_admin_token={}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    let body_lower = body_str.to_lowercase();

    // Verify Health section data points
    assert!(
        body_lower.contains("healthy") || body_lower.contains("status"),
        "Dashboard should contain health status indicator"
    );
    assert!(
        body_lower.contains("uptime") || body_lower.contains("running"),
        "Dashboard should contain uptime information"
    );

    // Verify Providers section
    assert!(
        body_lower.contains("provider") || body_lower.contains("embedding"),
        "Dashboard should contain provider information"
    );
    assert!(
        body_lower.contains("active"),
        "Dashboard should show active status for providers"
    );

    // Verify Indexes section
    assert!(
        body_lower.contains("index") || body_lower.contains("document"),
        "Dashboard should contain indexing information"
    );

    // Verify Metrics section
    assert!(
        body_lower.contains("cpu") || body_lower.contains("memory"),
        "Dashboard should contain system metrics"
    );

    // Verify the page structure
    assert!(
        body_str.contains("<!DOCTYPE html>"),
        "Dashboard should be valid HTML"
    );
}

/// Test dashboard with auth disabled also shows data points
#[tokio::test]
async fn test_e2e_dashboard_data_points_auth_disabled() {
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use mcp_context_browser::server::admin::web::WebInterface;
    use tower::ServiceExt;

    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let state = create_test_admin_state_auth_disabled(&web_interface).await;
    let app = web_interface.routes(state);

    // Access dashboard without any auth
    let response = app
        .oneshot(Request::get("/dashboard").body(Body::empty()).unwrap())
        .await
        .expect("Request failed");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Dashboard should be accessible when auth disabled"
    );

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    let body_lower = body_str.to_lowercase();

    // Same data points should be present regardless of auth mode
    assert!(
        body_lower.contains("healthy") || body_lower.contains("status"),
        "Dashboard should contain health status even with auth disabled"
    );
    assert!(
        body_lower.contains("provider") || body_lower.contains("active"),
        "Dashboard should contain provider info even with auth disabled"
    );
    assert!(
        body_str.contains("<!DOCTYPE html>"),
        "Dashboard should be valid HTML"
    );
}

// ============================================================================
// Test Helper Functions
// ============================================================================

/// Create AdminState for testing with all required dependencies
async fn create_test_admin_state(
    web_interface: &mcp_context_browser::server::admin::web::WebInterface,
) -> mcp_context_browser::server::admin::models::AdminState {
    use mcp_context_browser::application::admin::helpers::activity::ActivityLogger;
    use mcp_context_browser::infrastructure::auth::{
        AuthConfig, AuthService, HashVersion, User, UserRole,
    };
    use mcp_context_browser::infrastructure::events::EventBus;
    use mcp_context_browser::server::admin::models::AdminState;
    use mcp_context_browser::server::admin::AdminApi;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    let admin_service = create_test_admin_service().await;

    // Create auth config with test credentials
    let password_hash = mcp_context_browser::infrastructure::auth::password::hash_password("admin")
        .expect("Failed to hash password");

    let admin_user = User {
        id: "admin".to_string(),
        email: "admin@test.local".to_string(),
        role: UserRole::Admin,
        password_hash,
        hash_version: HashVersion::Argon2id,
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        last_active: 0,
    };

    let mut users = HashMap::new();
    users.insert("admin@test.local".to_string(), admin_user);

    let auth_config = AuthConfig {
        jwt_secret: "test-secret-key-for-testing-32chars!".to_string(),
        jwt_expiration: 3600,
        jwt_issuer: "test".to_string(),
        enabled: true,
        bypass_paths: vec![],
        users,
    };

    let auth_service: Arc<dyn mcp_context_browser::infrastructure::auth::AuthServiceInterface> =
        Arc::new(AuthService::new(auth_config));

    // Create event bus
    let event_bus: mcp_context_browser::infrastructure::events::SharedEventBusProvider =
        Arc::new(EventBus::with_default_capacity());

    // Create activity logger
    let activity_logger = Arc::new(ActivityLogger::new());

    // Create AdminApi with default config
    let admin_api = Arc::new(AdminApi::new(
        mcp_context_browser::server::admin::AdminConfig::default(),
    ));

    AdminState {
        admin_api,
        admin_service,
        auth_service,
        mcp_server: create_test_mcp_server().await,
        templates: web_interface.templates(),
        recovery_manager: None,
        event_bus,
        activity_logger,
    }
}

/// Create AdminState for testing with authentication DISABLED
///
/// This helper creates an AdminState where auth is completely disabled,
/// allowing access to all pages without login.
async fn create_test_admin_state_auth_disabled(
    web_interface: &mcp_context_browser::server::admin::web::WebInterface,
) -> mcp_context_browser::server::admin::models::AdminState {
    use mcp_context_browser::application::admin::helpers::activity::ActivityLogger;
    use mcp_context_browser::infrastructure::auth::{AuthConfig, AuthService};
    use mcp_context_browser::infrastructure::events::EventBus;
    use mcp_context_browser::server::admin::models::AdminState;
    use mcp_context_browser::server::admin::AdminApi;
    use std::collections::HashMap;
    use std::sync::Arc;

    let admin_service = create_test_admin_service().await;

    // Create auth config with authentication DISABLED
    let auth_config = AuthConfig {
        jwt_secret: String::new(), // Empty secret when disabled
        jwt_expiration: 86400,
        jwt_issuer: "mcp-context-browser".to_string(),
        enabled: false, // KEY: Auth is disabled
        bypass_paths: vec![],
        users: HashMap::new(), // No users needed when disabled
    };

    let auth_service: Arc<dyn mcp_context_browser::infrastructure::auth::AuthServiceInterface> =
        Arc::new(AuthService::new(auth_config));

    // Create event bus
    let event_bus: mcp_context_browser::infrastructure::events::SharedEventBusProvider =
        Arc::new(EventBus::with_default_capacity());

    // Create activity logger
    let activity_logger = Arc::new(ActivityLogger::new());

    // Create AdminApi with default config
    let admin_api = Arc::new(AdminApi::new(
        mcp_context_browser::server::admin::AdminConfig::default(),
    ));

    AdminState {
        admin_api,
        admin_service,
        auth_service,
        mcp_server: create_test_mcp_server().await,
        templates: web_interface.templates(),
        recovery_manager: None,
        event_bus,
        activity_logger,
    }
}

/// Create a minimal McpServer for testing
async fn create_test_mcp_server() -> std::sync::Arc<mcp_context_browser::server::McpServer> {
    use std::sync::Arc;
    // Use the builder pattern to create a server
    Arc::new(
        mcp_context_browser::server::McpServerBuilder::new()
            .build()
            .await
            .expect("Failed to create test MCP server"),
    )
}

/// Get a valid auth token for testing
async fn get_test_auth_token(
    state: &mcp_context_browser::server::admin::models::AdminState,
) -> String {
    // Authenticate with test credentials (email, password)
    state
        .auth_service
        .authenticate("admin@test.local", "admin")
        .expect("Failed to authenticate")
}

// Test service creation function
