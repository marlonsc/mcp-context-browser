# Integration Tests - Redis and NATS

This document explains how to run the comprehensive integration tests for Redis cache provider and NATS event bus against real local services.

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
# Option 1: Direct tests (fastest - tests connect directly to host services)
make test-docker

# Option 2: Docker container tests (container connects to host services)
docker-compose up
```

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
# Run only Redis tests
cargo test redis_cache_integration -- --nocapture

# Run only NATS tests
cargo test nats_event_bus_integration -- --nocapture

# Run all integration tests
make test

# Or with environment variables if services are on different hosts
REDIS_URL=redis://192.168.1.100:6379 \
NATS_URL=nats://192.168.1.100:4222 \
cargo test redis_cache_integration nats_event_bus_integration -- --nocapture
```

#### Method 2: Docker Container Tests (Recommended for CI/CD)

Uses Docker containers for OpenAI mock, Ollama, and Milvus while connecting to host Redis/NATS:

```bash
# Full Docker integration test cycle
make test-docker

# Or step by step:
make docker-up                           # Start Docker services
REDIS_URL=redis://127.0.0.1:6379 \
NATS_URL=nats://127.0.0.1:4222 \
make test-integration-docker            # Run tests inside docker environment
make docker-down                         # Stop Docker services
```

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
**File:** `tests/infrastructure/redis_cache_integration.rs`

Tests include:
- Provider creation and configuration
- Set/Get operations
- Delete operations
- Namespace clearing
- Key existence checks
- TTL expiration
- Health checks
- Concurrent access
- Connection pooling
- Large payload handling

Run:
```bash
cargo test redis_cache_integration -- --nocapture
```

### NATS Event Bus Tests
**File:** `tests/infrastructure/nats_event_bus_integration.rs`

Tests include:
- Provider creation and configuration
- Publish/Subscribe operations
- Multiple subscribers
- Different event types
- Concurrent publishing
- Health checks
- Message recovery
- Large payload handling
- Stream persistence

Run:
```bash
cargo test nats_event_bus_integration -- --nocapture
```

## Environment Variables

Tests automatically detect services using these environment variables (in order of priority):

### Redis
1. `REDIS_URL` - Primary: `redis://host:port`
2. `MCP_CACHE__URL` - Fallback: `redis://host:port`
3. Default: `redis://127.0.0.1:6379`

### NATS
1. `NATS_URL` - Primary: `nats://host:port`
2. `MCP_NATS_URL` - Fallback: `nats://host:port`
3. Default: `nats://127.0.0.1:4222`

Example:
```bash
# Use custom host services
REDIS_URL=redis://custom-host:6379 \
NATS_URL=nats://custom-host:4222 \
cargo test redis_cache_integration nats_event_bus_integration -- --nocapture
```

## Docker Integration

### docker-compose.yml

The main Docker Compose file includes:
- **openai-mock**: OpenAI API mock server (port 1080)
- **ollama**: Ollama embedding service (port 11434)
- **milvus-***: Milvus vector database (port 19530)
- **test-runner**: Test execution container

The test-runner connects to:
- Docker services via internal network (`mcp-openai-mock:1080`, etc.)
- Host services via `host.docker.internal:port` (macOS) or `172.17.0.1:port` (Linux)

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

Tests automatically skip if services are unavailable:

```rust
skip_if_no_redis!();  // Skips if Redis not available
skip_if_no_nats!();   // Skips if NATS not available
```

Output:
```
⚠️  Skipping test: Redis not available on localhost:6379
    Start Redis with: docker-compose up -d redis
```

## Make Targets

```bash
make test                      # Run all unit + integration tests locally
make test-docker               # Run with Docker services + host Redis/NATS
make test-integration          # Run only integration tests
make test-integration-docker   # Run integration tests in docker environment
make docker-up                 # Start Docker services
make docker-down               # Stop Docker services
make docker-logs               # View Docker logs
make docker-status             # Show service status
```

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
cargo test redis_cache_integration
```

### Tests Timeout

Increase timeout and add debugging:

```bash
RUST_LOG=debug cargo test redis_cache_integration -- --nocapture --test-threads=1
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
- Redis tests: ~15-20 seconds (including TTL wait)
- NATS tests: ~25-30 seconds (including persistence wait)
- Total: ~45-50 seconds

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
          - 6379:6379

      nats:
        image: nats:latest
        options: >-
          --health-cmd "wget -q --spider http://localhost:8222/healthz"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 4222:4222

    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      - name: Run integration tests
        run: |
          REDIS_URL=redis://127.0.0.1:6379 \
          NATS_URL=nats://127.0.0.1:4222 \
          cargo test redis_cache_integration nats_event_bus_integration -- --nocapture
```

## Additional Resources

- [Redis Documentation](https://redis.io/documentation)
- [NATS Documentation](https://docs.nats.io/)
- [MCP Context Browser Architecture](./architecture/ARCHITECTURE.md)
- [Provider Pattern Implementation](./adr/005-provider-pattern.md)

## Contributing

When adding new integration tests:

1. Use the existing patterns in `redis_cache_integration.rs` and `nats_event_bus_integration.rs`
2. Include environment variable support for flexible service locations
3. Use `skip_if_no_service!()` macro for graceful skipping
4. Add cleanup code to prevent test pollution
5. Include both success and failure paths
6. Document expected behavior in test comments

## Support

For issues or questions:

1. Check the Troubleshooting section above
2. Review test output with `--nocapture` flag
3. Check Docker logs: `docker-compose logs`
4. Check service health: `make docker-status`
5. Open an issue on GitHub with test output
