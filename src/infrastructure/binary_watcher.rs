//! Binary File Watcher for Auto-Respawn
//!
//! Monitors the server's executable file for updates and triggers
//! respawn events when a new version is detected.

use crate::infrastructure::events::{SharedEventBus, SystemEvent};
use anyhow::{Context, Result};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Configuration for binary watching
#[derive(Debug, Clone)]
pub struct BinaryWatcherConfig {
    /// Path to the binary to watch (defaults to /proc/self/exe)
    pub binary_path: Option<PathBuf>,
    /// Debounce duration to ensure file write is complete
    pub debounce_duration: Duration,
    /// Whether to automatically respawn on update
    pub auto_respawn: bool,
}

impl Default for BinaryWatcherConfig {
    fn default() -> Self {
        Self {
            binary_path: None, // Will resolve to /proc/self/exe
            debounce_duration: Duration::from_secs(3),
            auto_respawn: true,
        }
    }
}

/// Watches the server binary for updates and triggers respawn events
pub struct BinaryWatcher {
    config: BinaryWatcherConfig,
    event_bus: SharedEventBus,
    running: Arc<AtomicBool>,
    binary_path: PathBuf,
}

impl BinaryWatcher {
    /// Create a new binary watcher
    pub fn new(event_bus: SharedEventBus, config: BinaryWatcherConfig) -> Result<Self> {
        let binary_path = match &config.binary_path {
            Some(p) => p.clone(),
            None => {
                std::fs::read_link("/proc/self/exe").context("Failed to read /proc/self/exe")?
            }
        };

        info!("Binary watcher initialized for: {}", binary_path.display());

        Ok(Self {
            config,
            event_bus,
            running: Arc::new(AtomicBool::new(false)),
            binary_path,
        })
    }

    /// Create with default configuration
    pub fn with_defaults(event_bus: SharedEventBus) -> Result<Self> {
        Self::new(event_bus, BinaryWatcherConfig::default())
    }

    /// Get the binary path being watched
    pub fn binary_path(&self) -> &PathBuf {
        &self.binary_path
    }

    /// Check if the watcher is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Start watching the binary file
    pub async fn start(&self) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            warn!("Binary watcher already running");
            return Ok(());
        }

        let binary_path = self.binary_path.clone();
        let event_bus = Arc::clone(&self.event_bus);
        let running = Arc::clone(&self.running);
        let debounce = self.config.debounce_duration;
        let auto_respawn = self.config.auto_respawn;

        tokio::spawn(async move {
            if let Err(e) =
                run_watcher_loop(binary_path, event_bus, running, debounce, auto_respawn).await
            {
                warn!("Binary watcher error: {}", e);
            }
        });

        info!(
            "Binary watcher started with {}s debounce",
            self.config.debounce_duration.as_secs()
        );
        Ok(())
    }

    /// Stop the binary watcher
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        info!("Binary watcher stopped");
    }
}

/// Internal watcher loop
async fn run_watcher_loop(
    binary_path: PathBuf,
    event_bus: SharedEventBus,
    running: Arc<AtomicBool>,
    debounce: Duration,
    auto_respawn: bool,
) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<Event>(100);

    // Create file watcher
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        },
        Config::default(),
    )?;

    // Watch the parent directory (binary might be replaced via rename)
    let watch_path = binary_path.parent().unwrap_or(&binary_path).to_path_buf();

    watcher.watch(&watch_path, RecursiveMode::NonRecursive)?;
    debug!("Watching directory: {}", watch_path.display());

    let mut last_event_time: Option<Instant> = None;
    let mut pending_respawn = false;

    while running.load(Ordering::SeqCst) {
        tokio::select! {
            Some(event) = rx.recv() => {
                // Check if event is for our binary
                let is_our_binary = event.paths.iter().any(|p| {
                    p == &binary_path ||
                    p.file_name() == binary_path.file_name()
                });

                if is_our_binary && is_update_event(&event.kind) {
                    debug!("Binary update detected: {:?}", event.kind);
                    last_event_time = Some(Instant::now());
                    pending_respawn = true;
                }
            }

            _ = tokio::time::sleep(Duration::from_millis(500)) => {
                // Check if debounce period has passed
                if pending_respawn {
                    if let Some(last_time) = last_event_time {
                        if last_time.elapsed() >= debounce {
                            // Verify binary is stable
                            if is_binary_stable(&binary_path).await {
                                info!("Binary update confirmed, file is stable");

                                // Publish event
                                let path_str = binary_path.to_string_lossy().to_string();
                                if let Err(e) = event_bus.publish(SystemEvent::BinaryUpdated {
                                    path: path_str
                                }) {
                                    warn!("Failed to publish BinaryUpdated event: {}", e);
                                }

                                // Trigger respawn if auto_respawn is enabled
                                if auto_respawn {
                                    if let Err(e) = event_bus.publish(SystemEvent::Respawn) {
                                        warn!("Failed to publish Respawn event: {}", e);
                                    }
                                }

                                pending_respawn = false;
                                last_event_time = None;
                            } else {
                                debug!("Binary not yet stable, waiting...");
                            }
                        }
                    }
                }

                // Check if we should stop
                if !running.load(Ordering::SeqCst) {
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Check if the event kind indicates a file update
fn is_update_event(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}

/// Check if the binary file is stable (not being written)
async fn is_binary_stable(path: &PathBuf) -> bool {
    // Get initial file size
    let initial_size = match tokio::fs::metadata(path).await {
        Ok(m) => m.len(),
        Err(_) => return false,
    };

    // Wait a moment
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Check size again
    let final_size = match tokio::fs::metadata(path).await {
        Ok(m) => m.len(),
        Err(_) => return false,
    };

    // Size should be stable and file should be readable
    if initial_size != final_size {
        return false;
    }

    // Try to open the file to verify it's not being written
    tokio::fs::File::open(path).await.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::events::EventBus;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_binary_watcher_config_defaults() {
        let config = BinaryWatcherConfig::default();
        assert!(config.binary_path.is_none());
        assert_eq!(config.debounce_duration, Duration::from_secs(3));
        assert!(config.auto_respawn);
    }

    #[tokio::test]
    async fn test_binary_watcher_creation() {
        let event_bus = Arc::new(EventBus::default());
        let dir = tempdir().unwrap();
        let binary_path = dir.path().join("test_binary");
        std::fs::write(&binary_path, "test").unwrap();

        let config = BinaryWatcherConfig {
            binary_path: Some(binary_path.clone()),
            ..Default::default()
        };

        let watcher = BinaryWatcher::new(event_bus, config).unwrap();
        assert_eq!(watcher.binary_path(), &binary_path);
        assert!(!watcher.is_running());
    }

    #[tokio::test]
    async fn test_is_update_event() {
        assert!(is_update_event(&EventKind::Create(
            notify::event::CreateKind::File
        )));
        assert!(is_update_event(&EventKind::Modify(
            notify::event::ModifyKind::Data(notify::event::DataChange::Content)
        )));
        assert!(!is_update_event(&EventKind::Access(
            notify::event::AccessKind::Read
        )));
    }
}
