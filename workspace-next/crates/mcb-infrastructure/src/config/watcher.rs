//! Configuration file watcher for hot-reloading
//!
//! Provides automatic configuration reloading when the configuration file changes.

use crate::config::data::AppConfig;
use crate::config::loader::ConfigLoader;
use crate::error_ext::ErrorContext;
use crate::logging::log_config_loaded;
use mcb_domain::error::{Error, Result};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::sync::RwLock;

/// Configuration watch event
#[derive(Debug, Clone)]
pub enum ConfigWatchEvent {
    /// Configuration reloaded successfully
    Reloaded(Box<AppConfig>),
    /// Configuration reload failed
    ReloadFailed(String),
    /// Watcher started
    Started,
    /// Watcher stopped
    Stopped,
}

/// Configuration watcher for hot-reloading
pub struct ConfigWatcher {
    config_path: PathBuf,
    loader: ConfigLoader,
    current_config: Arc<RwLock<AppConfig>>,
    event_sender: Sender<ConfigWatchEvent>,
    _watcher: RecommendedWatcher,
}

impl ConfigWatcher {
    /// Create a new configuration watcher
    pub async fn new(config_path: PathBuf, initial_config: AppConfig) -> Result<Self> {
        let (event_sender, _) = broadcast::channel(16);
        let current_config = Arc::new(RwLock::new(initial_config));
        let loader = ConfigLoader::new().with_config_path(&config_path);

        // Create file watcher
        let mut watcher = Self::create_file_watcher(
            config_path.clone(),
            Arc::clone(&current_config),
            loader.clone(),
            event_sender.clone(),
        )
        .await?;

        // Watch the configuration file
        watcher
            .watch(&config_path, RecursiveMode::NonRecursive)
            .context("Failed to watch configuration file")?;

        Ok(Self {
            config_path,
            loader,
            current_config,
            event_sender,
            _watcher: watcher,
        })
    }

    /// Get the current configuration
    pub async fn get_config(&self) -> AppConfig {
        self.current_config.read().await.clone()
    }

    /// Subscribe to configuration change events
    pub fn subscribe(&self) -> Receiver<ConfigWatchEvent> {
        self.event_sender.subscribe()
    }

    /// Manually trigger a configuration reload
    pub async fn reload(&self) -> Result<AppConfig> {
        let new_config = self.loader.load()?;

        // Update current config
        *self.current_config.write().await = new_config.clone();

        // Send reload event
        let _ = self
            .event_sender
            .send(ConfigWatchEvent::Reloaded(Box::new(new_config.clone())));

        log_config_loaded(&self.config_path, true);

        Ok(new_config)
    }

    /// Get the configuration file path
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Create the file watcher
    async fn create_file_watcher(
        config_path: PathBuf,
        current_config: Arc<RwLock<AppConfig>>,
        loader: ConfigLoader,
        event_sender: Sender<ConfigWatchEvent>,
    ) -> Result<RecommendedWatcher> {
        let config_path_clone = config_path.clone();
        // Capture the Tokio runtime handle to use from the notify callback thread
        let runtime_handle = Handle::current();

        let watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                let config_path = config_path_clone.clone();
                let current_config = Arc::clone(&current_config);
                let loader = loader.clone();
                let event_sender = event_sender.clone();

                // Use the captured runtime handle to spawn tasks from the notify thread
                runtime_handle.spawn(async move {
                    match res {
                        Ok(event) => {
                            if Self::should_reload_config(&event) {
                                Self::handle_config_change(
                                    config_path,
                                    current_config,
                                    loader,
                                    event_sender,
                                )
                                .await;
                            }
                        }
                        Err(e) => {
                            let _ = event_sender.send(ConfigWatchEvent::ReloadFailed(format!(
                                "File watch error: {}",
                                e
                            )));
                        }
                    }
                });
            },
            Config::default(),
        )
        .context("Failed to create file watcher")?;

        Ok(watcher)
    }

    /// Check if the file event should trigger a config reload
    fn should_reload_config(event: &Event) -> bool {
        // Only reload on write or create events
        matches!(
            event.kind,
            notify::EventKind::Modify(notify::event::ModifyKind::Data(_))
                | notify::EventKind::Modify(notify::event::ModifyKind::Any)
                | notify::EventKind::Create(_)
        )
    }

    /// Handle configuration file change
    async fn handle_config_change(
        config_path: PathBuf,
        current_config: Arc<RwLock<AppConfig>>,
        loader: ConfigLoader,
        event_sender: Sender<ConfigWatchEvent>,
    ) {
        // Add a small delay to avoid reading partially written files
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        match loader.load() {
            Ok(new_config) => {
                // Update current config
                *current_config.write().await = new_config.clone();

                // Send reload event
                let _ = event_sender.send(ConfigWatchEvent::Reloaded(Box::new(new_config)));

                log_config_loaded(&config_path, true);
            }
            Err(e) => {
                let error_msg = format!("Failed to reload configuration: {}", e);
                let _ = event_sender.send(ConfigWatchEvent::ReloadFailed(error_msg));

                log_config_loaded(&config_path, false);
            }
        }
    }
}

/// Configuration watcher builder
pub struct ConfigWatcherBuilder {
    config_path: Option<PathBuf>,
    initial_config: Option<AppConfig>,
}

impl ConfigWatcherBuilder {
    /// Create a new configuration watcher builder
    pub fn new() -> Self {
        Self {
            config_path: None,
            initial_config: None,
        }
    }

    /// Set the configuration file path
    pub fn with_config_path<P: AsRef<std::path::Path>>(mut self, path: P) -> Self {
        self.config_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the initial configuration
    pub fn with_initial_config(mut self, config: AppConfig) -> Self {
        self.initial_config = Some(config);
        self
    }

    /// Build the configuration watcher
    pub async fn build(self) -> Result<ConfigWatcher> {
        let config_path = self.config_path.ok_or_else(|| Error::Configuration {
            message: "Configuration file path is required".to_string(),
            source: None,
        })?;

        let initial_config = self.initial_config.ok_or_else(|| Error::Configuration {
            message: "Initial configuration is required".to_string(),
            source: None,
        })?;

        ConfigWatcher::new(config_path, initial_config).await
    }
}

impl Default for ConfigWatcherBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration watcher utilities
pub struct ConfigWatcherUtils;

impl ConfigWatcherUtils {
    /// Create a watcher that automatically reloads on file changes
    pub async fn watch_config_file(
        config_path: PathBuf,
        initial_config: AppConfig,
    ) -> Result<ConfigWatcher> {
        ConfigWatcher::new(config_path, initial_config).await
    }

    /// Check if file watching is supported on the current platform
    pub fn is_file_watching_supported() -> bool {
        // File watching is generally supported on most platforms
        // but can be disabled in some environments
        !std::env::var("DISABLE_CONFIG_WATCHING")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
    }
}
