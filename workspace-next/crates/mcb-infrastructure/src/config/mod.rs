//! Configuration management with hot-reload capabilities
//!
//! Provides TOML configuration loading, validation, and hot-reloading
//! for all system components. This module manages the application's
//! configuration lifecycle.

pub mod data;
pub mod loader;
pub mod providers;
pub mod server;
pub mod types;
pub mod watcher;

pub use data::*;
pub use loader::*;
pub use providers::*;
pub use server::*;
pub use types::*;
pub use watcher::*;