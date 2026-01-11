//! Language-specific processors and configurations
//!
//! This module contains all language-specific chunking configurations
//! and processors for supported programming languages.
//!
//! Each language has its own file with a dedicated processor that implements
//! the `LanguageProcessor` trait for AST-based code chunking.

// Language processor modules
mod c;
mod cpp;
mod csharp;
mod go;
mod java;
mod javascript;
mod kotlin;
mod php;
mod python;
mod ruby;
mod rust;
mod swift;

// Re-export all processors
pub use c::CProcessor;
pub use cpp::CppProcessor;
pub use csharp::CSharpProcessor;
pub use go::GoProcessor;
pub use java::JavaProcessor;
pub use javascript::JavaScriptProcessor;
pub use kotlin::KotlinProcessor;
pub use php::PhpProcessor;
pub use python::PythonProcessor;
pub use ruby::RubyProcessor;
pub use rust::RustProcessor;
pub use swift::SwiftProcessor;
