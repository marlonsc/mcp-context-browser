# ADR 011: HTTP Transport - Request-Response Pattern Over SSE Streaming

## Status

**Documented**(v0.1.2)

> Implemented with POST request-response pattern. SSE streaming deferred to v0.2.0.
> Current implementation in `crates/mcb-server/src/transport/http.rs` provides:
>
> -   POST /MCP for client-to-server requests with immediate responses
> -   GET /MCP returns 501 Not Implemented with clear messaging
> -   Session management and message buffering infrastructure ready for future SSE support

## Context

The MCP (Model Context Protocol) specification defines a Streamable HTTP transport with:

-   **POST /MCP**: Client sends requests, receives responses (request-response pattern)
-   **GET /MCP**: Server streams updates to client via Server-Sent Events (SSE)

The current v0.1.0 implementation needed to decide whether to implement both patterns immediately or defer SSE streaming to a future release.

## Decision

**Implement request-response pattern only in v0.1.0**

1.**POST /MCP**: Fully functional request-response endpoint

-   Clients submit requests via POST and receive responses immediately
-   Supports session management, message buffering, and resumption
-   Works for all core functionality (search, index, etc.)

2.**GET /MCP**: Return 501 Not Implemented with clear messaging

-   Clients are explicitly informed SSE is not yet supported
-   Better than 200 OK with empty response (which would be misleading)
-   Infrastructure for SSE already in place (session handling, event IDs, message buffering)

## Rationale

### Why Request-Response is Sufficient for v0.1.0

1.**Functional Completeness**

-   All core MCP operations work via POST request-response pattern
-   Real-world usage patterns show POST is primary mechanism
-   Clients can implement polling for continuous updates if needed

2.**Reduced Complexity**

-   SSE streaming adds significant complexity:
-   Connection state management
-   Client reconnection handling
-   Event ordering and deduplication
-   Browser/HTTP proxy compatibility issues
-   Request-response is simpler, more reliable, and easier to debug

3.**Sufficient Infrastructure**

-   Session management already implemented
-   Message buffering for resumption ready
-   Event ID tracking (prepared for SSE)
-   No architectural changes needed for future SSE support

4.**Clear Communication**

-   Returning 501 with explanation is honest and helpful
-   Clients won't waste time trying to use non-existent feature
-   Sets clear expectations for v0.1.0 vs v0.2.0

### Why Defer to v0.2.0

1.**Future-Proof Architecture**

-   Current code structure supports adding SSE later
-   Session/buffering infrastructure is SSE-ready
-   No breaking changes needed when SSE is added

2.**Lower Risk**

-   Shipping v0.1.0 without SSE reduces complexity and bugs
-   Focus on core functionality quality
-   SSE can be added in incremental v0.2.0 release

3.**Alternative Patterns Available**

-   Clients can use polling with request-response
-   Pub/Sub via event bus for async notifications
-   WebSockets in future releases if needed

## Consequences

### Positive Consequences

1.**Reduced Scope & Risk**

-   v0.1.0 ships faster with fewer potential issues
-   Core functionality is stable and well-tested
-   Clear versioning and feature roadmap

2.**Better User Experience**

-   Explicit 501 response is better than misleading 200
-   Clients know exactly what's supported
-   No confusion about SSE vs polling

3.**Maintainability**

-   Simpler codebase easier to understand and modify
-   Request-response pattern is easier to test
-   Fewer edge cases to handle

4.**Architecture Flexibility**

-   Infrastructure in place for future SSE implementation
-   No need for breaking changes in v0.2.0
-   Can evaluate other patterns (WebSockets, gRPC) later

### Negative Consequences

1.**Limited Real-Time Streaming**

-   Clients must use polling or request-response for continuous updates
-   Not ideal for high-frequency update scenarios
-   SSE would be more efficient for server-pushed updates

2.**Incomplete Spec Compliance**

-   MCP specification defines streaming as optional feature
-   Current implementation doesn't fully support spec
-   May limit interoperability with certain MCP clients

3.**Feature Parity Gap**

-   Some MCP implementations may have SSE streaming
-   Users expecting full streaming support may be disappointed
-   v0.2.0 migration may require client code changes

## Alternatives Considered

### Alternative 1: Full SSE Implementation in v0.1.0

**Approach**: Implement Server-Sent Events streaming for GET /MCP endpoint

**Pros**:

-   Full MCP spec compliance
-   More efficient server-to-client updates
-   Real-time streaming support

**Cons**:

-   Significantly more complex implementation
-   Connection state management challenges
-   Browser/proxy compatibility issues
-   Higher risk of bugs in initial release
-   Delays v0.1.0 release

**Status**: Deferred to v0.2.0

### Alternative 2: Return 200 OK with Empty Stream

**Approach**: Respond to GET /MCP with 200 OK and empty SSE stream

**Pros**:

-   Spec-compliant response code

**Cons**:

-   Misleading to clients (implies working connection)
-   Clients won't know why they're getting no data
-   Encourages broken behavior
-   Bad user experience

**Status**: Rejected in favor of 501

### Alternative 3: WebSocket Transport

**Approach**: Use WebSocket instead of HTTP/SSE for bidirectional streaming

**Pros**:

-   Better for real-time bidirectional communication
-   Better performance for frequent updates
-   Simpler client libraries

**Cons**:

-   Not part of MCP spec (which uses HTTP)
-   Requires different client implementation
-   More infrastructure requirements
-   Out of scope for HTTP transport

**Status**: Deferred for potential v0.3.0 as alternative transport

## Implementation Notes

### Current State (v0.1.0)

```rust
// crates/mcb-server/src/transport/http.rs:115-143
async fn handle_mcp_get(
    State(state): State<HttpTransportState>,
    headers: HeaderMap,
) -> Result<Response, McpError> {
    // Session validation happens here

    // TODO: Implement Server-Sent Events streaming
    // For now, return 501 to indicate not implemented
    Err(McpError::NotImplemented(
        "SSE streaming not yet implemented. Use POST for request-response communication."
            .to_string(),
    ))
}
```

### Ready for v0.2.0 Implementation

The following infrastructure is already in place:

1.**Session Management**(`crates/mcb-server/src/transport/session.rs`)

-   Session creation and tracking
-   Activity timestamps
-   Message buffering

2.**Message Buffering**(in POST handler)

-   Event ID generation
-   Message history per session
-   Resumption support

3.**Error Handling**

-   McpError enum with NotImplemented variant
-   Response serialization infrastructure

### Migration Path to v0.2.0

```rust
// Pseudocode for v0.2.0 SSE implementation
async fn handle_mcp_get(
    State(state): State<HttpTransportState>,
    headers: HeaderMap,
) -> Result<Response, McpError> {
    // 1. Validate session (already done)
    let session_id = extract_session_id(&headers)?;
    let session = state.session_manager.get_session(&session_id)?;

    // 2. Create SSE stream
    let stream = create_sse_stream(
        state.session_manager.clone(),
        session_id.to_string(),
    );

    // 3. Return streaming response
    Ok(body_stream_response(stream))
}
```

## Recommendations

1.**Document clearly**in all client libraries and examples that GET /MCP is not yet implemented
2.**Monitor feedback**from users about SSE needs
3.**Plan v0.2.0**SSE implementation if users need real-time streaming
4.**Consider alternative patterns**if WebSocket demand grows (v0.3.0+)
5.**Update MCP compliance matrix**to note SSE as deferred feature

## Related ADRs

-   [ADR-001: Modular Crates Architecture](001-modular-crates-architecture.md) - Provider pattern for HTTP clients
-   [ADR-002: Async-First Architecture](002-async-first-architecture.md) - Async HTTP handling with Tokio
-   [ADR-007: Integrated Web Administration Interface](007-integrated-web-administration-interface.md) - Unified port architecture
-   [ADR-012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) - Shaku DI for transport services
-   [ADR-013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - mcb-server crate organization

## References

-   **MCP Specification**: [Model Context Protocol](https://modelcontextprotocol.io/)
-   **Transport Layer**: `crates/mcb-server/src/transport/http.rs`, `crates/mcb-server/src/transport/session.rs`
-   **Related Issues**: See GitHub issues tagged with "sse" or "streaming"
-   [Shaku Documentation](https://docs.rs/shaku) - DI framework (historical; see ADR-029)

## Reviewers

-   Architecture Review: Pending
-   Security Review: Pending (for v0.2.0 SSE implementation)
