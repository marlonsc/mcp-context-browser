//! Violation Trait
//!
//! Generic trait for all architecture violations. This enables a unified
//! way to handle violations across all validators.

use serde::Serialize;
use std::fmt::Display;
use std::path::PathBuf;

// Re-export Severity from parent module for convenience
pub use super::Severity;

/// Category of violation for grouping in reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum ViolationCategory {
    /// Architecture and layer boundaries
    Architecture,
    /// Code quality (unwrap, expect, panic)
    Quality,
    /// File organization and placement
    Organization,
    /// SOLID principles
    Solid,
    /// Dependency injection and Shaku patterns
    DependencyInjection,
    /// Performance patterns
    Performance,
    /// Async patterns
    Async,
    /// Documentation quality
    Documentation,
    /// Test organization
    Testing,
    /// Naming conventions
    Naming,
    /// KISS principle (complexity)
    Kiss,
    /// Refactoring completeness
    Refactoring,
    /// Error handling boundaries
    ErrorBoundary,
    /// Implementation quality
    Implementation,
    /// PMAT integration
    Pmat,
}

impl Display for ViolationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Architecture => write!(f, "Architecture"),
            Self::Quality => write!(f, "Quality"),
            Self::Organization => write!(f, "Organization"),
            Self::Solid => write!(f, "SOLID"),
            Self::DependencyInjection => write!(f, "DI/Shaku"),
            Self::Performance => write!(f, "Performance"),
            Self::Async => write!(f, "Async"),
            Self::Documentation => write!(f, "Documentation"),
            Self::Testing => write!(f, "Testing"),
            Self::Naming => write!(f, "Naming"),
            Self::Kiss => write!(f, "KISS"),
            Self::Refactoring => write!(f, "Refactoring"),
            Self::ErrorBoundary => write!(f, "Error Boundary"),
            Self::Implementation => write!(f, "Implementation"),
            Self::Pmat => write!(f, "PMAT"),
        }
    }
}

/// Generic violation trait - all violations implement this
///
/// This trait provides a unified interface for handling violations
/// across all validator types, enabling generic reporting and processing.
pub trait Violation: Display + Send + Sync {
    /// Unique violation ID (e.g., "DEP001", "QUAL002")
    fn id(&self) -> &str;

    /// Category for grouping in reports
    fn category(&self) -> ViolationCategory;

    /// Severity level
    fn severity(&self) -> Severity;

    /// File where violation occurred (if applicable)
    fn file(&self) -> Option<&PathBuf>;

    /// Line number where violation occurred (if applicable)
    fn line(&self) -> Option<usize>;

    /// Human-readable message describing the violation
    fn message(&self) -> String {
        self.to_string()
    }

    /// Suggested fix for the violation (if applicable)
    fn suggestion(&self) -> Option<String> {
        None
    }

    /// Convert to a boxed trait object for dynamic dispatch
    fn boxed(self) -> Box<dyn Violation>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

/// Extension trait for converting violations to boxed trait objects
pub trait ViolationExt {
    /// Convert to a vector of boxed violations
    fn into_boxed(self) -> Vec<Box<dyn Violation>>;
}

impl<T: Violation + 'static> ViolationExt for Vec<T> {
    fn into_boxed(self) -> Vec<Box<dyn Violation>> {
        self.into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect()
    }
}
