# ADR 003: C4 Model Documentation

## Status

Accepted

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

```
Purpose: Show how the system fits into the world
Audience: Everyone (technical and non-technical)
Content: System boundaries, users, external systems
Notation: Simple boxes and arrows
```

#### Level 2: Container Architecture

```
Purpose: Show high-level technology choices
Audience: Technical stakeholders
Content: Containers, technologies, communication patterns
Notation: Containers with technology labels
```

#### Level 3: Component Architecture

```
Purpose: Show component design and responsibilities
Audience: Developers and architects
Content: Components, interfaces, data flows
Notation: Detailed component relationships
```

#### Level 4: Code Architecture

```
Purpose: Show implementation details
Audience: Developers maintaining code
Content: Classes, interfaces, implementation patterns
Notation: UML class diagrams, sequence diagrams
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

```plantuml
@startuml System Context
!include <C4/C4_Container>

title System Context - MCP Context Browser

Person(user, "AI Assistant User", "Uses AI assistants like Claude Desktop")
System(mcp, "MCP Context Browser", "Provides semantic code search using vector embeddings")

System_Ext(codebase, "Code Repository", "Git repositories with source code")
System_Ext(ai_api, "AI Provider", "OpenAI, Ollama, etc.")
System_Ext(vector_db, "Vector Database", "Milvus, Pinecone, etc.")

Rel(user, mcp, "Queries code using natural language")
Rel(mcp, codebase, "Indexes and searches code")
Rel(mcp, ai_api, "Generates embeddings")
Rel(mcp, vector_db, "Stores and retrieves vectors")

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
      - 'docs/**'
      - 'src/**'
      - 'ARCHITECTURE.md'

jobs:
  validate-diagrams:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Validate PlantUML
        uses: cloudbees/plantuml-github-action@master
        with:
          args: -v -checkmetadata docs/diagrams/*.puml

  build-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Generate Diagrams
        run: make diagrams
      - name: Build Documentation
        run: make docs
      - name: Deploy to GitHub Pages
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

## References

-   [C4 Model Website](https://c4model.com/)
-   [PlantUML Documentation](https://plantuml.com/)
-   [Structurizr - C4 Tooling](https://structurizr.com/)
-   [The C4 model for visualising software architecture](https://www.infoq.com/articles/C4-architecture-model/)
