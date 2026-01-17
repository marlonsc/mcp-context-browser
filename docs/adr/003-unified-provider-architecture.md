# ADR 003: Unified Provider Architecture

## Status

**Implemented** (v0.1.1)

> Fully implemented with unified provider port traits across multiple categories (Embedding: 6 providers, Vector Store: 3 providers, Cache: 3 providers, Language: 12 processors, Events: 2 providers).

## Context

The MCP Context Browser interacts with multiple context sources (local memory, external providers, etc.), originally handled in different ways. Each "provider" of context had its own configurations and initialization, which increased complexity in adding new providers and maintaining consistency. We identified the opportunity to unify how providers are defined and loaded by the system, standardizing the interface and lifecycle. In addition, integrating providers into the DI container (Shaku) would bring consistency in dependency resolution.

## Decision

We defined a unified interface for context providers, so that all providers implement the same basic trait (for example, ContextProvider) with standard operations (such as init, shutdown, and search/storage methods). We unified the registration of these providers in the system as well: now, all providers are registered via ServiceManager/Shaku during initialization, instead of ad-hoc initializations scattered around. This means that to add a new provider, simply create an implementation of the trait and register it in the project's DI module - the lifecycle (initialization, use, and termination) will be managed homogeneously. All providers share common mechanisms for logging, configuration, and EventBus usage (see ADR 004) for emitting events from their operations.

## Consequences

The unification of providers brought coherence and ease of extension. New providers now follow a clear contract, reducing code duplication for infrastructure. Configuration became centralized: the application configuration file can list which providers to activate and their parameters, and the system loads them uniformly. It also facilitated error handling and monitoring, as providers now report events in a standardized way (e.g., a provider can emit an event in the EventBus when updating context, and any other interested component can listen). In counterpoint, there was rework to adapt legacy providers to the new interface and migrate old configurations. However, the benefits outweighed the costs - the code became more organized and less prone to integration errors. Action required: formalize in a specific ADR future decisions to add/remove providers or alter the unified trait, ensuring historical record of these architectural changes.
