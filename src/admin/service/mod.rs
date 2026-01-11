//! Admin service layer - SOLID principles implementation
//!
//! This service provides a clean interface to access system data
//! following SOLID principles and dependency injection.

mod implementation;
mod traits;
mod types;

pub use implementation::AdminServiceImpl;
pub use traits::*;
pub use types::*;
