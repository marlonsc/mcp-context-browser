# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records (ADRs) for the MCP Context Browser project. ADRs are permanent, historical records of architectural decisions made during the development of the system.

## What is an ADR?

An ADR is a document that captures an important architectural decision made along with its context and consequences. ADRs are immutable once accepted - they represent the historical record of why certain decisions were made.

## ADR Structure

Each ADR follows this template:

```
# ADR {number}: {title}

## Status
{Proposed | Accepted | Rejected | Deprecated | Superseded by ADR-xxx}

## Context
{What is the problem we are trying to solve?}

## Decision
{What decision was made?}

## Consequences
{What are the positive and negative consequences of this decision?}

## Alternatives Considered
{What other options were considered and why were they rejected?}

## Implementation Notes
{Any technical details about implementation}

## References
{Links to related documents, issues, or discussions}
```

## ADR Status

-   **Proposed**: Under discussion and review
-   **Accepted**: Decision made and implemented
-   **Rejected**: Decision rejected with rationale
-   **Deprecated**: No longer relevant
-   **Superseded**: Replaced by a newer ADR

## Current ADRs

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [001](001-provider-pattern-architecture.md) | Provider Pattern Architecture | Accepted | 2024-01-06 |
| [002](002-async-first-architecture.md) | Async-First Architecture | Accepted | 2024-01-06 |
| [003](003-c4-model-documentation.md) | C4 Model Documentation | Accepted | 2024-01-06 |
| [004](004-multi-provider-strategy.md) | Multi-Provider Strategy | Accepted | 2024-01-06 |

## Creating a New ADR

1.  **Identify the Decision**: Determine if the decision requires an ADR
2.  **Draft the ADR**: Use the template above and place it in `docs/adr/`
3.  **Review Process**: Technical review and stakeholder feedback
4.  **Accept/Reject**: Make final decision and update status
5.  **Implement**: Update code and documentation as needed

## ADR Maintenance

-   ADRs are immutable once accepted
-   Update the README index when adding new ADRs
-   Reference superseded ADRs in new decisions
-   Review ADRs periodically for continued relevance

## Tools and Automation

The project includes tools for ADR management:

```bash
# Create a new ADR draft
cargo run --bin adr-tool -- create "New Feature Decision"

# List all ADRs
cargo run --bin adr-tool -- list

# Search ADRs by keyword
cargo run --bin adr-tool -- search "security"
```

## Contributing

When proposing architectural changes:

1.  Create an ADR in draft status
2.  Discuss with the technical team
3.  Update status based on consensus
4.  Implement the accepted decision
5.  Update related documentation

See [CONTRIBUTING.md](../CONTRIBUTING.md) for detailed contribution guidelines.
