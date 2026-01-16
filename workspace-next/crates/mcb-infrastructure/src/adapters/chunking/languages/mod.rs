//! Language-specific processors for AST-based code chunking
//!
//! Each language has its own processor that implements the LanguageProcessor trait.

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
