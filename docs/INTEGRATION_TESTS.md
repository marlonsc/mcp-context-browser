# Integration Tests - Redis and NATS

This document explains how to run integration tests that use Redis (cache) and NATS (event bus) against real local services.

**Note:** Current integration tests use `skip_if_service_unavailable!` and check Milvus, Ollama, Redis, Postgres, etc. (see `crates/mcb-server/tests/integration/helpers.rs`). The specific `cargo test redis_cache_integration` / `nats_event_bus_integration` targets may not exist; use `make test` or `make test SCOPE=integration` for the actual suite. The Redis/NATS Docker setup below remains useful when those services are required.

## Quick Start

### Prerequisites

Ensure Redis and NATS are running on your host machine:

```bash

# Check Redis
redis-cli ping

# Expected: PONG

# Check NATS (via monitoring port)
curl -s http://localhost:8222/healthz

# Expected: OK
```

### Run All Integration Tests

```bash

# Option 1: Redis + NATS only (lightweight)
docker-compose -f docker-compose.testing.yml up -d
make test
# docker-compose -f docker-compose.testing.yml down -v when done

# Option 2: Full stack (Ollama, Milvus, etc.) via make docker-up
make docker-up
make test
make docker-down
```

`make docker-up` uses `docker-compose.yml` (Ollama, Milvus, OpenAI mock). For Redis and NATS only, use `docker-compose.testing.yml` as above.

## Detailed Setup

### 1. Start Host Services

**Option A: Manual startup**

```bash

# Terminal 1: Redis
redis-server --port 6379

# Terminal 2: NATS with JetStream
nats-server --jetstream
```

**Option B: Using Docker containers (recommended)**

```bash

# Start with custom docker-compose
docker-compose -f docker-compose.testing.yml up -d

# Verify services
docker-compose -f docker-compose.testing.yml ps
```

**Option C: System services**

```bash

# If installed via system package manager
systemctl start redis
systemctl start nats-server
```

### 2. Run Integration Tests

#### Method 1: Local Tests (Direct Connection)

```bash

# Run all tests (recommended; includes integration tests that use Redis/NATS when available)
make test

# Or only integration tests
make test SCOPE=integration

# With environment variables if services are on different hosts
REDIS_URL=redis://192.168.1.100:6379 NATS_URL=nats://192.168.1.100:4222 make test
```

If Redis/NATS-specific test targets (e.g. `redis_cache_integration`, `nats_event_bus_integration`) exist, you can run them with `cargo test <name> -- --nocapture`. Otherwise use `make test` above.

#### Method 2: Docker services + local tests

Start Redis and NATS via `docker-compose.testing.yml`, then run tests on the host:

```bash

docker-compose -f docker-compose.testing.yml up -d
REDIS_URL=redis://127.0.0.1:6379 NATS_URL=nats://127.0.0.1:4222 make test
docker-compose -f docker-compose.testing.yml down -v
```

For the full stack (Ollama, Milvus): `make docker-up` then `make test` then `make docker-down`.

#### Method 3: Full Docker Compose (Container Test Runner)

Test runner executes inside Docker container and connects to host services:

```bash

# Full test cycle with test-runner container
docker-compose up

# Or manually:
docker-compose up -d          # Start all services including test-runner
docker-compose logs -f        # Monitor test execution
docker-compose down -v        # Cleanup
```

## Test Files

### Redis Cache Provider Tests

**See:** `crates/mcb-server/tests/integration/helpers.rs` (e.g. `is_redis_available`), `crates/mcb-providers/src/cache/redis.rs`, and integration tests.

Tests include:

-   Provider creation and configuration
-   Set/Get operations
-   Delete operations
-   Namespace clearing
-   Key existence checks
-   TTL expiration
-   Health checks
-   Concurrent access
-   Connection pooling
-   Large payload handling

Run: `make test` or `make test SCOPE=integration`. If a dedicated `redis_cache_integration` test exists, use `cargo test redis_cache_integration -- --nocapture`.

### NATS Event Bus Tests

**See:** `crates/mcb-infrastructure/src/infrastructure/events.rs` and integration tests. NATS availability checks may use similar patterns to `is_redis_available` in `integration/helpers.rs`.

Tests include:

-   Provider creation and configuration
-   Publish/Subscribe operations
-   Multiple subscribers
-   Different event types
-   Concurrent publishing
-   Health checks
-   Message recovery
-   Large payload handling
-   Stream persistence

Run: `make test` or `make test SCOPE=integration`. If a dedicated `nats_event_bus_integration` test exists, use `cargo test nats_event_bus_integration -- --nocapture`.

## Environment Variables

Tests automatically detect services using these environment variables (in order of priority):

### Redis

1.  `REDIS_URL` - Primary: `redis://host:port`
2.  `MCP_CACHE__URL` - Fallback: `redis://host:port`
3.  Default: `redis://127.0.0.1:6379`

### NATS

1.  `NATS_URL` - Primary: `nats://host:port`
2.  `MCP_NATS_URL` - Fallback: `nats://host:port`
3.  Default: `nats://127.0.0.1:4222`

Example:

```bash

# Use custom host services
REDIS_URL=redis://custom-host:6379 NATS_URL=nats://custom-host:4222 make test
```

## Docker Integration

### docker-compose.yml

The main Docker Compose file includes:

-   **OpenAI-mock**: OpenAI API mock server (port 1080)
-   **Ollama**: Ollama embedding service (port 11434)
-   **Milvus-***: Milvus vector database (port 19530)
-   **test-runner**: Test execution container (runs `make test` inside the container)

The test-runner connects to:

-   Docker services via internal network (`mcp-openai-mock:1080`, etc.)
-   Host services via `host.docker.internal:port` (macOS) or `172.17.0.1:port` (Linux)

**Usage:**

```bash

# Start everything
docker-compose up

# Stop everything
docker-compose down -v

# View logs
docker-compose logs -f test-runner
```

### docker-compose.testing.yml

Lightweight compose with only Redis and NATS for quick testing:

**Usage:**

```bash

# Start only Redis and NATS
docker-compose -f docker-compose.testing.yml up -d

# Run tests
make test

# Stop services
docker-compose -f docker-compose.testing.yml down -v
```

## Service Detection

Tests automatically skip if services are unavailable via `skip_if_service_unavailable!` and helpers in `integration/helpers.rs` (e.g. `is_redis_available`, `is_milvus_available`, `is_ollama_available`):

```rust
skip_if_service_unavailable!("Redis", is_redis_available());
skip_if_service_unavailable!("Milvus", is_milvus_available());
```

When a required service is missing, tests skip with a message such as:

```
⊘ SKIPPED: Redis service not available (skipping test)
```

## Make Targets

```bash
make test                      # Run all unit + integration tests locally
make test SCOPE=integration    # Run only integration tests
make docker-up                 # Start main stack (docker-compose.yml: Ollama, Milvus, etc.)
make docker-down               # Stop main stack
make docker-logs               # View Docker logs
make docker                    # Show Docker service status
```

For Redis + NATS only, use `docker-compose -f docker-compose.testing.yml up -d` (and `down -v` when done). `make docker-up` uses the main compose, not the testing one.

## Troubleshooting

### Redis Connection Refused

```bash

# Check if Redis is running
redis-cli ping

# Start Redis
redis-server --port 6379 --appendonly yes

# Or with Docker
docker-compose -f docker-compose.testing.yml up -d redis
```

### NATS Connection Refused

```bash

# Check if NATS is running
telnet localhost 4222

# Start NATS (with JetStream)
nats-server --jetstream

# Or with Docker
docker-compose -f docker-compose.testing.yml up -d nats
```

### host.docker.internal not working (Linux)

The docker-compose.yml uses `extra_hosts` with `host-gateway` to automatically resolve `host.docker.internal` on Linux. If it still doesn't work:

```bash

# Get host IP
docker network inspect mcp-test

# Use IP directly
docker exec mcp-test-runner bash
export REDIS_URL=redis://172.17.0.1:6379  # Replace with actual host IP
make test
```

### Tests Timeout

Increase timeout and add debugging:

```bash
RUST_LOG=debug make test
# Or, for a single test: cargo test <test_name> -- --nocapture --test-threads=1
```

### Container Cannot Reach Host Services

Verify connectivity from container:

```bash

# From host
docker exec -it mcp-test-runner bash

# Inside container, test connectivity
redis-cli -h host.docker.internal -p 6379 ping
telnet host.docker.internal 4222
```

## Test Results

### Expected Output

```
Running 10 Redis integration tests...
✅ Redis cache provider created successfully
✅ Redis set/get operations work correctly
✅ Redis delete operation works correctly
✅ Redis clear namespace operation works correctly
✅ Redis exists operation works correctly
✅ Redis TTL expiration works correctly
✅ Redis health check works correctly
✅ Redis concurrent access works correctly
✅ Redis connection pooling works correctly
...

Running 8 NATS integration tests...
✅ NATS event bus created successfully
✅ NATS publish/subscribe works correctly
✅ NATS multiple subscribers work correctly
...

test result: ok. 18 passed; 0 failed; 0 ignored
```

### Performance

Typical execution times:

-   Redis tests: ~15-20 seconds (including TTL wait)
-   NATS tests: ~25-30 seconds (including persistence wait)
-   Total: ~45-50 seconds

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  integration-tests:
    runs-on: ubuntu-latest

    services:
      redis:
        image: redis:7-alpine
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
-   6379:6379

      nats:
        image: nats:latest
        options: >-
          --health-cmd "wget -q --spider http://localhost:8222/healthz"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
-   4222:4222

    steps:
-   uses: actions/checkout@v3
-   uses: dtolnay/rust-toolchain@stable

-   name: Run integration tests
        run: |
          REDIS_URL=redis://127.0.0.1:6379 NATS_URL=nats://127.0.0.1:4222 make test
```

## Additional Resources

-   [Redis Documentation](https://redis.io/documentation)
-   [NATS Documentation](https://docs.nats.io/)
-   [MCP Context Browser Architecture](./architecture/ARCHITECTURE.md)
-   [ADR-005: Context Cache Support (Moka and Redis)](./adr/005-context-cache-support.md)

## Contributing

When adding new integration tests:

1.  Use existing patterns in `crates/mcb-server/tests/integration/helpers.rs` and `crates/mcb-providers/src/cache/redis.rs`
2.  Include environment variable support for flexible service locations
3.  Use `skip_if_service_unavailable!("Service", is_*_available())` for graceful skipping
4.  Add cleanup code to prevent test pollution
5.  Include both success and failure paths
6.  Document expected behavior in test comments

## Support

For issues or questions:

1.  Check the Troubleshooting section above
2.  Review test output with `--nocapture` flag
3.  Check Docker logs: `docker-compose logs`
4.  Check service health: `make docker`
5.  Open an issue on GitHub with test output
