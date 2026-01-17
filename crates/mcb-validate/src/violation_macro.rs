//! Violation Definition Macro
//!
//! Provides a declarative macro for defining violation enums with
//! automatic trait implementations.
//!
//! # Example
//!
//! ```ignore
//! define_violations! {
//!     ViolationCategory::Architecture,
//!     pub enum DependencyViolation {
//!         #[violation(
//!             id = "DEP001",
//!             severity = Error,
//!             message = "Forbidden Cargo dependency: {crate_name} depends on {forbidden_dep}"
//!         )]
//!         ForbiddenCargoDependency {
//!             crate_name: String,
//!             forbidden_dep: String,
//!             file: PathBuf,
//!             line: usize,
//!         },
//!     }
//! }
//! ```

/// Macro to define violation enums with automatic trait implementations
///
/// This macro generates:
/// - The enum with all variants
/// - `Display` implementation with formatted messages
/// - `Violation` trait implementation
///
/// # Parameters
///
/// - `$category`: The `ViolationCategory` for all variants
/// - `$vis`: Visibility modifier (pub, pub(crate), etc.)
/// - `$name`: Name of the enum
/// - For each variant:
///   - `id`: Unique violation identifier (e.g., "DEP001")
///   - `severity`: Error, Warning, or Info
///   - `message`: Display message (can use {field_name} placeholders)
///   - `suggestion` (optional): Suggested fix
///   - Fields must include `file: PathBuf` and `line: usize` for location tracking
#[macro_export]
macro_rules! define_violations {
    (
        $category:expr,
        $vis:vis enum $name:ident {
            $(
                #[violation(
                    id = $id:literal,
                    severity = $severity:ident
                    $(, message = $msg:literal)?
                    $(, suggestion = $suggestion:literal)?
                )]
                $variant:ident {
                    $( $field:ident : $field_ty:ty ),* $(,)?
                }
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, serde::Serialize)]
        $vis enum $name {
            $( $variant { $( $field: $field_ty ),* } ),*
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            define_violations!(@format f, $($msg,)? $( $field ),*)
                        }
                    ),*
                }
            }
        }

        impl $crate::violation_trait::Violation for $name {
            fn id(&self) -> &str {
                match self {
                    $( Self::$variant { .. } => $id ),*
                }
            }

            fn category(&self) -> $crate::violation_trait::ViolationCategory {
                $category
            }

            fn severity(&self) -> $crate::violation_trait::Severity {
                match self {
                    $( Self::$variant { .. } => $crate::violation_trait::Severity::$severity ),*
                }
            }

            fn file(&self) -> Option<&std::path::PathBuf> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            define_violations!(@get_file $( $field : $field_ty ),*)
                        }
                    ),*
                }
            }

            fn line(&self) -> Option<usize> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            define_violations!(@get_line $( $field : $field_ty ),*)
                        }
                    ),*
                }
            }

            fn suggestion(&self) -> Option<String> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            define_violations!(@suggestion $($suggestion,)? $( $field ),*)
                        }
                    ),*
                }
            }
        }
    };

    // Format helper - with message template
    (@format $f:ident, $msg:literal, $( $field:ident ),*) => {
        write!($f, $msg, $( $field = $field ),*)
    };

    // Format helper - no message template (use Debug)
    (@format $f:ident, $( $field:ident ),*) => {
        write!($f, "{:?}", ($( $field ),*))
    };

    // Get file field helper
    (@get_file $( $field:ident : $field_ty:ty ),*) => {{
        $(
            define_violations!(@check_file_field $field : $field_ty);
        )*
        None
    }};

    (@check_file_field file : PathBuf) => { return Some(file) };
    (@check_file_field file : std::path::PathBuf) => { return Some(file) };
    (@check_file_field location : PathBuf) => { return Some(location) };
    (@check_file_field location : std::path::PathBuf) => { return Some(location) };
    (@check_file_field $field:ident : $field_ty:ty) => {};

    // Get line field helper
    (@get_line $( $field:ident : $field_ty:ty ),*) => {{
        $(
            define_violations!(@check_line_field $field : $field_ty);
        )*
        None
    }};

    (@check_line_field line : usize) => { return Some(*line) };
    (@check_line_field $field:ident : $field_ty:ty) => {};

    // Suggestion helper - with suggestion template
    (@suggestion $suggestion:literal, $( $field:ident ),*) => {
        Some(format!($suggestion, $( $field = $field ),*))
    };

    // Suggestion helper - no suggestion
    (@suggestion $( $field:ident ),*) => {
        None
    };
}

// Tests are moved to integration tests to avoid macro expansion issues
// with unused format arguments in the test module.
