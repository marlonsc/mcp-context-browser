//! Shared test utilities for mcb-validate tests
//!
//! This module provides common test helpers to avoid duplication across test files.

#![allow(dead_code)] // Test utilities may not all be used in every test file

use std::fs;
use tempfile::TempDir;

/// Create a minimal crate structure for testing
pub fn create_test_crate(temp: &TempDir, name: &str, content: &str) {
    create_test_crate_with_file(temp, name, "lib.rs", content);
}

/// Create a crate structure with a specific file name
pub fn create_test_crate_with_file(temp: &TempDir, name: &str, file_name: &str, content: &str) {
    // Create workspace Cargo.toml if it doesn't exist
    let workspace_cargo = temp.path().join("Cargo.toml");
    if !workspace_cargo.exists() {
        fs::write(
            &workspace_cargo,
            r#"[workspace]
members = ["crates/*"]
"#,
        )
        .unwrap();
    }

    // Create crate structure
    let crate_dir = temp.path().join("crates").join(name).join("src");
    fs::create_dir_all(&crate_dir).unwrap();
    fs::write(crate_dir.join(file_name), content).unwrap();

    let cargo_dir = temp.path().join("crates").join(name);
    fs::write(
        cargo_dir.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{}"
version = "0.1.1"
"#,
            name
        ),
    )
    .unwrap();
}

/// Create a crate with an additional file (not just lib.rs)
pub fn create_test_crate_with_extra_file(
    temp: &TempDir,
    name: &str,
    lib_content: &str,
    extra_file: &str,
    extra_content: &str,
) {
    create_test_crate(temp, name, lib_content);

    let crate_dir = temp.path().join("crates").join(name).join("src");
    fs::write(crate_dir.join(extra_file), extra_content).unwrap();
}

/// Create a crate structure with tests directory
pub fn create_test_crate_with_tests(
    temp: &TempDir,
    name: &str,
    lib_content: &str,
    test_content: &str,
) {
    create_test_crate(temp, name, lib_content);

    let tests_dir = temp.path().join("crates").join(name).join("tests");
    fs::create_dir_all(&tests_dir).unwrap();
    fs::write(tests_dir.join("integration_test.rs"), test_content).unwrap();
}

/// Create a file at a specific path within the temp directory
pub fn create_file_at_path(temp: &TempDir, relative_path: &str, content: &str) {
    let full_path = temp.path().join(relative_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(full_path, content).unwrap();
}

/// Get the workspace root for integration tests
pub fn get_workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Ensure temp directory has workspace structure
pub fn ensure_workspace_structure(temp: &TempDir) {
    let workspace_cargo = temp.path().join("Cargo.toml");
    if !workspace_cargo.exists() {
        fs::write(
            &workspace_cargo,
            r#"[workspace]
members = ["crates/*"]
"#,
        )
        .unwrap();
    }

    let crates_dir = temp.path().join("crates");
    if !crates_dir.exists() {
        fs::create_dir_all(&crates_dir).unwrap();
    }
}

/// Create a constants.rs file in a test crate (for testing exemptions)
pub fn create_constants_file(temp: &TempDir, name: &str, content: &str) {
    let crate_dir = temp.path().join("crates").join(name).join("src");
    fs::create_dir_all(&crate_dir).unwrap();
    fs::write(crate_dir.join("constants.rs"), content).unwrap();

    // Ensure Cargo.toml exists
    let cargo_path = temp.path().join("crates").join(name).join("Cargo.toml");
    if !cargo_path.exists() {
        fs::write(
            cargo_path,
            format!(
                r#"[package]
name = "{}"
version = "0.1.1"
"#,
                name
            ),
        )
        .unwrap();
    }
}

/// Create a null.rs file in a test crate (for testing null provider exemptions)
pub fn create_null_provider_file(temp: &TempDir, name: &str, content: &str) {
    let crate_dir = temp.path().join("crates").join(name).join("src");
    fs::create_dir_all(&crate_dir).unwrap();
    fs::write(crate_dir.join("null.rs"), content).unwrap();

    // Ensure Cargo.toml exists
    let cargo_path = temp.path().join("crates").join(name).join("Cargo.toml");
    if !cargo_path.exists() {
        fs::write(
            cargo_path,
            format!(
                r#"[package]
name = "{}"
version = "0.1.1"
"#,
                name
            ),
        )
        .unwrap();
    }
}

/// Assert that violations list is empty with descriptive message
pub fn assert_no_violations<V: std::fmt::Debug>(violations: &[V], context: &str) {
    assert!(
        violations.is_empty(),
        "{}: expected no violations, got {} - {:?}",
        context,
        violations.len(),
        violations
    );
}

/// Assert that violations list has expected count
pub fn assert_violation_count<V: std::fmt::Debug>(violations: &[V], expected: usize, context: &str) {
    assert_eq!(
        violations.len(),
        expected,
        "{}: expected {} violations, got {} - {:?}",
        context,
        expected,
        violations.len(),
        violations
    );
}
