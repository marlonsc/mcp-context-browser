# MCP Context Browser - Documentation

[![Documentation Status](https://img.shields.io/badge/docs-automated-green)](https://github.com/marlonsc/mcp-context-browser/actions)
[![Version](https://img.shields.io/badge/version-0.0.2--alpha-blue)](https://github.com/marlonsc/mcp-context-browser/releases)
[![Architecture](https://img.shields.io/badge/architecture-C4--model-blue)](docs/architecture/ARCHITECTURE.md)

**Comprehensive documentation for the MCP Context Browser project**

## 📚 Documentation Structure

This documentation is organized into focused sections for different audiences and purposes:

### 📖 User Guide

User-facing documentation for installation, usage, and features.

-   **[README](user-guide/README.md)** - Project overview, quick start, and basic usage
-   **[Features](user-guide/README.md#current-capabilities-v001)** - Current capabilities and features

### 🛠️ Developer Guide

Documentation for developers contributing to the project.

-   **[Contributing](developer/CONTRIBUTING.md)** - Development setup and contribution guidelines
-   **[Roadmap](developer/ROADMAP.md)** - Development roadmap and milestones

### 🏗️ Architecture

Technical architecture documentation following C4 model principles.

-   **[Architecture Overview](architecture/ARCHITECTURE.md)** - Comprehensive system architecture
-   **[Architecture Diagrams](architecture/diagrams/)** - Visual architecture documentation
    -   [System Context](architecture/diagrams/generated/index.html) - System boundaries and external systems
    -   [Container Architecture](architecture/diagrams/generated/index.html) - Service and deployment architecture
-   **[Architecture Decision Records](architecture/adr/)** - Historical architectural decisions
    -   [ADR 001: Provider Pattern](architecture/adr/001-provider-pattern-architecture.md)
    -   [ADR 002: Async-First Architecture](architecture/adr/002-async-first-architecture.md)
    -   [ADR 003: C4 Model Documentation](architecture/adr/003-c4-model-documentation.md)
    -   [ADR 004: Multi-Provider Strategy](architecture/adr/004-multi-provider-strategy.md)

### 🚀 Operations

Operational documentation for deployment and maintenance.

-   **[Deployment Guide](operations/DEPLOYMENT.md)** - Deployment configurations and environments
-   **[Changelog](operations/CHANGELOG.md)** - Version history and release notes

### 📋 Templates

Documentation templates and standards.

-   **[ADR Template](templates/adr-template.md)** - Template for new Architecture Decision Records

## 🔧 Documentation Automation

This documentation is fully automated and validated. Use these commands:

```bash
# Generate all documentation
make docs

# Validate documentation structure
make validate-docs

# Generate architecture diagrams
make diagrams

# Check documentation consistency
make docs-consistency

# Full documentation CI pipeline
make docs-ci
```

## 📊 Documentation Quality

| Aspect | Status | Description |
|--------|--------|-------------|
| **Automation** | ✅ Automated | Fully automated generation and validation |
| **Consistency** | ✅ Validated | Cross-references and structure validation |
| **Architecture** | ✅ C4 Model | Structured architectural documentation |
| **Diagrams** | ✅ Generated | PlantUML-generated architecture diagrams |
| **Validation** | ✅ CI/CD | Automated validation in CI pipeline |

## 🎯 Documentation Principles

1.  **Single Source of Truth** - Documentation stays synchronized with code
2.  **Audience-Specific** - Different views for different stakeholders
3.  **Automated Maintenance** - No manual updates required
4.  **Version Controlled** - All documentation is version controlled
5.  **Quality Assured** - Automated validation and consistency checks

## 📈 Documentation Metrics

-   **Coverage**: All major components documented
-   **Freshness**: Updated automatically with code changes
-   **Accessibility**: Clear navigation and search-friendly
-   **Maintainability**: Automated generation reduces maintenance burden

## 🤝 Contributing to Documentation

When contributing to documentation:

1.  **Use Templates** - Follow established templates for consistency
2.  **Automate Updates** - Ensure documentation updates are automated
3.  **Validate Changes** - Run `make validate-docs` before committing
4.  **Update References** - Keep cross-references current
5.  **Follow Standards** - Adhere to established formatting and structure

## 🔍 Finding Information

-   **New to the project?** Start with [User Guide](user-guide/README.md)
-   **Want to contribute?** Read [Contributing Guide](developer/CONTRIBUTING.md)
-   **Need technical details?** See [Architecture Overview](architecture/ARCHITECTURE.md)
-   **Planning deployment?** Check [Deployment Guide](operations/DEPLOYMENT.md)

---

**Last updated:** Generated automatically - see [CI Status](https://github.com/marlonsc/mcp-context-browser/actions)
