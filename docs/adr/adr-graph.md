digraph {
  node [shape=plaintext]
  subgraph {
    _1 [label="1. Record architecture decisions"; URL="0001-record-architecture-decisions.html"];
    _1 [label="ADR 001: Provider Pattern Architecture"; URL="001-provider-pattern-architecture.html"];
    _2 [label="ADR 002: Async-First Architecture"; URL="002-async-first-architecture.html"];
    _1 -> _2 [style="dotted", weight=1];
    _3 [label="ADR 003: C4 Model Documentation"; URL="003-C4-model-documentation.html"];
    _2 ->_3 [style="dotted", weight=1];
    _4 [label="ADR 004: Multi-Provider Strategy"; URL="004-multi-provider-strategy.html"];
    _3 -> _4 [style="dotted", weight=1];
    _5 [label="ADR 005: Documentation Excellence v0.1.0"; URL="005-documentation-excellence-v0.1.0.html"];
    _4 ->_5 [style="dotted", weight=1];
    _6 [label="ADR 006: Code Audit and Architecture Improvements v0.1.0"; URL="006-code-audit-and-improvements-v0.1.0.html"];
    _5 -> _6 [style="dotted", weight=1];
    _7 [label="ADR 007: Integrated Web Administration Interface"; URL="007-integrated-web-administration-interface.html"];
    _6 ->_7 [style="dotted", weight=1];
    _8 [label="ADR 008: Git-Aware Semantic Indexing v0.2.0"; URL="008-git-aware-semantic-indexing-v0.2.0.html"];
    _7 -> _8 [style="dotted", weight=1];
    _9 [label="ADR 009: Persistent Session Memory v0.2.0"; URL="009-persistent-session-memory-v0.2.0.html"];
    _8 ->_9 [style="dotted", weight=1];
    _1 ->_9 [label="extends", style="dashed"];
    _4 ->_9 [label="extends", style="dashed"];
    _10 [label="ADR 010: Hooks Subsystem v0.2.0"; URL="010-hooks-subsystem-agent-backed.html"];
    _9 -> _10 [style="dotted", weight=1];
    _1 -> _10 [label="extends", style="dashed"];
    _9 -> _10 [label="uses", style="dashed"];
    _8 -> _10 [label="integrates", style="dashed"];
    _13 [label="ADR 013: Clean Architecture Crate Separation"; URL="013-clean-architecture-crate-separation.html"];
    _24 [label="ADR 024: Simplified Dependency Injection"; URL="024-simplified-dependency-injection.html"];
    _27 [label="ADR 027: Architecture Evolution v0.1.3"; URL="027-architecture-evolution-v013.html"];
    _10 ->_27 [style="dotted", weight=1];
    _13 ->_27 [label="extends", style="dashed"];
    _24 ->_27 [label="extends", style="dashed"];
    _27 ->_8 [label="prepares", style="dashed"];
  }
}
