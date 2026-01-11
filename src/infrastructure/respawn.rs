//! Process Respawn via exec()
//!
//! Provides functionality to replace the current process with a new
//! instance of the same binary, preserving the PID (ideal for systemd).

use anyhow::{Context, Result};
use std::ffi::CString;
use std::path::PathBuf;
use tracing::info;

/// Configuration for respawn behavior
#[derive(Debug, Clone)]
pub struct RespawnConfig {
    /// Path to the binary (defaults to /proc/self/exe)
    pub binary_path: Option<PathBuf>,
    /// Whether to use exec (true) or exit-with-restart-code (false)
    pub use_exec: bool,
    /// Exit code to use for restart signal (when use_exec is false)
    pub restart_exit_code: i32,
}

impl Default for RespawnConfig {
    fn default() -> Self {
        Self {
            binary_path: None,
            use_exec: true,
            restart_exit_code: 71, // Custom restart code for systemd
        }
    }
}

/// Respawn manager for process replacement
pub struct RespawnManager {
    config: RespawnConfig,
    binary_path: PathBuf,
}

impl RespawnManager {
    /// Create a new respawn manager
    pub fn new(config: RespawnConfig) -> Result<Self> {
        let binary_path = match &config.binary_path {
            Some(p) => p.clone(),
            None => {
                std::fs::read_link("/proc/self/exe").context("Failed to read /proc/self/exe")?
            }
        };

        Ok(Self {
            config,
            binary_path,
        })
    }

    /// Create with default configuration
    pub fn with_defaults() -> Result<Self> {
        Self::new(RespawnConfig::default())
    }

    /// Get the binary path
    pub fn binary_path(&self) -> &PathBuf {
        &self.binary_path
    }

    /// Execute the respawn
    ///
    /// If use_exec is true, this will replace the current process and NOT return.
    /// If use_exec is false, this will exit with the restart code.
    pub fn respawn(&self) -> Result<()> {
        if self.config.use_exec {
            self.exec_respawn()
        } else {
            self.exit_respawn()
        }
    }

    /// Respawn using exec() - replaces current process
    fn exec_respawn(&self) -> Result<()> {
        info!(
            "Executing respawn via exec(): {}",
            self.binary_path.display()
        );

        // Get current arguments
        let args: Vec<CString> = std::env::args()
            .map(|s| CString::new(s).expect("Argument contains null byte"))
            .collect();

        // Get current environment
        let env: Vec<CString> = std::env::vars()
            .map(|(k, v)| CString::new(format!("{}={}", k, v)).expect("Env var contains null byte"))
            .collect();

        // Convert binary path to CString
        let exe_path = CString::new(self.binary_path.to_string_lossy().as_bytes())
            .context("Binary path contains null byte")?;

        // Log before exec (won't return on success)
        info!(
            "Executing: {} with {} args, {} env vars",
            self.binary_path.display(),
            args.len(),
            env.len()
        );

        // Execute the new binary
        // This replaces the current process image, so on success, this never returns
        nix::unistd::execve(&exe_path, &args, &env).context("execve failed")?;

        // This line is never reached on successful exec
        unreachable!("execve should not return on success")
    }

    /// Respawn using exit code - for systemd Restart=on-failure
    fn exit_respawn(&self) -> Result<()> {
        info!(
            "Triggering respawn via exit code {}",
            self.config.restart_exit_code
        );

        // Exit with special code that systemd interprets as "restart me"
        std::process::exit(self.config.restart_exit_code);
    }
}

/// Convenience function to perform exec-based respawn
///
/// # Safety
/// This function will replace the current process and NOT return on success.
pub fn exec_restart() -> Result<()> {
    let manager = RespawnManager::with_defaults()?;
    manager.respawn()
}

/// Convenience function to check if respawn is available
pub fn can_respawn() -> bool {
    // Check if we can read our own binary path
    std::fs::read_link("/proc/self/exe").is_ok()
}

/// Get the current binary path
pub fn get_binary_path() -> Result<PathBuf> {
    std::fs::read_link("/proc/self/exe").context("Failed to read /proc/self/exe")
}

/// Prepare for respawn (log state, flush buffers, etc.)
pub async fn prepare_for_respawn() {
    info!("Preparing for respawn...");

    // Give time for logs to flush
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Additional cleanup could go here:
    // - Flush metrics
    // - Close database connections gracefully
    // - Save state if needed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_respawn_config_defaults() {
        let config = RespawnConfig::default();
        assert!(config.binary_path.is_none());
        assert!(config.use_exec);
        assert_eq!(config.restart_exit_code, 71);
    }

    #[test]
    fn test_respawn_manager_creation() {
        // This test may fail in some environments without /proc
        if can_respawn() {
            let manager = RespawnManager::with_defaults().unwrap();
            assert!(
                manager.binary_path().exists()
                    || manager.binary_path().to_str().unwrap().contains("target")
            );
        }
    }

    #[test]
    fn test_can_respawn() {
        // On Linux with /proc, this should work
        // On other platforms or containers without /proc, it may not
        let result = can_respawn();
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_get_binary_path() {
        if can_respawn() {
            let path = get_binary_path().unwrap();
            // The path should exist or at least be a valid path
            assert!(!path.to_string_lossy().is_empty());
        }
    }
}
