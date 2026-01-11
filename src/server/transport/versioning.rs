//! Server Version Compatibility
//!
//! Handles ±1 minor version compatibility for server updates.

use super::config::VersionConfig;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Result of version compatibility check
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CompatibilityResult {
    /// Versions are fully compatible
    Compatible,
    /// Versions differ but within tolerance (warning)
    Warning { message: String },
    /// Versions are incompatible
    Incompatible { message: String },
}

impl CompatibilityResult {
    /// Check if the result allows proceeding
    pub fn allows_proceed(&self) -> bool {
        matches!(self, Self::Compatible | Self::Warning { .. })
    }

    /// Get the result as a string for headers
    pub fn as_header_value(&self) -> &'static str {
        match self {
            Self::Compatible => "compatible",
            Self::Warning { .. } => "warning",
            Self::Incompatible { .. } => "incompatible",
        }
    }
}

/// Parsed semantic version
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    /// Parse a version string (e.g., "0.1.0")
    pub fn parse(version: &str) -> Option<Self> {
        let parts: Vec<&str> = version.trim_start_matches('v').split('.').collect();
        if parts.len() < 2 {
            return None;
        }

        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);

        Some(Self {
            major,
            minor,
            patch,
        })
    }

    /// Calculate the "distance" between two versions in minor versions
    pub fn minor_distance(&self, other: &SemVer) -> Option<u32> {
        if self.major != other.major {
            return None; // Different major versions are incompatible
        }

        let diff = (self.minor as i64 - other.minor as i64).unsigned_abs() as u32;
        Some(diff)
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Server version compatibility checker
pub struct VersionChecker {
    current_version: SemVer,
    config: VersionConfig,
}

impl VersionChecker {
    /// Create a new version checker
    ///
    /// Falls back to version 0.0.0 if CARGO_PKG_VERSION cannot be parsed
    /// (should never happen since Cargo validates version format)
    pub fn new(config: VersionConfig) -> Self {
        let current_version = SemVer::parse(env!("CARGO_PKG_VERSION")).unwrap_or(SemVer {
            major: 0,
            minor: 0,
            patch: 0,
        });

        Self {
            current_version,
            config,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(VersionConfig::default())
    }

    /// Get the current server version
    pub fn current_version(&self) -> &SemVer {
        &self.current_version
    }

    /// Get version as string
    pub fn version_string(&self) -> String {
        self.current_version.to_string()
    }

    /// Check compatibility with client's expected version
    pub fn check_compatibility(&self, expected_version: &str) -> CompatibilityResult {
        let expected = match SemVer::parse(expected_version) {
            Some(v) => v,
            None => {
                warn!("Failed to parse expected version: {}", expected_version);
                return CompatibilityResult::Warning {
                    message: format!("Could not parse version: {}", expected_version),
                };
            }
        };

        debug!(
            "Version check: server={}, expected={}",
            self.current_version, expected
        );

        // Check major version - must match
        if expected.major != self.current_version.major {
            return CompatibilityResult::Incompatible {
                message: format!(
                    "Major version mismatch: server={}, expected={}",
                    self.current_version, expected
                ),
            };
        }

        // Calculate minor version distance
        match expected.minor_distance(&self.current_version) {
            Some(distance) => {
                if distance == 0 {
                    // Exact minor version match (patch may differ)
                    CompatibilityResult::Compatible
                } else if distance <= self.config.version_tolerance {
                    // Within tolerance
                    if self.config.warn_only {
                        CompatibilityResult::Warning {
                            message: format!(
                                "Version difference: server={}, expected={} (within tolerance)",
                                self.current_version, expected
                            ),
                        }
                    } else {
                        CompatibilityResult::Compatible
                    }
                } else {
                    // Outside tolerance
                    CompatibilityResult::Incompatible {
                        message: format!(
                            "Version too different: server={}, expected={} (tolerance={})",
                            self.current_version, expected, self.config.version_tolerance
                        ),
                    }
                }
            }
            None => CompatibilityResult::Incompatible {
                message: format!(
                    "Major version mismatch: server={}, expected={}",
                    self.current_version, expected
                ),
            },
        }
    }

    /// Get version info for API response
    pub fn get_version_info(&self) -> VersionInfo {
        VersionInfo {
            server_version: self.version_string(),
            version_tolerance: self.config.version_tolerance,
            warn_only: self.config.warn_only,
        }
    }
}

/// Version information for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub server_version: String,
    pub version_tolerance: u32,
    pub warn_only: bool,
}

/// HTTP headers for version communication
pub mod headers {
    /// Header for client to send expected server version
    pub const EXPECTED_SERVER_VERSION: &str = "X-Expected-Server-Version";
    /// Header for server to send its version
    pub const SERVER_VERSION: &str = "X-Server-Version";
    /// Header for compatibility result
    pub const COMPATIBILITY: &str = "X-Compatibility";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parse() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let v = SemVer::parse("0.1.0").ok_or("Failed to parse version 0.1.0")?;
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 1);
        assert_eq!(v.patch, 0);

        let v = SemVer::parse("v1.2.3").ok_or("Failed to parse version v1.2.3")?;
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);

        let v = SemVer::parse("2.5").ok_or("Failed to parse version 2.5")?;
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 5);
        assert_eq!(v.patch, 0);

        Ok(())
    }

    #[test]
    fn test_minor_distance() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let v1 = SemVer::parse("1.2.0").ok_or("Failed to parse version 1.2.0")?;
        let v2 = SemVer::parse("1.3.0").ok_or("Failed to parse version 1.3.0")?;
        let v3 = SemVer::parse("2.2.0").ok_or("Failed to parse version 2.2.0")?;

        assert_eq!(v1.minor_distance(&v2), Some(1));
        assert_eq!(v2.minor_distance(&v1), Some(1));
        assert_eq!(v1.minor_distance(&v3), None); // Different major

        Ok(())
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
    }
}
