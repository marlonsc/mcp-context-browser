# ADR 003: C4 Model Documentation

## Status

Accepted

> C4 model documentation structure reflects the Clean Architecture eight-crate workspace:
>
> **Diagram Artifacts** (`docs/diagrams/`):
>
> -   `system-context.puml` - System boundaries with AI providers and vector stores
> -   `container-architecture.puml` - Eight-crate workspace organization
> -   `C4-template.puml` - Template for new diagrams
>
> **Crate-Level Components** (Level 3):
>
> -   `mcb-domain` - Core entities, ports (traits), value objects
> -   `mcb-application` - Use cases, services, domain orchestration
> -   `mcb-providers` - Provider implementations (embedding, vector_store, language)
> -   `mcb-infrastructure` - DI modules, configuration, cross-cutting concerns
> -   `mcb-server` - MCP protocol handlers, admin API
> -   `mcb-validate` - Architecture validation tooling
> -   `mcb` - Public API facade

## Context

The MCP Context Browser is a complex system with multiple architectural layers, external integrations, and evolving requirements. The project needs comprehensive, scalable documentation that can be understood by different audiences (developers, architects, operations teams) at various levels of detail.

Current documentation challenges:

-   Architecture documentation was inconsistent and incomplete
-   Different audiences needed different levels of detail
-   Visual diagrams were missing or outdated
-   No standardized approach to documenting architectural decisions
-   Difficulty communicating system complexity to stakeholders

The team needed a structured approach to architecture documentation that would scale with the project and provide clear communication channels.

## Decision

Adopt the C4 model for architecture documentation, using PlantUML for diagram generation and Markdown for structured documentation. The C4 model provides four levels of architectural detail with clear scope and audience definitions.

Implementation approach:

-   **Context Diagrams**: System-level overview for non-technical stakeholders
-   **Container Diagrams**: High-level technical overview for technical stakeholders
-   **Component Diagrams**: Detailed design for developers
-   **Code Diagrams**: Implementation-level detail for maintainers
-   PlantUML for consistent, version-controlled diagram generation
-   Structured Markdown documentation with clear navigation
-   Automated diagram validation and generation

## Consequences

C4 model provides excellent structure and scalability but requires discipline in maintaining multiple levels of documentation.

### Positive Consequences

-   **Clear Structure**: Four well-defined levels of architectural detail
-   **Audience-Specific**: Different views for different stakeholders
-   **Consistency**: Standardized notation and format
-   **Maintainability**: Modular documentation that can evolve
-   **Tooling Support**: Rich ecosystem of tools and integrations
-   **Communication**: Better understanding across technical and non-technical teams

### Negative Consequences

-   **Documentation Overhead**: Multiple diagrams and documents to maintain
-   **Learning Curve**: Team needs to learn C4 notation and concepts
-   **Maintenance Burden**: Diagrams can become outdated if not maintained
-   **Tool Complexity**: PlantUML syntax has learning curve
-   **Scope Management**: Need to decide what belongs at each level

## Alternatives Considered

### Alternative 1: Free-Form Documentation

-   **Description**: Custom documentation structure without formal methodology
-   **Pros**: Flexible, no learning curve, quick to start
-   **Cons**: Inconsistent, hard to navigate, doesn't scale
-   **Rejection Reason**: Leads to poor documentation quality and maintenance issues

### Alternative 2: UML Only

-   **Description**: Traditional UML diagrams for all architectural views
-   **Pros**: Formal notation, detailed modeling capabilities
-   **Cons**: Too technical for non-technical audiences, complex to maintain
-   **Rejection Reason**: Overkill for our needs and doesn't serve diverse audiences well

### Alternative 3: Arc42 Template

-   **Description**: Comprehensive architecture documentation template
-   **Pros**: Very thorough, covers all aspects, proven methodology
-   **Cons**: Too heavyweight, would take too long to implement fully
-   **Rejection Reason**: Overkill for current team size and project stage

### Alternative 4: 4+1 Architectural View Model

-   **Description**: Rational Unified Process architectural views
-   **Pros**: Formal methodology, comprehensive coverage
-   **Cons**: Complex to understand and maintain, academic focus
-   **Rejection Reason**: Too complex for practical use in agile development

## Implementation Notes

### C4 Level Structure

#### Level 1: System Context

```text
Purpose: Show how the system fits into the world
Audience: Everyone (technical and non-technical)
Content: System boundaries, users, external systems
Notation: Simple boxes and arrows
```

#### Level 2: Container Architecture (Cargo Workspace)

```text
Purpose: Show the eight-crate Clean Architecture organization
Audience: Technical stakeholders
Content: Crate boundaries, dependencies, technology choices
Notation: Crates with layer annotations

Current Containers:
┌─────────────────────────────────────────────────────────┐
│                     mcb-server                          │  Layer 5: Protocol
│                  (MCP Protocol, Admin)                  │
├─────────────────────────────────────────────────────────┤
│                  mcb-infrastructure                     │  Layer 4: Infrastructure
│              (DI, Config, Cross-cutting)                │
├──────────────────┬──────────────────────────────────────┤
│  mcb-application │        mcb-providers                 │  Layers 2-3
│   (Use Cases)    │    (Provider Implementations)        │
├──────────────────┴──────────────────────────────────────┤
│                      mcb-domain                         │  Layer 1: Domain
│               (Ports, Entities, Value Objects)          │
└─────────────────────────────────────────────────────────┘
```

#### Level 3: Component Architecture (Crate Internals)

```text
Purpose: Show component design within each crate
Audience: Developers and architects
Content: Modules, port traits, adapter implementations
Notation: Shaku modules, port/adapter relationships

Example (mcb-infrastructure components):
-   di/modules/       → Shaku module definitions
-   di/factory/       → Provider factories
-   config/           → Configuration management
-   adapters/         → Null adapters for testing
```

#### Level 4: Code Architecture (Port/Adapter Patterns)

```text
Purpose: Show Shaku DI patterns and port implementations
Audience: Developers maintaining code
Content: Trait definitions, Component derives, interface bindings
Notation: Rust trait/impl relationships

Example:
  EmbeddingProvider (port trait in mcb-domain)
    ├── NullEmbeddingProvider (mcb-providers) - testing
    ├── OllamaEmbeddingProvider (mcb-providers) - local
    └── OpenAIEmbeddingProvider (mcb-providers) - cloud
```

### PlantUML Integration

#### Diagram Generation Pipeline

```makefile

# Makefile integration
.PHONY: docs diagrams clean-docs

diagrams: $(wildcard docs/diagrams/*.puml)
 @for file in $^; do \
  plantuml -tpng $$file; \
  plantuml -tsvg $$file; \
 done

docs: diagrams
 @echo "Generating documentation..."
 @cargo run --bin doc-generator

clean-docs:
 @rm -f docs/diagrams/*.png docs/diagrams/*.svg
```

#### Diagram Template Structure

**System Context Diagram:**

```plantuml
@startuml System Context
!include <C4/C4_Container>

title System Context - MCP Context Browser

Person(user, "AI Assistant User", "Uses AI assistants like Claude Desktop")
System(mcp, "MCP Context Browser", "Provides semantic code search using vector embeddings")

System_Ext(codebase, "Code Repository", "Git repositories with source code")
System_Ext(ai_api, "AI Provider", "OpenAI, Ollama, etc.")
System_Ext(vector_db, "Vector Database", "Milvus, In-Memory, etc.")

Rel(user, mcp, "Queries code using natural language")
Rel(mcp, codebase, "Indexes and searches code")
Rel(mcp, ai_api, "Generates embeddings")
Rel(mcp, vector_db, "Stores and retrieves vectors")

@enduml
```

**Container Architecture Diagram (Eight-Crate Workspace):**

```plantuml
@startuml Container Architecture
!include <C4/C4_Container>

title Container Architecture - MCP Context Browser (v0.1.2)

Container(server, "mcb-server", "Rust/Tokio", "MCP protocol handlers, Admin API")
Container(infra, "mcb-infrastructure", "Rust/Shaku", "DI modules, config, factories")
Container(app, "mcb-application", "Rust", "Use cases: Context, Search, Indexing")
Container(providers, "mcb-providers", "Rust", "Embedding, VectorStore, Language providers")
Container(domain, "mcb-domain", "Rust", "Ports (traits), Entities, Value Objects")
Container(validate, "mcb-validate", "Rust", "Architecture validation tooling")
Container(facade, "mcb", "Rust", "Public API facade and re-exports")

Rel(server, infra, "Uses DI container")
Rel(server, app, "Calls use cases")
Rel(infra, app, "Wires services")
Rel(infra, providers, "Creates providers via factories")
Rel(app, domain, "Implements business logic")
Rel(providers, domain, "Implements port traits")

@enduml
```

### Documentation Automation

#### CI/CD Integration

```yaml

# .github/workflows/docs.yml
name: Documentation
on:
  push:
    branches: [ main ]
    paths:
-   'docs/**'
-   'src/**'
-   'ARCHITECTURE.md'

jobs:
  validate-diagrams:
    runs-on: ubuntu-latest
    steps:
-   uses: actions/checkout@v3
-   name: Validate PlantUML
        uses: cloudbees/plantuml-github-action@master
        with:
          args: -v -checkmetadata docs/diagrams/*.puml

  build-docs:
    runs-on: ubuntu-latest
    steps:
-   uses: actions/checkout@v3
-   name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
-   name: Generate Diagrams
        run: make diagrams
-   name: Build Documentation
        run: make docs
-   name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./docs/build
```

#### Documentation Validation

```rust
pub struct DocumentationValidator {
    plantuml_path: PathBuf,
    markdown_files: Vec<PathBuf>,
}

impl DocumentationValidator {
    pub async fn validate_all(&self) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();

        // Validate PlantUML syntax
        for diagram in &self.plantuml_files() {
            if !self.validate_plantuml(diagram).await? {
                report.add_error(format!("Invalid PlantUML: {}", diagram.display()));
            }
        }

        // Validate Markdown links
        for doc in &self.markdown_files {
            if !self.validate_markdown_links(doc).await? {
                report.add_warning(format!("Broken links in: {}", doc.display()));
            }
        }

        // Validate diagram consistency
        if !self.validate_diagram_consistency().await? {
            report.add_warning("Diagrams may be inconsistent with code".to_string());
        }

        Ok(report)
    }
}
```

### Documentation Maintenance

#### Change Tracking

```rust
#[derive(Serialize, Deserialize)]
pub struct ArchitectureChange {
    pub timestamp: DateTime<Utc>,
    pub component: String,
    pub change_type: ChangeType,
    pub description: String,
    pub affected_diagrams: Vec<String>,
    pub requires_review: bool,
}

pub struct DocumentationTracker {
    changes: Vec<ArchitectureChange>,
}

impl DocumentationTracker {
    pub async fn record_change(&mut self, change: ArchitectureChange) -> Result<()> {
        self.changes.push(change);

        // Update affected diagrams
        for diagram in &change.affected_diagrams {
            self.update_diagram_timestamp(diagram).await?;
        }

        // Trigger review if needed
        if change.requires_review {
            self.notify_reviewers(&change).await?;
        }

        Ok(())
    }
}
```

### Future Enhancements (v0.1.0 "Documentation Excellence")

The v0.1.0 release will significantly enhance the C4 model documentation through automation and advanced tooling:

#### 🤖 Automated Diagram Generation

-   **Code Analysis Integration**: `cargo-modules` and `rust-code-analysis` for automatic component discovery
-   **Dependency Graph Generation**: Interactive dependency graphs from source code analysis
-   **Real-time Updates**: Diagrams automatically updated when code changes

#### 📊 Advanced Visualization

-   **Interactive Diagrams**: Web-based interactive C4 diagrams with drill-down capabilities
-   **Cross-References**: Links between diagrams, code, and documentation
-   **Search Integration**: Full-text search across all architectural documentation

#### 🔍 Quality Assurance

-   **Automated Validation**: ADR compliance checking against architectural diagrams
-   **Consistency Checks**: Automated verification that diagrams match implementation
-   **Change Tracking**: Automated detection of architectural drift

#### 📈 Metrics and Analytics

-   **Documentation Coverage**: Automated tracking of architectural documentation completeness
-   **Quality Scoring**: A+ grade standards for architectural documentation
-   **Maintenance Analytics**: Tracking documentation maintenance burden reduction

## Related ADRs

-   [ADR-001: Provider Pattern Architecture](001-provider-pattern-architecture.md) - Port/adapter pattern documented
-   [ADR-012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) - DI architecture diagrams
-   [ADR-013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Eight-crate organization

## References

-   [C4 Model Website](https://c4model.com/)
-   [PlantUML Documentation](https://plantuml.com/)
-   [Structurizr - C4 Tooling](https://structurizr.com/)
-   [The C4 model for visualising software architecture](https://www.infoq.com/articles/C4-architecture-model/)
-   [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
-   [Shaku Documentation](https://docs.rs/shaku)
