//! Tests for infrastructure constants
//!
//! Validates that infrastructure constants have reasonable values
//! and maintain expected relationships.
#![allow(clippy::assertions_on_constants)]

use mcb_infrastructure::constants::*;

// ============================================================================
// HTTP Pool Constants Tests
// ============================================================================

#[test]
fn test_http_pool_constants_reasonable() {
    // Timeouts should be reasonable (not too short, not too long)
    assert!(HTTP_REQUEST_TIMEOUT_SECS >= 5, "Request timeout too short");
    assert!(HTTP_REQUEST_TIMEOUT_SECS <= 120, "Request timeout too long");

    assert!(
        HTTP_CLIENT_IDLE_TIMEOUT_SECS >= 30,
        "Idle timeout too short"
    );
    assert!(
        HTTP_CLIENT_IDLE_TIMEOUT_SECS <= 300,
        "Idle timeout too long"
    );

    assert!(HTTP_KEEPALIVE_SECS >= 30, "Keepalive too short");
    assert!(HTTP_KEEPALIVE_SECS <= 120, "Keepalive too long");

    assert!(HTTP_MAX_IDLE_PER_HOST >= 5, "Max idle per host too low");
    assert!(HTTP_MAX_IDLE_PER_HOST <= 50, "Max idle per host too high");
}

#[test]
fn test_http_timeout_relationships() {
    // Idle timeout should be greater than request timeout
    assert!(
        HTTP_CLIENT_IDLE_TIMEOUT_SECS >= HTTP_REQUEST_TIMEOUT_SECS,
        "Idle timeout should be >= request timeout"
    );
}

// ============================================================================
// Embedding Dimension Constants Tests
// ============================================================================

#[test]
fn test_embedding_dimension_constants() {
    // All embedding dimensions should be positive
    assert!(EMBEDDING_DIMENSION_NULL > 0);
    assert!(EMBEDDING_DIMENSION_FASTEMBED_DEFAULT > 0);
    assert!(EMBEDDING_DIMENSION_OPENAI_SMALL > 0);
    assert!(EMBEDDING_DIMENSION_OPENAI_LARGE > 0);
    assert!(EMBEDDING_DIMENSION_OPENAI_ADA > 0);
    assert!(EMBEDDING_DIMENSION_VOYAGEAI_DEFAULT > 0);
    assert!(EMBEDDING_DIMENSION_VOYAGEAI_CODE > 0);
    assert!(EMBEDDING_DIMENSION_OLLAMA_NOMIC > 0);
    assert!(EMBEDDING_DIMENSION_OLLAMA_MINILM > 0);
    assert!(EMBEDDING_DIMENSION_OLLAMA_MXBAI > 0);
    assert!(EMBEDDING_DIMENSION_OLLAMA_ARCTIC > 0);
    assert!(EMBEDDING_DIMENSION_OLLAMA_DEFAULT > 0);
    assert!(EMBEDDING_DIMENSION_GEMINI > 0);
}

#[test]
fn test_embedding_dimension_common_values() {
    // Common embedding dimensions are powers of 2 or multiples of 128/256
    let common_dims = [256, 384, 512, 768, 1024, 1536, 2048, 3072];

    // Null provider uses common dimension
    assert!(
        common_dims.contains(&EMBEDDING_DIMENSION_NULL),
        "Null dimension should be a common value"
    );

    // OpenAI dimensions should be in known range
    assert!(
        EMBEDDING_DIMENSION_OPENAI_SMALL >= 1024,
        "OpenAI small should be >= 1024"
    );
    assert!(
        EMBEDDING_DIMENSION_OPENAI_LARGE > EMBEDDING_DIMENSION_OPENAI_SMALL,
        "OpenAI large should be > small"
    );
}

#[test]
fn test_embedding_dimension_openai_consistency() {
    // OpenAI ADA and Small should have same dimension
    assert_eq!(
        EMBEDDING_DIMENSION_OPENAI_ADA, EMBEDDING_DIMENSION_OPENAI_SMALL,
        "OpenAI ADA and small should have same dimensions"
    );
}

// ============================================================================
// Cache Constants Tests
// ============================================================================

#[test]
fn test_cache_ttl_constants() {
    // Cache TTL should be reasonable
    assert!(CACHE_DEFAULT_TTL_SECS >= 60, "Cache TTL too short");
    assert!(CACHE_DEFAULT_TTL_SECS <= 86400, "Cache TTL too long");
}

#[test]
fn test_cache_size_constants() {
    // Cache size should be reasonable
    assert!(
        CACHE_DEFAULT_SIZE_LIMIT >= 1024 * 1024,
        "Cache size too small"
    ); // At least 1MB
    assert!(
        CACHE_DEFAULT_SIZE_LIMIT <= 1024 * 1024 * 1024,
        "Cache size too large"
    ); // At most 1GB
}

#[test]
fn test_cache_namespace_separator() {
    // Namespace separator should be a single character
    assert_eq!(
        CACHE_NAMESPACE_SEPARATOR.len(),
        1,
        "Namespace separator should be single char"
    );
}

// ============================================================================
// Authentication Constants Tests
// ============================================================================

#[test]
fn test_jwt_expiration_constants() {
    // JWT expiration should be reasonable
    assert!(
        JWT_DEFAULT_EXPIRATION_SECS >= 3600,
        "JWT expiration too short"
    ); // At least 1 hour
    assert!(
        JWT_DEFAULT_EXPIRATION_SECS <= 604800,
        "JWT expiration too long"
    ); // At most 1 week

    // Refresh token should be longer than access token
    assert!(
        JWT_REFRESH_EXPIRATION_SECS > JWT_DEFAULT_EXPIRATION_SECS,
        "Refresh token should be longer than access token"
    );
}

#[test]
fn test_bcrypt_cost() {
    // Bcrypt cost should be in reasonable range
    assert!(BCRYPT_DEFAULT_COST >= 10, "Bcrypt cost too low (insecure)");
    assert!(BCRYPT_DEFAULT_COST <= 15, "Bcrypt cost too high (slow)");
}

#[test]
fn test_auth_headers() {
    // Headers should be lowercase (HTTP/2 requirement)
    assert_eq!(
        API_KEY_HEADER.to_lowercase(),
        API_KEY_HEADER,
        "API key header should be lowercase"
    );
    assert_eq!(
        AUTHORIZATION_HEADER.to_lowercase(),
        AUTHORIZATION_HEADER,
        "Authorization header should be lowercase"
    );

    // Bearer prefix should end with space
    assert!(
        BEARER_PREFIX.ends_with(' '),
        "Bearer prefix should end with space"
    );
}

// ============================================================================
// Crypto Constants Tests
// ============================================================================

#[test]
fn test_aes_gcm_constants() {
    // AES-256-GCM standard sizes
    assert_eq!(AES_GCM_KEY_SIZE, 32, "AES-256 key size should be 32 bytes");
    assert_eq!(
        AES_GCM_NONCE_SIZE, 12,
        "AES-GCM nonce size should be 12 bytes"
    );
}

#[test]
fn test_pbkdf2_iterations() {
    // PBKDF2 iterations should be high enough for security
    assert!(
        PBKDF2_ITERATIONS >= 10_000,
        "PBKDF2 iterations too low (insecure)"
    );
}

// ============================================================================
// Server Constants Tests
// ============================================================================

#[test]
fn test_server_port_constants() {
    // Ports should be in valid range
    assert!(DEFAULT_HTTP_PORT > 1024, "HTTP port should be > 1024");
    assert!(DEFAULT_HTTPS_PORT > 1024, "HTTPS port should be > 1024");
    assert!(DEFAULT_HTTP_PORT < 65535, "HTTP port should be < 65535");
    assert!(DEFAULT_HTTPS_PORT < 65535, "HTTPS port should be < 65535");

    // HTTPS should be different from HTTP
    assert_ne!(
        DEFAULT_HTTP_PORT, DEFAULT_HTTPS_PORT,
        "HTTP and HTTPS ports should differ"
    );
}

#[test]
fn test_server_timeout_constants() {
    assert!(REQUEST_TIMEOUT_SECS >= 10, "Request timeout too short");
    assert!(CONNECTION_TIMEOUT_SECS >= 5, "Connection timeout too short");
    assert!(
        REQUEST_TIMEOUT_SECS >= CONNECTION_TIMEOUT_SECS,
        "Request timeout should be >= connection timeout"
    );
}

// ============================================================================
// Health Check Constants Tests
// ============================================================================

#[test]
fn test_health_check_constants() {
    // Health check should be quick
    assert!(
        HEALTH_CHECK_TIMEOUT_SECS <= 30,
        "Health check timeout too long"
    );

    // Interval should be reasonable
    assert!(
        HEALTH_CHECK_INTERVAL_SECS >= 10,
        "Health check interval too short"
    );
    assert!(
        HEALTH_CHECK_INTERVAL_SECS <= 300,
        "Health check interval too long"
    );

    // Failure threshold should be reasonable
    assert!(
        HEALTH_CHECK_FAILURE_THRESHOLD >= 2,
        "Failure threshold too low"
    );
    assert!(
        HEALTH_CHECK_FAILURE_THRESHOLD <= 10,
        "Failure threshold too high"
    );
}

// ============================================================================
// Circuit Breaker Constants Tests
// ============================================================================

#[test]
fn test_circuit_breaker_constants() {
    // Failure threshold should be reasonable
    assert!(
        CIRCUIT_BREAKER_FAILURE_THRESHOLD >= 3,
        "Circuit breaker failure threshold too low"
    );

    // Success threshold should be less than failure threshold
    assert!(
        CIRCUIT_BREAKER_SUCCESS_THRESHOLD <= CIRCUIT_BREAKER_FAILURE_THRESHOLD,
        "Success threshold should be <= failure threshold"
    );

    // Timeout should be reasonable
    assert!(
        CIRCUIT_BREAKER_TIMEOUT_SECS >= 30,
        "Circuit breaker timeout too short"
    );
}

// ============================================================================
// Rate Limiter Constants Tests
// ============================================================================

#[test]
fn test_rate_limiter_constants() {
    // Default RPS should be reasonable
    assert!(RATE_LIMITER_DEFAULT_RPS >= 10, "Default RPS too low");

    // Burst should be greater than or equal to RPS
    assert!(
        RATE_LIMITER_DEFAULT_BURST >= RATE_LIMITER_DEFAULT_RPS,
        "Burst should be >= RPS"
    );
}

// ============================================================================
// File System Constants Tests
// ============================================================================

#[test]
fn test_file_permissions() {
    // Standard Unix permissions
    assert_eq!(
        DEFAULT_FILE_PERMISSIONS, 0o644,
        "File permissions should be 0o644"
    );
    assert_eq!(
        DEFAULT_DIR_PERMISSIONS, 0o755,
        "Dir permissions should be 0o755"
    );
}

// ============================================================================
// Shutdown Constants Tests
// ============================================================================

#[test]
fn test_shutdown_constants() {
    // Graceful should be longer than force
    assert!(
        GRACEFUL_SHUTDOWN_TIMEOUT_SECS > FORCE_SHUTDOWN_TIMEOUT_SECS,
        "Graceful timeout should be > force timeout"
    );

    // Both should be reasonable
    assert!(
        GRACEFUL_SHUTDOWN_TIMEOUT_SECS <= 120,
        "Graceful timeout too long"
    );
    assert!(FORCE_SHUTDOWN_TIMEOUT_SECS >= 5, "Force timeout too short");
}
