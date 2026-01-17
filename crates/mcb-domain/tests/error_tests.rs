//! Unit tests for domain error types

use mcb_domain::Error;

#[test]
fn test_error_creation() {
    let error = Error::generic("Something went wrong");
    // Generic wraps a boxed error, test via display
    let display_str = format!("{}", error);
    assert!(display_str.contains("Something went wrong"));
}

#[test]
fn test_not_found_error() {
    let error = Error::not_found("user");
    match error {
        Error::NotFound { resource } => assert_eq!(resource, "user"),
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_invalid_argument_error() {
    let error = Error::invalid_argument("Invalid input provided");
    match error {
        Error::InvalidArgument { message } => assert_eq!(message, "Invalid input provided"),
        _ => panic!("Expected InvalidArgument error"),
    }
}

#[test]
fn test_embedding_error() {
    let error = Error::embedding("Model not available");
    match error {
        Error::Embedding { message } => assert_eq!(message, "Model not available"),
        _ => panic!("Expected Embedding error"),
    }
}

#[test]
fn test_vector_db_error() {
    let error = Error::vector_db("Connection failed");
    match error {
        Error::VectorDb { message } => assert_eq!(message, "Connection failed"),
        _ => panic!("Expected VectorDb error"),
    }
}

#[test]
fn test_io_error() {
    let error = Error::io("File not found");
    match error {
        Error::Io { message, source: _ } => {
            assert_eq!(message, "File not found");
        }
        _ => panic!("Expected Io error"),
    }
}

#[test]
fn test_config_error() {
    let error = Error::config("Missing required config");
    match error {
        Error::Config { message } => assert_eq!(message, "Missing required config"),
        _ => panic!("Expected Config error"),
    }
}

#[test]
fn test_internal_error() {
    let error = Error::internal("Unexpected internal error");
    match error {
        Error::Internal { message } => assert_eq!(message, "Unexpected internal error"),
        _ => panic!("Expected Internal error"),
    }
}

#[test]
fn test_cache_error() {
    let error = Error::cache("Cache operation failed");
    match error {
        Error::Cache { message } => assert_eq!(message, "Cache operation failed"),
        _ => panic!("Expected Cache error"),
    }
}

#[test]
fn test_string_error() {
    let error: Error = "Simple string error".into();
    match error {
        Error::String(msg) => assert_eq!(msg, "Simple string error"),
        _ => panic!("Expected String error"),
    }
}

#[test]
fn test_error_from_string() {
    let error: Error = String::from("String error").into();
    match error {
        Error::String(msg) => assert_eq!(msg, "String error"),
        _ => panic!("Expected String error"),
    }
}

#[test]
fn test_error_display() {
    // Test that errors can be displayed (implement Debug and Display)
    let error = Error::not_found("test-resource");
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("NotFound"));
    assert!(debug_str.contains("test-resource"));
}

#[test]
fn test_error_equality_via_string() {
    // Test that different error types can be distinguished
    let not_found = Error::not_found("resource");
    let invalid_arg = Error::invalid_argument("bad argument");

    // Verify they are different variants via pattern matching
    assert!(matches!(not_found, Error::NotFound { .. }));
    assert!(matches!(invalid_arg, Error::InvalidArgument { .. }));
    assert!(!matches!(not_found, Error::InvalidArgument { .. }));
}

#[test]
fn test_network_error() {
    let error = Error::network("Connection refused");
    match error {
        Error::Network { message, source: _ } => {
            assert_eq!(message, "Connection refused");
        }
        _ => panic!("Expected Network error"),
    }
}

#[test]
fn test_database_error() {
    let error = Error::database("Query failed");
    match error {
        Error::Database { message, source: _ } => {
            assert_eq!(message, "Query failed");
        }
        _ => panic!("Expected Database error"),
    }
}

#[test]
fn test_authentication_error() {
    let error = Error::authentication("Invalid token");
    match error {
        Error::Authentication { message, source: _ } => {
            assert_eq!(message, "Invalid token");
        }
        _ => panic!("Expected Authentication error"),
    }
}

#[test]
fn test_infrastructure_error() {
    let error = Error::infrastructure("Service unavailable");
    match error {
        Error::Infrastructure { message, source: _ } => {
            assert_eq!(message, "Service unavailable");
        }
        _ => panic!("Expected Infrastructure error"),
    }
}

#[test]
fn test_configuration_error() {
    let error = Error::configuration("Missing setting");
    match error {
        Error::Configuration { message, source: _ } => {
            assert_eq!(message, "Missing setting");
        }
        _ => panic!("Expected Configuration error"),
    }
}
