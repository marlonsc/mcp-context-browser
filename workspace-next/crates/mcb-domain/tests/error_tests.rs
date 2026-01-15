//! Unit tests for domain error types

#[cfg(test)]
mod tests {
    use mcb_domain::Error;

    #[test]
    fn test_error_creation() {
        let error = Error::generic("Something went wrong");
        match error {
            Error::Generic(msg) => assert_eq!(msg, "Something went wrong".to_string()),
            _ => panic!("Expected Generic error"),
        }
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
            Error::Io { source: _ } => {
                // We can't easily test the exact source, but we can verify it's an Io error
                assert!(true);
            },
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
        let error = Error::generic("Test error");
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Generic"));
        assert!(debug_str.contains("Test error"));
    }

    #[test]
    fn test_error_equality() {
        let error1 = Error::generic("Same message");
        let error2 = Error::generic("Same message");
        let error3 = Error::generic("Different message");

        // Note: Error doesn't implement PartialEq in this case, so we can't test equality directly
        // But we can test that different error types are different
        match (error1, error2, error3) {
            (Error::Generic(msg1), Error::Generic(msg2), Error::Generic(msg3)) => {
                assert_eq!(msg1, msg2);
                assert_ne!(msg1, msg3);
            },
            _ => panic!("Unexpected error types"),
        }
    }
}