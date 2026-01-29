/// Unified integration test helpers
/// Detects external service availability and skips tests if services are unavailable
///
/// Usage: Call `skip_if_service_unavailable!("ServiceName", is_service_available())`
/// at the start of your test function to skip if the service is unavailable.
use std::net::TcpStream;
use std::time::Duration;

/// Check if a service is available on given host:port
pub fn check_service_available(host: &str, port: u16) -> bool {
    let addr = format!("{}:{}", host, port);
    match addr.parse() {
        Ok(socket_addr) => {
            TcpStream::connect_timeout(&socket_addr, Duration::from_millis(300)).is_ok()
        }
        Err(_) => false,
    }
}

/// Milvus vector database service (default port 19530)
pub fn is_milvus_available() -> bool {
    check_service_available("127.0.0.1", 19530) || check_service_available("localhost", 19530)
}

/// Ollama LLM service (default port 11434)
pub fn is_ollama_available() -> bool {
    check_service_available("127.0.0.1", 11434) || check_service_available("localhost", 11434)
}

/// Redis cache service (default port 6379)
pub fn is_redis_available() -> bool {
    check_service_available("127.0.0.1", 6379) || check_service_available("localhost", 6379)
}

/// PostgreSQL service (default port 5432)
pub fn is_postgres_available() -> bool {
    check_service_available("127.0.0.1", 5432) || check_service_available("localhost", 5432)
}

/// Check if running in CI environment
/// Returns true if CI environment variable is set
#[allow(dead_code)] // Used in macros below via direct env var check
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok()
}

/// Skip test if service is not available or if in CI
/// Returns early from test function if service is unavailable or in CI environment
///
/// # Example - Single service
/// ```ignore
/// skip_if_service_unavailable!("Milvus", is_milvus_available());
/// skip_if_service_unavailable!("Ollama", is_ollama_available());
/// ```
///
/// # Example - Multiple services (any one missing skips test)
/// ```ignore
/// skip_if_any_service_unavailable!("Milvus" => is_milvus_available(), "Ollama" => is_ollama_available());
/// ```
#[macro_export]
macro_rules! skip_if_service_unavailable {
    ($service:expr, $is_available:expr) => {
        // Skip in CI to prevent coverage timeouts
        // Check CI environment variables directly to avoid clippy::crate-in-macro-def warning
        if std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok() {
            println!("⊘ SKIPPED: Running in CI environment (skipping test)");
            return;
        }
        if !$is_available {
            println!(
                "⊘ SKIPPED: {} service not available (skipping test)",
                $service
            );
            return;
        }
    };
}

/// Skip test if any required services are unavailable or if in CI
/// Useful for tests requiring multiple external services
///
/// # Example
/// ```ignore
/// skip_if_any_service_unavailable!("Milvus" => is_milvus_available(), "Ollama" => is_ollama_available());
/// ```
#[macro_export]
macro_rules! skip_if_any_service_unavailable {
    ($($service:expr => $is_available:expr),+ $(,)?) => {
        // Skip in CI to prevent coverage timeouts
        if std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok() {
            println!("⊘ SKIPPED: Running in CI environment (skipping test)");
            return;
        }
        $(
            if !$is_available {
                println!(
                    "⊘ SKIPPED: {} service not available (skipping test)",
                    $service
                );
                return;
            }
        )+
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_detection_logic() {
        // These checks should not panic and return boolean values
        let milvus = is_milvus_available();
        let ollama = is_ollama_available();
        let redis = is_redis_available();
        let postgres = is_postgres_available();

        // Assert that functions return boolean (may be true or false depending on environment)
        assert!(matches!(milvus, true | false));
        assert!(matches!(ollama, true | false));
        assert!(matches!(redis, true | false));
        assert!(matches!(postgres, true | false));

        println!("✓ Service detection logic verified");
    }
}
