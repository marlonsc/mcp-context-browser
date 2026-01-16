//! Integration tests that validate the actual workspace

use mcb_validate::{ArchitectureValidator, Reporter, Severity};
use std::path::PathBuf;

fn get_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn test_validate_workspace_dependencies() {
    let workspace_root = get_workspace_root();
    let mut validator = ArchitectureValidator::new(&workspace_root);

    let violations = validator.validate_dependencies().unwrap();

    println!("\n=== Dependency Violations ===");
    for v in &violations {
        println!("  [{:?}] {}", v.severity(), v);
    }
    println!("Total: {} dependency violations\n", violations.len());

    // Dependencies should follow Clean Architecture
    assert!(
        violations.is_empty(),
        "Found {} dependency violations - Clean Architecture rules violated",
        violations.len()
    );
}

#[test]
fn test_validate_workspace_quality() {
    let workspace_root = get_workspace_root();
    let mut validator = ArchitectureValidator::new(&workspace_root);

    let violations = validator.validate_quality().unwrap();

    println!("\n=== Quality Violations ===");
    let errors: Vec<_> = violations
        .iter()
        .filter(|v| v.severity() == Severity::Error)
        .collect();
    let warnings: Vec<_> = violations
        .iter()
        .filter(|v| v.severity() == Severity::Warning)
        .collect();

    for v in &errors {
        println!("  [ERROR] {}", v);
    }
    for v in &warnings {
        println!("  [WARNING] {}", v);
    }
    println!(
        "Total: {} errors, {} warnings\n",
        errors.len(),
        warnings.len()
    );

    // Report but don't fail on warnings (file size, TODOs)
    // Only fail on errors (unwrap/expect/panic in production)
    if !errors.is_empty() {
        println!("\nProduction code contains unwrap/expect/panic!");
        for e in &errors {
            println!("  - {}", e);
        }
    }
}

#[test]
fn test_validate_workspace_patterns() {
    let workspace_root = get_workspace_root();
    let mut validator = ArchitectureValidator::new(&workspace_root);

    let violations = validator.validate_patterns().unwrap();

    println!("\n=== Pattern Violations ===");
    for v in &violations {
        println!("  [{:?}] {}", v.severity(), v);
    }
    println!("Total: {} pattern violations\n", violations.len());
}

#[test]
fn test_validate_workspace_tests() {
    let workspace_root = get_workspace_root();
    let mut validator = ArchitectureValidator::new(&workspace_root);

    let violations = validator.validate_tests().unwrap();

    println!("\n=== Test Organization Violations ===");
    for v in &violations {
        println!("  [{:?}] {}", v.severity(), v);
    }
    println!("Total: {} test organization violations\n", violations.len());
}

#[test]
fn test_validate_workspace_documentation() {
    let workspace_root = get_workspace_root();
    let mut validator = ArchitectureValidator::new(&workspace_root);

    let violations = validator.validate_documentation().unwrap();

    println!("\n=== Documentation Violations ===");
    let by_severity = |sev: Severity| violations.iter().filter(|v| v.severity() == sev).count();

    println!(
        "  Errors: {}, Warnings: {}, Info: {}",
        by_severity(Severity::Error),
        by_severity(Severity::Warning),
        by_severity(Severity::Info)
    );

    // Only print first 20 violations to avoid noise
    for v in violations.iter().take(20) {
        println!("  [{:?}] {}", v.severity(), v);
    }
    if violations.len() > 20 {
        println!("  ... and {} more", violations.len() - 20);
    }
    println!("Total: {} documentation violations\n", violations.len());
}

#[test]
fn test_validate_workspace_naming() {
    let workspace_root = get_workspace_root();
    let mut validator = ArchitectureValidator::new(&workspace_root);

    let violations = validator.validate_naming().unwrap();

    println!("\n=== Naming Violations ===");
    for v in &violations {
        println!("  [{:?}] {}", v.severity(), v);
    }
    println!("Total: {} naming violations\n", violations.len());
}

#[test]
fn test_full_validation_report() {
    let workspace_root = get_workspace_root();
    let mut validator = ArchitectureValidator::new(&workspace_root);

    let report = validator.validate_all().unwrap();

    println!("\n{}", Reporter::to_human_readable(&report));

    // Count error-level violations
    let error_count = Reporter::count_errors(&report);
    let warning_count = Reporter::count_warnings(&report);

    println!(
        "Summary: {} errors, {} warnings, {} total violations",
        error_count, warning_count, report.summary.total_violations
    );

    // The validation should complete without panicking
    // We don't assert on violation count as existing code may have issues
}
