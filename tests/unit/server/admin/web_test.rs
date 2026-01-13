//! Web interface integration tests
//!
//! Comprehensive tests for the admin web interface including:
//! - Template loading and rendering
//! - View model creation
//! - Link validation
//! - API endpoint consistency

use mcp_context_browser::server::admin::web::builders::ViewModelBuilder;
use mcp_context_browser::server::admin::web::view_models::*;
use mcp_context_browser::server::admin::web::WebInterface;
use tera::Context;

// ============================================================================
// Web Interface Creation Tests
// ============================================================================

#[test]
fn test_web_interface_creation() {
    let web_interface = WebInterface::new();
    assert!(
        web_interface.is_ok(),
        "WebInterface should be created successfully"
    );
}

#[test]
fn test_web_interface_templates_loaded() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    // Verify all templates are loaded
    let template_names = templates.get_template_names().collect::<Vec<_>>();

    let expected_templates = vec![
        "base.html",
        "dashboard.html",
        "providers.html",
        "indexes.html",
        "configuration.html",
        "logs.html",
        "maintenance.html",
        "diagnostics.html",
        "data_management.html",
        "login.html",
        "icons.html",
        "error.html",
        "htmx/dashboard_metrics.html",
        "htmx/providers_list.html",
        "htmx/indexes_list.html",
        "htmx/subsystems_list.html",
        "htmx/config_diff.html",
    ];

    for expected in expected_templates {
        assert!(
            template_names.contains(&expected),
            "Template '{}' should be loaded",
            expected
        );
    }
}

// ============================================================================
// Template Rendering Tests
// ============================================================================

#[test]
fn test_dashboard_renders_with_minimal_data() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let dashboard_vm = DashboardViewModel {
        page: "dashboard",
        metrics: MetricsViewModel::new(0.0, 0.0, 0, 0.0),
        providers: ProvidersViewModel::new(vec![]),
        indexes: IndexesSummaryViewModel {
            active_count: 0,
            total_documents: 0,
            total_documents_formatted: "0".to_string(),
            is_indexing: false,
        },
        activities: vec![],
        system_health: HealthViewModel::new("healthy", 0, 1),
    };

    let vm_json = serde_json::to_string(&dashboard_vm).unwrap();
    let mut context = Context::new();
    context.insert("vm", &dashboard_vm);
    context.insert("vm_json", &vm_json);
    context.insert("page", "dashboard");

    let result = templates.render("dashboard.html", &context);
    assert!(
        result.is_ok(),
        "Dashboard should render with minimal data: {:?}",
        result.err()
    );

    let html = result.unwrap();
    assert!(
        html.contains("System Dashboard"),
        "Dashboard should contain title"
    );
    assert!(
        html.len() > 1000,
        "Dashboard should have substantial content"
    );
}

#[test]
fn test_dashboard_renders_with_full_data() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let dashboard_vm = DashboardViewModel {
        page: "dashboard",
        metrics: MetricsViewModel::new(45.5, 62.3, 1234, 15.7),
        providers: ProvidersViewModel::new(vec![
            ProviderViewModel::new(
                "openai-1".to_string(),
                "OpenAI GPT".to_string(),
                "embedding".to_string(),
                "available".to_string(),
            ),
            ProviderViewModel::new(
                "ollama-1".to_string(),
                "Ollama Local".to_string(),
                "embedding".to_string(),
                "unavailable".to_string(),
            ),
        ]),
        indexes: IndexesSummaryViewModel {
            active_count: 1,
            total_documents: 5000,
            total_documents_formatted: "5,000".to_string(),
            is_indexing: false,
        },
        activities: vec![
            ActivityViewModel::new(
                "act-1".to_string(),
                "Index completed successfully".to_string(),
                chrono::Utc::now(),
                "success",
                "indexing".to_string(),
            ),
            ActivityViewModel::new(
                "act-2".to_string(),
                "Provider health check failed".to_string(),
                chrono::Utc::now(),
                "error",
                "health".to_string(),
            ),
        ],
        system_health: HealthViewModel::new("healthy", 3661, 12345),
    };

    let vm_json = serde_json::to_string(&dashboard_vm).unwrap();
    let mut context = Context::new();
    context.insert("vm", &dashboard_vm);
    context.insert("vm_json", &vm_json);
    context.insert("page", "dashboard");

    let result = templates.render("dashboard.html", &context);
    assert!(
        result.is_ok(),
        "Dashboard should render with full data: {:?}",
        result.err()
    );
}

#[test]
fn test_providers_renders_with_empty_list() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let providers_vm = ProvidersViewModel::new(vec![]);
    let mut context = Context::new();
    context.insert("vm", &providers_vm);
    context.insert("page", "providers");

    let result = templates.render("providers.html", &context);
    assert!(
        result.is_ok(),
        "Providers should render with empty list: {:?}",
        result.err()
    );
}

#[test]
fn test_providers_renders_with_data() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let providers_vm = ProvidersViewModel::new(vec![
        ProviderViewModel::new(
            "openai-1".to_string(),
            "OpenAI".to_string(),
            "embedding".to_string(),
            "available".to_string(),
        ),
        ProviderViewModel::new(
            "milvus-1".to_string(),
            "Milvus DB".to_string(),
            "vector_store".to_string(),
            "error".to_string(),
        ),
    ]);

    let mut context = Context::new();
    context.insert("vm", &providers_vm);
    context.insert("page", "providers");

    let result = templates.render("providers.html", &context);
    assert!(
        result.is_ok(),
        "Providers should render with data: {:?}",
        result.err()
    );
}

#[test]
fn test_indexes_renders_with_empty_list() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let indexes_vm = IndexesViewModel::new(vec![], 0);
    let mut context = Context::new();
    context.insert("vm", &indexes_vm);
    context.insert("page", "indexes");

    let result = templates.render("indexes.html", &context);
    assert!(
        result.is_ok(),
        "Indexes should render with empty list: {:?}",
        result.err()
    );
}

#[test]
fn test_indexes_renders_with_data() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let indexes_vm = IndexesViewModel::new(
        vec![
            IndexViewModel::new(
                "main-index".to_string(),
                "Main Codebase Index".to_string(),
                "active".to_string(),
                5000,
                1704067200,
                1704153600,
            ),
            IndexViewModel::new(
                "secondary-index".to_string(),
                "Secondary Index".to_string(),
                "indexing".to_string(),
                1000,
                1704067200,
                1704153600,
            ),
        ],
        6000,
    );

    let mut context = Context::new();
    context.insert("vm", &indexes_vm);
    context.insert("page", "indexes");

    let result = templates.render("indexes.html", &context);
    assert!(
        result.is_ok(),
        "Indexes should render with data: {:?}",
        result.err()
    );
}

#[test]
fn test_configuration_renders() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let config_vm = ConfigurationViewModel {
        page: "config",
        page_description: "Manage system settings",
        categories: vec![
            ConfigCategoryViewModel {
                name: "Indexing".to_string(),
                description: "Indexing settings".to_string(),
                settings: vec![
                    ConfigSettingViewModel {
                        key: "indexing.chunk_size".to_string(),
                        label: "Chunk Size".to_string(),
                        value: serde_json::json!(512),
                        value_display: "512".to_string(),
                        setting_type: "number",
                        description: "Size of chunks".to_string(),
                        editable: true,
                    },
                    ConfigSettingViewModel {
                        key: "indexing.chunk_overlap".to_string(),
                        label: "Chunk Overlap".to_string(),
                        value: serde_json::json!(50),
                        value_display: "50".to_string(),
                        setting_type: "number",
                        description: "Overlap between chunks".to_string(),
                        editable: true,
                    },
                ],
            },
            ConfigCategoryViewModel {
                name: "Security".to_string(),
                description: "Security settings".to_string(),
                settings: vec![ConfigSettingViewModel {
                    key: "security.enable_auth".to_string(),
                    label: "Enable Auth".to_string(),
                    value: serde_json::json!(true),
                    value_display: "Enabled".to_string(),
                    setting_type: "boolean",
                    description: "Enable authentication".to_string(),
                    editable: true,
                }],
            },
        ],
    };

    let mut context = Context::new();
    context.insert("vm", &config_vm);
    context.insert("page", config_vm.page);
    context.insert("page_description", config_vm.page_description);

    let result = templates.render("configuration.html", &context);
    assert!(
        result.is_ok(),
        "Configuration should render: {:?}",
        result.err()
    );
}

#[test]
fn test_logs_renders() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let logs_vm = LogsViewModel {
        page: "logs",
        page_description: "View and filter system logs",
        entries: vec![
            LogEntryViewModel {
                timestamp: "2024-01-01 12:00:00".to_string(),
                level: "INFO".to_string(),
                level_class: "bg-blue-100 text-blue-800",
                message: "Server started".to_string(),
                source: "main".to_string(),
            },
            LogEntryViewModel {
                timestamp: "2024-01-01 12:01:00".to_string(),
                level: "ERROR".to_string(),
                level_class: "bg-red-100 text-red-800",
                message: "Connection failed".to_string(),
                source: "database".to_string(),
            },
        ],
        total_count: 100,
        stats: LogStatsViewModel {
            total: 100,
            errors: 5,
            warnings: 10,
            info: 85,
        },
    };

    let mut context = Context::new();
    context.insert("vm", &logs_vm);
    context.insert("page", logs_vm.page);
    context.insert("page_description", logs_vm.page_description);

    let result = templates.render("logs.html", &context);
    assert!(result.is_ok(), "Logs should render: {:?}", result.err());
}

#[test]
fn test_error_renders() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let error_vm = ViewModelBuilder::build_error("Test Error", "Something went wrong", None);
    let mut context = Context::new();
    context.insert("error", &error_vm);
    context.insert("page", "error");

    let result = templates.render("error.html", &context);
    assert!(result.is_ok(), "Error should render: {:?}", result.err());

    let html = result.unwrap();
    assert!(
        html.contains("Test Error"),
        "Error page should contain error title"
    );
}

#[test]
fn test_error_renders_with_details() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let error_vm = ViewModelBuilder::build_error(
        "Database Error",
        "Connection failed",
        Some("Stack trace: ..."),
    );
    let mut context = Context::new();
    context.insert("error", &error_vm);
    context.insert("page", "error");

    let result = templates.render("error.html", &context);
    assert!(
        result.is_ok(),
        "Error with details should render: {:?}",
        result.err()
    );
}

#[test]
fn test_login_renders() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let context = Context::new();
    let result = templates.render("login.html", &context);
    assert!(result.is_ok(), "Login should render: {:?}", result.err());
}

#[test]
fn test_maintenance_renders() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let mut context = Context::new();
    context.insert("page", "maintenance");
    context.insert("page_description", "System maintenance operations");
    let result = templates.render("maintenance.html", &context);
    assert!(
        result.is_ok(),
        "Maintenance should render: {:?}",
        result.err()
    );
}

#[test]
fn test_diagnostics_renders() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let mut context = Context::new();
    context.insert("page", "diagnostics");
    context.insert(
        "page_description",
        "System health and connectivity diagnostics",
    );
    let result = templates.render("diagnostics.html", &context);
    assert!(
        result.is_ok(),
        "Diagnostics should render: {:?}",
        result.err()
    );
}

#[test]
fn test_data_management_renders() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let mut context = Context::new();
    context.insert("page", "data");
    context.insert("page_description", "Backup and restore operations");
    let result = templates.render("data_management.html", &context);
    assert!(
        result.is_ok(),
        "Data management should render: {:?}",
        result.err()
    );
}

// ============================================================================
// HTMX Partial Tests
// ============================================================================

#[test]
fn test_htmx_dashboard_metrics_renders() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let dashboard_vm = DashboardViewModel {
        page: "dashboard",
        metrics: MetricsViewModel::new(45.5, 62.3, 1234, 15.7),
        providers: ProvidersViewModel::new(vec![]),
        indexes: IndexesSummaryViewModel {
            active_count: 0,
            total_documents: 0,
            total_documents_formatted: "0".to_string(),
            is_indexing: false,
        },
        activities: vec![],
        system_health: HealthViewModel::new("healthy", 0, 1),
    };

    let mut context = Context::new();
    context.insert("vm", &dashboard_vm);

    let result = templates.render("htmx/dashboard_metrics.html", &context);
    assert!(
        result.is_ok(),
        "HTMX dashboard metrics should render: {:?}",
        result.err()
    );
}

#[test]
fn test_htmx_providers_list_renders() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let providers = vec![
        ProviderViewModel::new(
            "openai-1".to_string(),
            "OpenAI".to_string(),
            "embedding".to_string(),
            "available".to_string(),
        ),
        ProviderViewModel::new(
            "milvus-1".to_string(),
            "Milvus".to_string(),
            "vector_store".to_string(),
            "error".to_string(),
        ),
    ];

    let mut context = Context::new();
    context.insert("providers", &providers);

    let result = templates.render("htmx/providers_list.html", &context);
    assert!(
        result.is_ok(),
        "HTMX providers list should render: {:?}",
        result.err()
    );
}

#[test]
fn test_htmx_indexes_list_renders() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let indexes = vec![IndexViewModel::new(
        "main-index".to_string(),
        "Main Index".to_string(),
        "active".to_string(),
        1000,
        1704067200,
        1704153600,
    )];

    let mut context = Context::new();
    context.insert("indexes", &indexes);

    let result = templates.render("htmx/indexes_list.html", &context);
    assert!(
        result.is_ok(),
        "HTMX indexes list should render: {:?}",
        result.err()
    );
}

#[test]
fn test_htmx_subsystems_list_renders() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    // Subsystems list needs subsystems data
    let mut context = Context::new();
    context.insert("subsystems", &Vec::<serde_json::Value>::new());

    let result = templates.render("htmx/subsystems_list.html", &context);
    assert!(
        result.is_ok(),
        "HTMX subsystems list should render: {:?}",
        result.err()
    );
}

#[test]
fn test_htmx_config_diff_renders() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let mut context = Context::new();
    context.insert("changes", &Vec::<serde_json::Value>::new());
    context.insert("has_changes", &false);

    let result = templates.render("htmx/config_diff.html", &context);
    assert!(
        result.is_ok(),
        "HTMX config diff should render: {:?}",
        result.err()
    );
}

// ============================================================================
// Link Validation Tests
// ============================================================================

#[test]
fn test_nav_links_match_routes() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    // Render dashboard template which extends base.html
    let dashboard_vm = DashboardViewModel {
        page: "dashboard",
        metrics: MetricsViewModel::new(0.0, 0.0, 0, 0.0),
        providers: ProvidersViewModel::new(vec![]),
        indexes: IndexesSummaryViewModel {
            active_count: 0,
            total_documents: 0,
            total_documents_formatted: "0".to_string(),
            is_indexing: false,
        },
        activities: vec![],
        system_health: HealthViewModel::new("healthy", 0, 1),
    };

    let vm_json = serde_json::to_string(&dashboard_vm).unwrap();
    let mut context = Context::new();
    context.insert("vm", &dashboard_vm);
    context.insert("vm_json", &vm_json);
    context.insert("page", "dashboard");

    let html = templates
        .render("dashboard.html", &context)
        .expect("Dashboard template should render");

    // Check that nav links are present and correctly formatted
    let expected_nav_links = vec!["/dashboard", "/providers", "/indexes", "/config"];

    for href in expected_nav_links {
        let search = format!("href=\"{}\"", href);
        assert!(
            html.contains(&search),
            "Navigation should contain link to {} (searching for '{}' in HTML of length {})",
            href,
            search,
            html.len()
        );
    }

    // Check logout form action
    assert!(
        html.contains("action=\"/admin/auth/logout\""),
        "Logout form should post to /admin/auth/logout"
    );
}

#[test]
fn test_dashboard_api_endpoints() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let dashboard_vm = DashboardViewModel {
        page: "dashboard",
        metrics: MetricsViewModel::new(0.0, 0.0, 0, 0.0),
        providers: ProvidersViewModel::new(vec![]),
        indexes: IndexesSummaryViewModel {
            active_count: 0,
            total_documents: 0,
            total_documents_formatted: "0".to_string(),
            is_indexing: false,
        },
        activities: vec![],
        system_health: HealthViewModel::new("healthy", 0, 1),
    };

    let vm_json = serde_json::to_string(&dashboard_vm).unwrap();
    let mut context = Context::new();
    context.insert("vm", &dashboard_vm);
    context.insert("vm_json", &vm_json);
    context.insert("page", "dashboard");

    let html = templates
        .render("dashboard.html", &context)
        .expect("Dashboard should render");

    // Check API endpoints used in JavaScript
    assert!(
        html.contains("/admin/dashboard/metrics"),
        "Dashboard should call /admin/dashboard/metrics for refresh"
    );
    assert!(
        html.contains("/admin/activities"),
        "Dashboard should call /admin/activities for activity refresh"
    );
}

#[test]
fn test_diagnostics_api_endpoints() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let mut context = Context::new();
    context.insert("page", "diagnostics");
    context.insert("page_description", "System diagnostics");

    let html = templates
        .render("diagnostics.html", &context)
        .expect("Diagnostics should render");

    // Check that endpoints use /admin/diagnostic (singular)
    assert!(
        html.contains("/admin/diagnostic/health"),
        "Diagnostics should use /admin/diagnostic/health"
    );
    assert!(
        html.contains("/admin/diagnostic/connectivity"),
        "Diagnostics should use /admin/diagnostic/connectivity"
    );
    assert!(
        html.contains("/admin/diagnostic/performance"),
        "Diagnostics should use /admin/diagnostic/performance"
    );

    // Make sure old pluralized endpoints are NOT present
    assert!(
        !html.contains("/admin/diagnostics/"),
        "Diagnostics should NOT use pluralized /admin/diagnostics/"
    );
}

#[test]
fn test_data_management_api_endpoints() {
    let web_interface = WebInterface::new().expect("Failed to create web interface");
    let templates = web_interface.templates();

    let mut context = Context::new();
    context.insert("page", "data");
    context.insert("page_description", "Backup and restore");

    let html = templates
        .render("data_management.html", &context)
        .expect("Data management should render");

    // Check that endpoint uses /admin/backup (singular)
    assert!(
        html.contains("/admin/backup"),
        "Data management should use /admin/backup"
    );

    // Make sure old pluralized endpoint is NOT present
    assert!(
        !html.contains("/admin/backups"),
        "Data management should NOT use pluralized /admin/backups"
    );
}

// ============================================================================
// View Model Tests
// ============================================================================

#[test]
fn test_provider_view_model_status_class() {
    let available = ProviderViewModel::new(
        "1".to_string(),
        "Test".to_string(),
        "embedding".to_string(),
        "available".to_string(),
    );
    assert!(available.is_active);
    assert!(available.status_class.contains("green"));

    let unavailable = ProviderViewModel::new(
        "2".to_string(),
        "Test".to_string(),
        "embedding".to_string(),
        "unavailable".to_string(),
    );
    assert!(!unavailable.is_active);
    assert!(unavailable.status_class.contains("red"));

    let starting = ProviderViewModel::new(
        "3".to_string(),
        "Test".to_string(),
        "embedding".to_string(),
        "starting".to_string(),
    );
    assert!(!starting.is_active);
    assert!(starting.status_class.contains("yellow"));
}

#[test]
fn test_index_view_model_status_class() {
    let active = IndexViewModel::new(
        "1".to_string(),
        "Test".to_string(),
        "active".to_string(),
        1000,
        0,
        0,
    );
    assert!(active.is_active);
    assert!(!active.is_indexing);
    assert!(active.status_class.contains("green"));

    let indexing = IndexViewModel::new(
        "2".to_string(),
        "Test".to_string(),
        "indexing".to_string(),
        500,
        0,
        0,
    );
    assert!(!indexing.is_active);
    assert!(indexing.is_indexing);
    assert!(indexing.status_class.contains("yellow"));

    let error = IndexViewModel::new(
        "3".to_string(),
        "Test".to_string(),
        "error".to_string(),
        0,
        0,
        0,
    );
    assert!(!error.is_active);
    assert!(!error.is_indexing);
    assert!(error.status_class.contains("red"));
}

#[test]
fn test_health_view_model_status_class() {
    let healthy = HealthViewModel::new("healthy", 3600, 1);
    assert!(healthy.status_class.contains("green"));
    assert!(healthy.indicator_class.contains("green"));

    let degraded = HealthViewModel::new("degraded", 3600, 1);
    assert!(degraded.status_class.contains("yellow"));
    assert!(degraded.indicator_class.contains("yellow"));

    let critical = HealthViewModel::new("critical", 3600, 1);
    assert!(critical.status_class.contains("red"));
    assert!(critical.indicator_class.contains("red"));
}

#[test]
fn test_metrics_view_model_formatting() {
    let metrics = MetricsViewModel::new(45.5, 62.3, 1234567, 15.7);
    assert_eq!(metrics.cpu_usage_formatted, "45.5%");
    assert_eq!(metrics.memory_usage_formatted, "62.3%");
    assert_eq!(metrics.total_queries_formatted, "1,234,567");
    assert_eq!(metrics.avg_latency_formatted, "15.7ms");
}

#[test]
fn test_activity_view_model_level_classes() {
    let success = ActivityViewModel::new(
        "1".to_string(),
        "Success".to_string(),
        chrono::Utc::now(),
        "success",
        "test".to_string(),
    );
    assert!(success.level_class.contains("green"));
    assert!(success.indicator_class.contains("green"));

    let warning = ActivityViewModel::new(
        "2".to_string(),
        "Warning".to_string(),
        chrono::Utc::now(),
        "warning",
        "test".to_string(),
    );
    assert!(warning.level_class.contains("yellow"));
    assert!(warning.indicator_class.contains("yellow"));

    let error = ActivityViewModel::new(
        "3".to_string(),
        "Error".to_string(),
        chrono::Utc::now(),
        "error",
        "test".to_string(),
    );
    assert!(error.level_class.contains("red"));
    assert!(error.indicator_class.contains("red"));

    let info = ActivityViewModel::new(
        "4".to_string(),
        "Info".to_string(),
        chrono::Utc::now(),
        "info",
        "test".to_string(),
    );
    assert!(info.level_class.contains("blue"));
    assert!(info.indicator_class.contains("blue"));
}
