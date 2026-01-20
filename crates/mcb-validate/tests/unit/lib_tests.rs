//! Unit tests for `mcb_validate::lib` module

use mcb_validate::{ArchitectureValidator, Severity, ValidationConfig};
use std::path::PathBuf;

#[test]
fn test_severity_serialization() {
    let severity = Severity::Error;
    let json = serde_json::to_string(&severity).unwrap();
    assert_eq!(json, "\"Error\"");
}

#[test]
fn test_validation_config_creation() {
    let config = ValidationConfig::new("/workspace");
    assert_eq!(config.workspace_root.to_str().unwrap(), "/workspace");
    assert!(config.additional_src_paths.is_empty());
    assert!(config.exclude_patterns.is_empty());
}

#[test]
fn test_validation_config_builder() {
    let config = ValidationConfig::new("/workspace")
        .with_additional_path("../src")
        .with_additional_path("../legacy")
        .with_exclude_pattern("target/")
        .with_exclude_pattern("tests/fixtures/");

    assert_eq!(config.additional_src_paths.len(), 2);
    assert_eq!(config.exclude_patterns.len(), 2);
}

#[test]
fn test_validation_config_should_exclude() {
    let config = ValidationConfig::new("/workspace")
        .with_exclude_pattern("target/")
        .with_exclude_pattern("fixtures/");

    assert!(config.should_exclude(&PathBuf::from("/workspace/target/debug")));
    assert!(config.should_exclude(&PathBuf::from("/workspace/tests/fixtures/data.json")));
    assert!(!config.should_exclude(&PathBuf::from("/workspace/src/lib.rs")));
}

#[test]
fn test_architecture_validator_with_config() {
    let config = ValidationConfig::new("/tmp/test-workspace")
        .with_additional_path("../legacy-src")
        .with_exclude_pattern("target/");

    let validator = ArchitectureValidator::with_config(config);
    let config_ref = validator.config();

    assert_eq!(config_ref.additional_src_paths.len(), 1);
    assert_eq!(config_ref.exclude_patterns.len(), 1);
}
