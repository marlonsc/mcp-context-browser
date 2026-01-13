//! Tests for Server Version Compatibility
//!
//! Tests migrated from src/server/transport/versioning.rs

use mcp_context_browser::server::transport::config::VersionConfig;
use mcp_context_browser::server::transport::{CompatibilityResult, SemVer, VersionChecker};

#[test]
fn test_semver_parse() {
    let v = SemVer::parse("0.1.0").expect("Failed to parse version 0.1.0");
    assert_eq!(v.major, 0);
    assert_eq!(v.minor, 1);
    assert_eq!(v.patch, 0);

    let v = SemVer::parse("v1.2.3").expect("Failed to parse version v1.2.3");
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 2);
    assert_eq!(v.patch, 3);

    let v = SemVer::parse("2.5").expect("Failed to parse version 2.5");
    assert_eq!(v.major, 2);
    assert_eq!(v.minor, 5);
    assert_eq!(v.patch, 0);
}

#[test]
fn test_semver_parse_invalid() {
    // Invalid version strings
    assert!(SemVer::parse("").is_none());
    assert!(SemVer::parse("1").is_none());
    assert!(SemVer::parse("abc").is_none());
}

#[test]
fn test_minor_distance() {
    let v1 = SemVer::parse("1.2.0").expect("Failed to parse version 1.2.0");
    let v2 = SemVer::parse("1.3.0").expect("Failed to parse version 1.3.0");
    let v3 = SemVer::parse("2.2.0").expect("Failed to parse version 2.2.0");

    assert_eq!(v1.minor_distance(&v2), Some(1));
    assert_eq!(v2.minor_distance(&v1), Some(1));
    assert_eq!(v1.minor_distance(&v3), None); // Different major
}

#[test]
fn test_minor_distance_same_version() {
    let v1 = SemVer::parse("1.2.3").expect("Failed to parse version");
    let v2 = SemVer::parse("1.2.5").expect("Failed to parse version");

    // Same minor version, different patch
    assert_eq!(v1.minor_distance(&v2), Some(0));
}

#[test]
fn test_semver_display() {
    let v = SemVer::parse("1.2.3").expect("Failed to parse version");
    assert_eq!(v.to_string(), "1.2.3");
}

#[test]
fn test_version_checker_compatible() {
    let config = VersionConfig {
        version_tolerance: 1,
        warn_only: true,
    };
    let checker = VersionChecker::new(config);
    let current = checker.version_string();

    let result = checker.check_compatibility(&current);
    assert_eq!(result, CompatibilityResult::Compatible);
}

#[test]
fn test_version_checker_warning() {
    let config = VersionConfig {
        version_tolerance: 1,
        warn_only: true,
    };
    let checker = VersionChecker::new(config);

    // Create a version 1 minor apart
    let mut expected = checker.current_version().clone();
    expected.minor = expected.minor.saturating_add(1);

    let result = checker.check_compatibility(&expected.to_string());
    assert!(matches!(result, CompatibilityResult::Warning { .. }));
    assert!(result.allows_proceed());
}

#[test]
fn test_version_checker_incompatible() {
    let config = VersionConfig {
        version_tolerance: 1,
        warn_only: false,
    };
    let checker = VersionChecker::new(config);

    // Create a version 2+ minor versions apart
    let mut expected = checker.current_version().clone();
    expected.minor = expected.minor.saturating_add(5);

    let result = checker.check_compatibility(&expected.to_string());
    assert!(matches!(result, CompatibilityResult::Incompatible { .. }));
    assert!(!result.allows_proceed());
}

#[test]
fn test_version_checker_major_mismatch() {
    let config = VersionConfig {
        version_tolerance: 1,
        warn_only: false,
    };
    let checker = VersionChecker::new(config);

    // Create a version with different major
    let mut expected = checker.current_version().clone();
    expected.major = expected.major.saturating_add(1);

    let result = checker.check_compatibility(&expected.to_string());
    assert!(matches!(result, CompatibilityResult::Incompatible { .. }));
    assert!(!result.allows_proceed());
}

#[test]
fn test_compatibility_result_header() {
    assert_eq!(
        CompatibilityResult::Compatible.as_header_value(),
        "compatible"
    );
    assert_eq!(
        CompatibilityResult::Warning {
            message: "test".to_string()
        }
        .as_header_value(),
        "warning"
    );
    assert_eq!(
        CompatibilityResult::Incompatible {
            message: "test".to_string()
        }
        .as_header_value(),
        "incompatible"
    );
}

#[test]
fn test_compatibility_result_allows_proceed() {
    assert!(CompatibilityResult::Compatible.allows_proceed());
    assert!(CompatibilityResult::Warning {
        message: "test".to_string()
    }
    .allows_proceed());
    assert!(!CompatibilityResult::Incompatible {
        message: "test".to_string()
    }
    .allows_proceed());
}

#[test]
fn test_version_checker_with_defaults() {
    let checker = VersionChecker::with_defaults();

    // Should have a valid version string
    let version = checker.version_string();
    assert!(!version.is_empty());
    assert!(SemVer::parse(&version).is_some());
}

#[test]
fn test_version_checker_invalid_expected() {
    let checker = VersionChecker::with_defaults();

    // Invalid version should return a warning, not an error
    let result = checker.check_compatibility("invalid");
    assert!(matches!(result, CompatibilityResult::Warning { .. }));
    assert!(result.allows_proceed());
}

#[test]
fn test_version_info() {
    let config = VersionConfig {
        version_tolerance: 2,
        warn_only: false,
    };
    let checker = VersionChecker::new(config);

    let info = checker.get_version_info();
    assert!(!info.server_version.is_empty());
    assert_eq!(info.version_tolerance, 2);
    assert!(!info.warn_only);
}
