//! Security middleware for production hardening
//!
//! Implements comprehensive security headers, request validation,
//! and protection against common web vulnerabilities.

use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Whether security middleware is enabled
    pub enabled: bool,
    /// Content Security Policy
    pub content_security_policy: Option<String>,
    /// Whether to enable HSTS
    pub hsts_enabled: bool,
    /// HSTS max age in seconds
    pub hsts_max_age: u32,
    /// Whether to include subdomains in HSTS
    pub hsts_include_subdomains: bool,
    /// Whether to enable X-Frame-Options
    pub x_frame_options: Option<String>,
    /// Whether to enable X-Content-Type-Options
    pub x_content_type_options: bool,
    /// Referrer policy
    pub referrer_policy: Option<String>,
    /// Permissions policy
    pub permissions_policy: Option<String>,
    /// Cross-Origin-Embedder-Policy
    pub cross_origin_embedder_policy: Option<String>,
    /// Cross-Origin-Opener-Policy
    pub cross_origin_opener_policy: Option<String>,
    /// Cross-Origin-Resource-Policy
    pub cross_origin_resource_policy: Option<String>,
    /// Allowed origins for CORS (empty means allow all in development)
    pub allowed_origins: HashSet<String>,
    /// Maximum request body size in bytes
    pub max_request_size: usize,
    /// Whether to block common attack patterns
    pub block_suspicious_requests: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            // CSP for admin dashboard - requires 'unsafe-inline' for:
            // - Inline <script> blocks in admin templates (dashboard.html, logs.html, etc.)
            // - Tailwind CSS inline configuration
            // - Alpine.js reactive attributes
            // SECURITY NOTE: This CSP is designed for the internal admin interface.
            // API endpoints are protected by authentication and rate limiting.
            // TODO: Consider extracting inline scripts to external files (Phase 3 refactoring)
            content_security_policy: Some(concat!(
                "default-src 'self'; ",
                // Script sources: self + CDNs for Alpine/HTMX/Chart.js + unsafe-inline for inline scripts
                "script-src 'self' https://cdn.tailwindcss.com https://unpkg.com https://cdn.jsdelivr.net 'unsafe-inline'; ",
                // Styles: self + inline for Tailwind utility classes
                "style-src 'self' 'unsafe-inline'; ",
                // Images: self + data URIs for inline icons + HTTPS for external images
                "img-src 'self' data: https:; ",
                // Fonts: self + data URIs for embedded fonts
                "font-src 'self' data:; ",
                // Connections: self only (no external API calls from admin)
                "connect-src 'self'; ",
                // Forms: self only
                "form-action 'self'; ",
                // Base URI: self only (prevent base tag hijacking)
                "base-uri 'self'; ",
                // Object sources: none (no plugins)
                "object-src 'none'; ",
                // Frame ancestors: none (prevent clickjacking via iframes)
                "frame-ancestors 'none'"
            ).to_string()),
            hsts_enabled: true,
            hsts_max_age: 31_536_000, // 1 year (with underscores for readability)
            hsts_include_subdomains: true,
            x_frame_options: Some("DENY".to_string()),
            x_content_type_options: true,
            referrer_policy: Some("strict-origin-when-cross-origin".to_string()),
            permissions_policy: Some("camera=(), microphone=(), geolocation=(), payment=(), usb=(), bluetooth=()".to_string()),
            cross_origin_embedder_policy: Some("require-corp".to_string()),
            cross_origin_opener_policy: Some("same-origin".to_string()),
            cross_origin_resource_policy: Some("same-origin".to_string()),
            allowed_origins: HashSet::new(),
            max_request_size: 10 * 1024 * 1024, // 10MB
            block_suspicious_requests: true,
        }
    }
}

/// Security middleware layer
pub async fn security_middleware(
    State(config): State<SecurityConfig>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if !config.enabled {
        return Ok(next.run(req).await);
    }

    // Validate request size
    if let Some(content_length) = req.headers().get(header::CONTENT_LENGTH) {
        if let Ok(size_str) = content_length.to_str() {
            if let Ok(size) = size_str.parse::<usize>() {
                if size > config.max_request_size {
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
            }
        }
    }

    // Block suspicious requests
    if config.block_suspicious_requests {
        if let Err(response) = validate_request_safety(&req).await {
            return Ok(response);
        }
    }

    // Process the request
    let mut response = next.run(req).await;

    // Add security headers to response
    add_security_headers(&mut response, &config);

    Ok(response)
}

/// Validate request for suspicious patterns
async fn validate_request_safety(req: &Request) -> Result<(), Response> {
    let uri = req.uri();
    let path = uri.path();
    let query = uri.query().unwrap_or("");

    // Block path traversal attempts
    if path.contains("..") || path.contains("\\") {
        return Err(create_security_error("Path traversal attempt detected"));
    }

    // Block suspicious query parameters
    let suspicious_patterns = [
        "<script",
        "</script>",
        "javascript:",
        "data:",
        "vbscript:",
        "onload=",
        "onerror=",
        "onclick=",
        "onmouseover=",
        "../../",
        "..\\",
        "%2e%2e%2f",
        "%2e%2e\\",
        "union select",
        "1=1",
        "or 1=1",
        "drop table",
        "delete from",
    ];

    let full_request = format!("{}?{}", path, query).to_lowercase();
    for pattern in &suspicious_patterns {
        if full_request.contains(&pattern.to_lowercase()) {
            tracing::warn!("Suspicious request pattern detected: {}", pattern);
            return Err(create_security_error("Suspicious request pattern detected"));
        }
    }

    // Block requests with unusual headers
    let suspicious_headers = [
        "x-forwarded-for",
        "x-real-ip",
        "x-client-ip",
        "x-forwarded-host",
        "x-forwarded-proto",
    ];

    for header_name in suspicious_headers {
        if let Some(value) = req.headers().get(header_name) {
            if let Ok(value_str) = value.to_str() {
                if value_str.contains(",") || value_str.contains("\n") || value_str.contains("\r") {
                    tracing::warn!(
                        "Suspicious header detected: {} = {}",
                        header_name,
                        value_str
                    );
                    return Err(create_security_error("Invalid header format"));
                }
            }
        }
    }

    Ok(())
}

/// Create a security error response
fn create_security_error(message: &str) -> Response {
    tracing::warn!("Security violation: {}", message);
    (
        StatusCode::BAD_REQUEST,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::json!({
            "error": "Security violation",
            "message": message,
            "code": "SECURITY_VIOLATION"
        })
        .to_string(),
    )
        .into_response()
}

/// Add security headers to response
fn add_security_headers(response: &mut Response, config: &SecurityConfig) {
    let headers = response.headers_mut();

    // Content Security Policy
    if let Some(csp) = &config.content_security_policy {
        if let Ok(value) = HeaderValue::from_str(csp) {
            headers.insert("Content-Security-Policy", value);
        }
    }

    // HTTP Strict Transport Security
    if config.hsts_enabled {
        let mut hsts_value = format!("max-age={}", config.hsts_max_age);
        if config.hsts_include_subdomains {
            hsts_value.push_str("; includeSubDomains");
        }
        if let Ok(value) = HeaderValue::from_str(&hsts_value) {
            headers.insert("Strict-Transport-Security", value);
        }
    }

    // X-Frame-Options
    if let Some(xfo) = &config.x_frame_options {
        if let Ok(value) = HeaderValue::from_str(xfo) {
            headers.insert("X-Frame-Options", value);
        }
    }

    // X-Content-Type-Options
    if config.x_content_type_options {
        headers.insert(
            "X-Content-Type-Options",
            HeaderValue::from_static("nosniff"),
        );
    }

    // Referrer-Policy
    if let Some(rp) = &config.referrer_policy {
        if let Ok(value) = HeaderValue::from_str(rp) {
            headers.insert("Referrer-Policy", value);
        }
    }

    // Permissions-Policy
    if let Some(pp) = &config.permissions_policy {
        if let Ok(value) = HeaderValue::from_str(pp) {
            headers.insert("Permissions-Policy", value);
        }
    }

    // Cross-Origin-Embedder-Policy
    if let Some(coep) = &config.cross_origin_embedder_policy {
        if let Ok(value) = HeaderValue::from_str(coep) {
            headers.insert("Cross-Origin-Embedder-Policy", value);
        }
    }

    // Cross-Origin-Opener-Policy
    if let Some(coop) = &config.cross_origin_opener_policy {
        if let Ok(value) = HeaderValue::from_str(coop) {
            headers.insert("Cross-Origin-Opener-Policy", value);
        }
    }

    // Cross-Origin-Resource-Policy
    if let Some(corp) = &config.cross_origin_resource_policy {
        if let Ok(value) = HeaderValue::from_str(corp) {
            headers.insert("Cross-Origin-Resource-Policy", value);
        }
    }

    // X-Request-ID for tracing
    if let Ok(value) = HeaderValue::from_str(&format!("req-{}", uuid::Uuid::new_v4())) {
        headers.insert("X-Request-ID", value);
    }

    // Server header
    headers.insert(
        header::SERVER,
        HeaderValue::from_static("mcp-context-browser"),
    );
}

/// Advanced request validation middleware
pub async fn request_validation_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Validate HTTP method
    let method = req.method();
    if !is_allowed_method(method) {
        return Err(StatusCode::METHOD_NOT_ALLOWED);
    }

    // Validate URI
    let uri = req.uri();
    if validate_uri(uri).is_err() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate headers
    if validate_headers(req.headers()).is_err() {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(next.run(req).await)
}

/// Check if HTTP method is allowed
fn is_allowed_method(method: &Method) -> bool {
    matches!(
        method,
        &Method::GET
            | &Method::POST
            | &Method::PUT
            | &Method::DELETE
            | &Method::HEAD
            | &Method::OPTIONS
    )
}

/// Validate URI for security
fn validate_uri(uri: &Uri) -> Result<(), &'static str> {
    let path = uri.path();
    let query = uri.query().unwrap_or("");

    // Check path length
    if path.len() > 2048 {
        return Err("URI path too long");
    }

    // Check query length
    if query.len() > 2048 {
        return Err("URI query too long");
    }

    // Check for null bytes
    if path.contains('\0') {
        return Err("Null byte in URI");
    }

    // Check for encoded null bytes
    if path.contains("%00") {
        return Err("Encoded null byte in URI");
    }

    // Check for suspicious patterns
    let suspicious = ["\r", "\n", "\t", "<", ">"];
    for pattern in suspicious {
        if path.contains(pattern) {
            return Err("Suspicious character in URI");
        }
    }

    // Check for percent-encoded suspicious characters
    let suspicious_encoded = ["%3C", "%3E", "%22", "%27"]; // < > " '
    for pattern in suspicious_encoded {
        if path.contains(pattern) {
            return Err("Suspicious encoded character in URI");
        }
    }

    Ok(())
}

/// Validate request headers
fn validate_headers(headers: &HeaderMap) -> Result<(), &'static str> {
    for (name, value) in headers {
        // Check header name length
        if name.as_str().len() > 256 {
            return Err("Header name too long");
        }

        // Check header value
        if let Ok(value_str) = value.to_str() {
            // Check value length
            if value_str.len() > 8192 {
                return Err("Header value too long");
            }

            // Check for control characters
            if value_str
                .chars()
                .any(|c| c.is_control() && !c.is_whitespace())
            {
                return Err("Control character in header");
            }
        } else {
            return Err("Invalid header encoding");
        }
    }

    Ok(())
}
