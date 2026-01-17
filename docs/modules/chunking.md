# chunking Module

**Source**: `crates/mcb-application/src/domain_services/chunking.rs` (orchestrator) and `crates/mcb-providers/src/language/` (processors)
**Crates**: `mcb-application`, `mcb-providers`

## Overview

The chunking system provides AST-based code parsing for 12 programming languages. In v0.1.1, this functionality is split across two crates:

-   **mcb-application**: ChunkingOrchestrator (domain service)
-   **mcb-providers**: Language processors (12 languages)

## Components

### ChunkingOrchestrator (`mcb-application`)

Coordinates batch chunking operations:

```rust
pub struct ChunkingOrchestrator {
    pub fn process_files(&self, files: &[PathBuf]) -> Result<Vec<CodeChunk>>;
    pub fn process_file(&self, path: &Path) -> Result<Vec<CodeChunk>>;
}
```

### Language Processors (`mcb-providers`)

Tree-sitter based processors for 12 languages:

| Processor | Language | Parser |
|-----------|----------|--------|
| `RustProcessor` | Rust | tree-sitter-rust |
| `PythonProcessor` | Python | tree-sitter-python |
| `JavaScriptProcessor` | JavaScript | tree-sitter-javascript |
| `TypeScriptProcessor` | TypeScript | tree-sitter-typescript |
| `GoProcessor` | Go | tree-sitter-go |
| `JavaProcessor` | Java | tree-sitter-java |
| `CProcessor` | C | tree-sitter-c |
| `CppProcessor` | C++ | tree-sitter-cpp |
| `CSharpProcessor` | C# | tree-sitter-c-sharp |
| `RubyProcessor` | Ruby | tree-sitter-ruby |
| `PhpProcessor` | PHP | tree-sitter-php |
| `SwiftProcessor` | Swift | tree-sitter-swift |
| `KotlinProcessor` | Kotlin | tree-sitter-kotlin-ng |

## File Structure

```text
crates/mcb-application/src/domain_services/
└── chunking.rs              # ChunkingOrchestrator

crates/mcb-providers/src/language/
├── rust.rs                  # Rust processor
├── python.rs                # Python processor
├── javascript.rs            # JavaScript processor
├── typescript.rs            # TypeScript processor
├── go.rs                    # Go processor
├── java.rs                  # Java processor
├── c.rs                     # C processor
├── cpp.rs                   # C++ processor
├── csharp.rs                # C# processor
├── ruby.rs                  # Ruby processor
├── php.rs                   # PHP processor
├── swift.rs                 # Swift processor
├── kotlin.rs                # Kotlin processor
└── mod.rs                   # Module exports
```

## Feature Flags

Language processors are controlled by feature flags in `mcb-providers`:

```toml
[features]
lang-all = ["lang-rust", "lang-python", "lang-javascript", ...]
lang-rust = ["tree-sitter-rust"]
lang-python = ["tree-sitter-python"]
# ... etc
```

## Cross-References

-   **Providers**: [providers.md](./providers.md) (language processors)
-   **Application**: [application.md](./application.md) (ChunkingOrchestrator)
-   **Domain**: [domain.md](./domain.md) (CodeChunk type)
-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

*Updated 2026-01-17 - Reflects modular crate architecture (v0.1.1)*
