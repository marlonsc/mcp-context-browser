# MCP Context Browser - Documentation

[![Documentation Status](https://img.shields.io/badge/docs-automated-green)](https://github.com/marlonsc/mcp-context-browser/actions)
[![Version](https://img.shields.io/badge/version-0.1.1-blue)](https://github.com/marlonsc/mcp-context-browser/releases)
[![Architecture](https://img.shields.io/badge/architecture-C4--model-blue)](docs/architecture/ARCHITECTURE.md)

**Comprehensive documentation for the MCP Context Browser project**

## 📚 Documentation Structure

This documentation is organized into focused sections for different audiences and purposes:

### 📖 User Guide

User-facing documentation for installation, usage, and features.

\1-  **[README](user-guide/README.md)**- Project overview, quick start, and basic usage
\1-  **[Features](user-guide/README.md#current-capabilities-v001)**- Current capabilities and features

### 🛠️ Developer Guide

Documentation for developers contributing to the project.

\1-  **[Contributing](developer/CONTRIBUTING.md)**- Development setup and contribution guidelines
\1-  **[Roadmap](developer/ROADMAP.md)**- Development roadmap and milestones

### 🏗️ Architecture

Technical architecture documentation following C4 model principles.

\1-  **[Architecture Overview](architecture/ARCHITECTURE.md)**- Comprehensive system architecture
\1-  **[Architecture Diagrams](diagrams/)**- Visual architecture documentation
\1-   [System Context](diagrams/generated/index.html) - System boundaries and external systems
\1-   [Container Architecture](diagrams/generated/index.html) - Service and deployment architecture
\1-  **[Architecture Decision Records](adr/)**- Historical architectural decisions
\1-   [ADR 001: Provider Pattern](adr/001-provider-pattern-architecture.md)
\1-   [ADR 002: Async-First Architecture](adr/002-async-first-architecture.md)
\1-   [ADR 003: C4 Model Documentation](adr/003-c4-model-documentation.md)
\1-   [ADR 004: Multi-Provider Strategy](adr/004-multi-provider-strategy.md)

### 🚀 Operations

Operational documentation for deployment and maintenance.

\1-  **[Deployment Guide](operations/DEPLOYMENT.md)**- Deployment configurations and environments
\1-  **[Changelog](operations/CHANGELOG.md)**- Version history and release notes

### 📋 Templates

Documentation templates and standards.

\1-  **[ADR Template](templates/adr-template.md)**- Template for new Architecture Decision Records

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
|**Automation**| ✅ Automated | Fully automated generation and validation |
|**Consistency**| ✅ Validated | Cross-references and structure validation |
|**Architecture**| ✅ C4 Model | Structured architectural documentation |
|**Diagrams**| ✅ Generated | PlantUML-generated architecture diagrams |
|**Validation**| ✅ CI/CD | Automated validation in CI pipeline |

## 🎯 Documentation Principles

1.  **Single Source of Truth**- Documentation stays synchronized with code
2.  **Audience-Specific**- Different views for different stakeholders
3.  **Automated Maintenance**- No manual updates required
4.  **Version Controlled**- All documentation is version controlled
5.  **Quality Assured**- Automated validation and consistency checks

## 📈 Documentation Metrics

\1-  **Coverage**: All major components documented
\1-  **Freshness**: Updated automatically with code changes
\1-  **Accessibility**: Clear navigation and search-friendly
\1-  **Maintainability**: Automated generation reduces maintenance burden

## 🤝 Contributing to Documentation

When contributing to documentation:

1.  **Use Templates**- Follow established templates for consistency
2.  **Automate Updates**- Ensure documentation updates are automated
3.  **Validate Changes**- Run `make validate-docs` before committing
4.  **Update References**- Keep cross-references current
5.  **Follow Standards**- Adhere to established formatting and structure

## 🔍 Finding Information

\1-  **New to the project?**Start with [User Guide](user-guide/README.md)
\1-  **Want to contribute?**Read [Contributing Guide](developer/CONTRIBUTING.md)
\1-  **Need technical details?**See [Architecture Overview](architecture/ARCHITECTURE.md)
\1-  **Planning deployment?**Check [Deployment Guide](operations/DEPLOYMENT.md)

---

**Last updated:**Generated automatically - see [CI Status](https://github.com/marlonsc/mcp-context-browser/actions)
