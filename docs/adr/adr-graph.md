digraph {
  node [shape=plaintext]
  subgraph {
    _1 [label="ADR 001: Modular Crates Architecture"; URL="001-modular-crates-architecture.html"];
    _2 [label="ADR 002: Async-First Architecture"; URL="002-async-first-architecture.html"];
    _1 ->_2 [style="dotted", weight=1];
    _3 [label="ADR 003: Unified Provider Architecture"; URL="003-unified-provider-architecture.html"];
    _2 -> _3 [style="dotted", weight=1];
    _4 [label="ADR 004: Event Bus (Local and Distributed)"; URL="004-event-bus-local-distributed.html"];
    _3 ->_4 [style="dotted", weight=1];
    _5 [label="ADR 005: Context Cache Support (Moka and Redis)"; URL="005-context-cache-support.html"];
    _4 -> _5 [style="dotted", weight=1];
    _6 [label="ADR 006: Code Audit and Architecture Improvements"; URL="006-code-audit-and-improvements.html"];
    _5 ->_6 [style="dotted", weight=1];
    _7 [label="ADR 007: Integrated Web Administration Interface"; URL="007-integrated-web-administration-interface.html"];
    _6 -> _7 [style="dotted", weight=1];
    _8 [label="ADR 008: Git-Aware Semantic Indexing v0.2.0"; URL="008-git-aware-semantic-indexing-v0.2.0.html"];
    _7 ->_8 [style="dotted", weight=1];
    _9 [label="ADR 009: Persistent Session Memory v0.2.0"; URL="009-persistent-session-memory-v0.2.0.html"];
    _8 -> _9 [style="dotted", weight=1];
    _1 -> _9 [label="extends", style="dashed"];
    _4 -> _9 [label="extends", style="dashed"];
    _10 [label="ADR 010: Hooks Subsystem with Agent-Backed Processing"; URL="010-hooks-subsystem-agent-backed.html"];
    _9 ->_10 [style="dotted", weight=1];
    _1 ->_10 [label="extends", style="dashed"];
    _9 ->_10 [label="uses", style="dashed"];
    _8 ->_10 [label="integrates", style="dashed"];
    _13 [label="ADR 013: Clean Architecture Crate Separation"; URL="013-clean-architecture-crate-separation.html"];
    _24 [label="ADR 024: Simplified Dependency Injection"; URL="024-simplified-dependency-injection.html"];
    _27 [label="ADR 027: Architecture Evolution v0.1.3"; URL="027-architecture-evolution-v013.html"];
    _29 [label="ADR 029: Hexagonal Architecture with dill"; URL="029-hexagonal-architecture-dill.html"];
    _30 [label="ADR 030: Multi-Provider Strategy"; URL="030-multi-provider-strategy.html"];
    _31 [label="ADR 031: Documentation Excellence"; URL="031-documentation-excellence.html"];
    _10 ->_27 [style="dotted", weight=1];
    _13 ->_27 [label="extends", style="dashed"];
    _24 ->_27 [label="extends", style="dashed"];
    _24 ->_29 [label="superseded by", style="dashed"];
    _27 ->_8 [label="prepares", style="dashed"];
    _30 ->_9 [label="extends", style="dashed"];
    _5 ->_31 [label="relates", style="dashed"];
  }
}
