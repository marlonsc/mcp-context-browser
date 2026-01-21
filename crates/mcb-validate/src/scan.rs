//! Shared file-scanning helpers for validators.

use crate::{Result, ValidationConfig};
use std::path::Path;
use walkdir::WalkDir;

/// True if a path points to a test file or tests directory.
pub fn is_test_path(path: &str) -> bool {
    path.contains("_test.rs") || path.contains("/tests/")
}

/// Iterate over Rust source files in each crate's `src` directory.
pub fn for_each_crate_rs_path<F>(config: &ValidationConfig, mut f: F) -> Result<()>
where
    F: FnMut(&Path, &Path, &str) -> Result<()>,
{
    for crate_dir in config.get_source_dirs()? {
        let src_dir = crate_dir.join("src");
        if !src_dir.exists() {
            continue;
        }

        let crate_name = crate_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

        for entry in WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            f(entry.path(), &src_dir, crate_name)?;
        }
    }

    Ok(())
}

/// Iterate over Rust source files in configured scan directories.
pub fn for_each_scan_rs_path<F>(
    config: &ValidationConfig,
    skip_validate_crate: bool,
    mut f: F,
) -> Result<()>
where
    F: FnMut(&Path, &Path) -> Result<()>,
{
    for src_dir in config.get_scan_dirs()? {
        if skip_validate_crate && src_dir.to_string_lossy().contains("mcb-validate") {
            continue;
        }

        for entry in WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            f(entry.path(), &src_dir)?;
        }
    }

    Ok(())
}
