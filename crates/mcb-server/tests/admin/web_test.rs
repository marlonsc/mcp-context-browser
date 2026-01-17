//! Tests for Admin Web UI
//!
//! Tests the web dashboard pages and routes.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use mcb_server::admin::web::web_router;
use tower::ServiceExt;

#[tokio::test]
async fn test_dashboard_returns_html() {
    let router = web_router();

    let response = router
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Dashboard"));
}

#[tokio::test]
async fn test_config_page_returns_html() {
    let router = web_router();

    let response = router
        .oneshot(
            Request::builder()
                .uri("/ui/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("Configuration"));
}

#[tokio::test]
async fn test_health_page_returns_html() {
    let router = web_router();

    let response = router
        .oneshot(
            Request::builder()
                .uri("/ui/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("Health Status"));
}

#[tokio::test]
async fn test_indexing_page_returns_html() {
    let router = web_router();

    let response = router
        .oneshot(
            Request::builder()
                .uri("/ui/indexing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("Indexing Status"));
}

#[tokio::test]
async fn test_favicon_returns_svg() {
    let router = web_router();

    let response = router
        .oneshot(
            Request::builder()
                .uri("/favicon.ico")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "image/svg+xml"
    );
}
