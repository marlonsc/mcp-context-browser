//! Tests for test organization validation

#[cfg(test)]
mod tests_org_tests {
    use mcb_validate::tests_org::{TestValidator, TestViolation};
    use std::fs;
    use tempfile::TempDir;

    fn create_test_crate_with_tests(
        temp: &TempDir,
        name: &str,
        src_content: &str,
        test_content: &str,
    ) {
        let crate_dir = temp.path().join("crates").join(name);
        let src_dir = crate_dir.join("src");
        let tests_dir = crate_dir.join("tests");

        fs::create_dir_all(&src_dir).unwrap();
        fs::create_dir_all(&tests_dir).unwrap();

        fs::write(src_dir.join("lib.rs"), src_content).unwrap();
        fs::write(tests_dir.join("integration_test.rs"), test_content).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            format!(
                r#"
[package]
name = "{name}"
version = "0.1.1"
"#
            ),
        )
        .unwrap();
    }

    #[test]
    fn test_inline_test_detection() {
        let temp = TempDir::new().unwrap();
        let crate_dir = temp.path().join("crates").join("mcb-test").join("src");
        fs::create_dir_all(&crate_dir).unwrap();

        fs::write(
            crate_dir.join("lib.rs"),
            r"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }
}
",
        )
        .unwrap();

        fs::write(
            temp.path()
                .join("crates")
                .join("mcb-test")
                .join("Cargo.toml"),
            r#"
[package]
name = "mcb-test"
version = "0.1.1"
"#,
        )
        .unwrap();

        let validator = TestValidator::new(temp.path());
        let violations = validator.validate_no_inline_tests().unwrap();

        assert_eq!(violations.len(), 1);
        match &violations[0] {
            TestViolation::InlineTestModule { .. } => {}
            _ => panic!("Expected InlineTestModule"),
        }
    }

    #[test]
    fn test_function_naming_validation() {
        let temp = TempDir::new().unwrap();
        create_test_crate_with_tests(
            &temp,
            "mcb-test",
            "pub fn add(a: i32, b: i32) -> i32 { a + b }",
            r"
#[test]
fn bad_name() {
    // This test function has a bad name (doesn't start with 'test_')
    // The actual validation happens in test_function_naming_validation
    assert_eq!(2 + 2, 4); // Basic assertion to ensure test runs
}

#[test]
fn test_good_name() {
    // This test function has a good name (starts with 'test_')
    // The actual validation happens in test_function_naming_validation
    assert_eq!(2 + 2, 4); // Basic assertion to ensure test runs
}
",
        );

        let validator = TestValidator::new(temp.path());
        let violations = validator.validate_test_function_naming().unwrap();

        // Should find 1 bad name
        let bad_names: Vec<_> = violations
            .iter()
            .filter(|v| matches!(v, TestViolation::BadTestFunctionName { .. }))
            .collect();
        assert_eq!(bad_names.len(), 1);
    }
}
