use crate::domain::error::{Error, Result};
use arc_swap::ArcSwap;
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};

use super::loader::ConfigLoader;
use super::types::Config;

pub struct ConfigWatcher {
    config: Arc<ArcSwap<Config>>,
    loader: ConfigLoader,
    path: PathBuf,
}

impl ConfigWatcher {
    pub fn new(config: Arc<ArcSwap<Config>>, path: PathBuf) -> Self {
        Self {
            config,
            loader: ConfigLoader::new(),
            path,
        }
    }

    pub async fn watch(&self) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(1);
        let path = self.path.clone();
        let path_for_watcher = path.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    if event.kind.is_modify() || event.kind.is_create() {
                        let _ = tx.blocking_send(());
                    }
                }
            },
            NotifyConfig::default(),
        )
        .map_err(|e| Error::config(format!("Failed to create watcher: {}", e)))?;

        if path.exists() {
            watcher
                .watch(&path, RecursiveMode::NonRecursive)
                .map_err(|e| Error::config(format!("Failed to watch config file: {}", e)))?;
        } else if let Some(parent) = path.parent() {
            if parent.exists() {
                watcher
                    .watch(parent, RecursiveMode::NonRecursive)
                    .map_err(|e| {
                        Error::config(format!("Failed to watch config directory: {}", e))
                    })?;
            }
        }

        info!("Watching configuration file: {:?}", path_for_watcher);

        let config_swap = Arc::clone(&self.config);
        let loader = self.loader;
        let watch_path = path_for_watcher.clone();

        tokio::spawn(async move {
            while rx.recv().await.is_some() {
                info!("Configuration change detected, reloading...");
                match loader.load_with_file(&watch_path).await {
                    Ok(new_config) => {
                        config_swap.store(Arc::new(new_config));
                        info!("Configuration reloaded successfully");
                    }
                    Err(e) => {
                        error!("Failed to reload configuration: {}", e);
                    }
                }
            }
        });

        // Keep the watcher alive
        Box::leak(Box::new(watcher));

        Ok(())
    }
}

impl Clone for ConfigWatcher {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            loader: self.loader,
            path: self.path.clone(),
        }
    }
}
