//! Error Extension Tests

use mcb_domain::error::{Error, Result};
use mcb_infrastructure::error_ext::{ErrorContext, infra};
use std::io;

#[test]
fn test_error_context_extension() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");

    let result: Result<()> = Err(io_error).io_context("failed to read file");
    assert!(result.is_err());

    if let Err(Error::Io { source, message }) = result {
        assert!(message.contains("failed to read file"));
        assert!(source.is_some());
    } else {
        panic!("Expected Io error");
    }
}

#[test]
fn test_infra_error_creation() {
    let error = infra::infrastructure_error_msg("test error message");

    match error {
        Error::Infrastructure { message, source } => {
            assert_eq!(message, "test error message");
            assert!(source.is_none());
        }
        _ => panic!("Expected Infrastructure error"),
    }
}
