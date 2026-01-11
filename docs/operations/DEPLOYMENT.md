# Deployment Guide

## 🚀 Local Development Setup

MCP Context Browser currently supports local deployment for development and testing. The system is designed as an MCP server that communicates via stdio with AI assistants.

## 📦 Installation

### Prerequisites

-   **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
-   **Git**: For cloning the repository

### Build from Source

```bash
# Clone the repository
git clone https://github.com/marlonsc/mcp-context-browser.git
cd mcp-context-browser

# Build in debug mode (recommended for development)
cargo build

# Or build optimized release
cargo build --release
```

### Run the Server

```bash
# Run in debug mode (shows more output)
cargo run

# Or run the release build
./target/release/mcp-context-browser
```

The server will start and listen for MCP protocol messages on stdin/stdout. It currently provides placeholder responses for MCP tools.

## ⚙️ Configuration

### Basic Configuration

Create a `config.toml` file in the project root:

```toml
# Embedding provider configuration
[embedding_provider]
provider = "mock"  # Options: mock, openai, ollama, gemini, voyageai

# Vector store configuration
[vector_store]
provider = "memory"  # Options: memory, milvus, filesystem, encrypted
```

### Configuration Options

| Setting | Description | Default | Status |
|---------|-------------|---------|--------|
| `embedding_provider.provider` | Embedding provider to use | `"mock"` | ✅ Available |
| `vector_store.provider` | Vector storage backend | `"memory"` | ✅ Available |

## 🧪 Testing the Setup

### Verify Installation

```bash
# Check if binary was built
ls -la target/debug/mcp-context-browser

# Run basic help/version check (when implemented)
./target/debug/mcp-context-browser --version
```

### MCP Protocol Testing

The server communicates via the MCP protocol over stdin/stdout. To test manually:

```bash
# Send a simple MCP initialize message
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | ./target/debug/mcp-context-browser
```

## 🐳 Docker Development (Future)

> **Note**: Docker support is planned for future releases. Currently, only local Rust builds are supported.

## 🔧 Troubleshooting

### Common Issues

#### Build Failures

```bash
# Clean and rebuild
cargo clean
cargo build

# Check Rust version
rustc --version
cargo --version
```

#### Runtime Issues

```bash
# Enable debug logging (when implemented)
RUST_LOG=debug cargo run

# Check system resources
df -h  # Disk space
free -h  # Memory
```

### Getting Help

-   Check existing [GitHub Issues](https://github.com/marlonsc/mcp-context-browser/issues)
-   Review the [ARCHITECTURE.md](../architecture/ARCHITECTURE.md) for technical details
-   See [CONTRIBUTING.md](../developer/CONTRIBUTING.md) for development setup

## 🚀 Future Deployment Options

The following deployment configurations are planned for future releases:

-   **Docker containerization**
-   **Kubernetes orchestration**
-   **Multi-user support**
-   **Cloud-native deployments**

These will be documented as they become available.

---

## 🏢 Option 2: Distributed Service (Team/Enterprise)

**Best for**: Team collaboration, enterprise deployments, multi-user environments

### Kubernetes Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mcp-context-browser
  labels:
    app: mcp-context-browser
spec:
  replicas: 3
  selector:
    matchLabels:
      app: mcp-context-browser
  template:
    metadata:
      labels:
        app: mcp-context-browser
    spec:
      containers:
      - name: mcp-context-browser
        image: mcp-context-browser:latest
        ports:
        - containerPort: 3000
        env:
        - name: MCP_MODE
          value: "distributed"
        - name: STORAGE_PROVIDER
          value: "milvus"
        - name: MILVUS_URI
          value: "milvus-service:19530"
        - name: DATABASE_URL
          value: "postgresql://user:password@db:5432/mcp_db"
        - name: REDIS_URL
          value: "redis://redis:6379"
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
```

### Docker Compose (Development)

```yaml
# docker-compose.yml
version: '3.8'
services:
  mcp-context-browser:
    build: .
    ports:
      - "3000:3000"
      - "9090:9090"  # Metrics endpoint
    environment:
      - MCP_MODE=distributed
      - STORAGE_PROVIDER=milvus
      - MILVUS_URI=milvus:19530
      - DATABASE_URL=postgresql://user:password@postgres:5432/mcp_db
      - REDIS_URL=redis://redis:6379
      - JWT_SECRET=your-secret-key
    depends_on:
      - milvus
      - postgres
      - redis
    volumes:
      - ./config:/app/config:ro
      - ./data:/app/data

  milvus:
    image: milvusdb/milvus:latest
    ports:
      - "19530:19530"
      - "9091:9091"
    volumes:
      - milvus_data:/var/lib/milvus
    command: milvus run standalone

  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: mcp_db
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

volumes:
  milvus_data:
  postgres_data:
  redis_data:
```

### Enterprise Configuration

```toml
# config/enterprise.toml
[server]
host = "0.0.0.0"
port = 3000
workers = 4

[database]
url = "postgresql://user:password@localhost:5432/mcp_db"
max_connections = 20

[cache]
redis_url = "redis://localhost:6379"
ttl_seconds = 3600

[security]
jwt_secret = "your-256-bit-secret"
session_timeout = 3600

[storage]
provider = "milvus"
milvus_uri = "localhost:19530"
collection_prefix = "mcp_"

[ai]
default_provider = "openai"
openai_api_key = "${OPENAI_API_KEY}"
anthropic_api_key = "${ANTHROPIC_API_KEY}"
ollama_url = "http://localhost:11434"

[git]
repositories_path = "/var/lib/mcp/repositories"
max_repository_size = "1GB"
supported_vcs = ["git", "svn", "mercurial"]

[monitoring]
metrics_endpoint = "/metrics"
health_endpoint = "/health"
log_level = "info"

[compliance]
audit_log_enabled = true
gdpr_compliance = true
data_retention_days = 2555
```

### Load Balancing

```yaml
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: mcp-context-browser-lb
spec:
  selector:
    app: mcp-context-browser
  ports:
    - port: 80
      targetPort: 3000
  type: LoadBalancer

---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: mcp-context-browser-ingress
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  tls:
    - hosts:
        - mcp.yourcompany.com
      secretName: mcp-tls
  rules:
    - host: mcp.yourcompany.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: mcp-context-browser-lb
                port:
                  number: 80
```

---

## ☁️ Option 3: Hybrid Cloud-Edge

**Best for**: Global organizations, distributed teams, edge computing scenarios

### Architecture Overview

```text
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   AI Assistant  │◄──►│   Edge Node     │◄──►│  Cloud Service  │
│   (Distributed) │    │   (Local AI)   │    │   (Heavy AI)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │   Sync Layer    │
                    │   (Real-time)   │
                    └─────────────────┘
```

### Edge Node Configuration

```toml
# config/edge.toml
[deployment]
mode = "edge"
cloud_sync_enabled = true
cloud_endpoint = "https://mcp.yourcompany.com"
sync_interval_seconds = 30

[storage]
primary_provider = "milvus"
backup_provider = "file"
sync_to_cloud = true

[ai]
local_models = ["nomic-embed-text", "codellama:7b"]
cloud_fallback = true
offline_mode = true

[cache]
local_cache_size = "2GB"
sync_cache = true
prefetch_intelligence = true
```

### Cloud Service Configuration

```toml
# config/cloud.toml
[deployment]
mode = "cloud"
multi_tenant = true
edge_sync_enabled = true

[storage]
primary_provider = "milvus"
distributed = true
replicas = 3

[ai]
providers = ["openai", "anthropic", "ollama"]
load_balancing = true
model_routing = true

[scaling]
auto_scale_enabled = true
min_instances = 3
max_instances = 50
cpu_threshold = 70
memory_threshold = 80
```

### Synchronization Configuration

```toml
# config/sync.toml
[sync]
enabled = true
mode = "bidirectional"
conflict_resolution = "timestamp"

[edge_to_cloud]
intelligence_sync = true
codebase_sync = false  # Privacy: code stays local
usage_metrics = true

[cloud_to_edge]
model_updates = true
intelligence_updates = true
configuration_updates = true

[network]
compression = true
encryption = true
bandwidth_limits = "10Mbps"
retry_policy = "exponential_backoff"
```

---

## 💾 Storage Provider Configuration

### Milvus (Primary Vector Database)

```yaml
# milvus-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: milvus-config
data:
  milvus.yaml: |
    # Milvus configuration
    etcd:
      endpoints:
        - etcd-service:2379
    minio:
      address: minio-service
      port: 9000
      accessKeyID: minioadmin
      secretAccessKey: minioadmin
      useSSL: false
      bucketName: "milvus-bucket"
    pulsar:
      address: pulsar-service
      port: 6650
    common:
      defaultPartitionName: "_default"
      defaultIndexName: "_default_idx"
      retentionDuration: 432000
      entityExpiration: -1
      indexSliceSize: 16
```

### PostgreSQL (Hybrid Storage)

```sql
-- Initialize database
CREATE DATABASE mcp_context_browser;

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS vector;  -- For pgvector

-- Create tables
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    tenant_id UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE repositories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id),
    tenant_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    url TEXT NOT NULL,
    vcs_type VARCHAR(50) DEFAULT 'git',
    indexed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE code_embeddings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    repository_id UUID REFERENCES repositories(id),
    file_path TEXT NOT NULL,
    content_hash VARCHAR(64) NOT NULL,
    embedding vector(768),  -- Adjust dimension based on model
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_code_embeddings_repository ON code_embeddings(repository_id);
CREATE INDEX idx_code_embeddings_embedding ON code_embeddings USING ivfflat (embedding vector_cosine_ops);
```

### Redis (Caching & Sessions)

```yaml
# redis-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: redis-config
data:
  redis.conf: |
    # Redis configuration
    maxmemory 256mb
    maxmemory-policy allkeys-lru
    tcp-keepalive 300
    timeout 300
    databases 16
    save 900 1
    save 300 10
    save 60 10000
```

---

## 🔧 Configuration Management

### Environment Variables

```bash
# Core settings
export MCP_MODE=distributed
export MCP_HOST=0.0.0.0
export MCP_PORT=3000

# Database
export DATABASE_URL=postgresql://user:password@host:5432/db
export REDIS_URL=redis://host:6379

# AI Providers
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...
export OLLAMA_URL=http://localhost:11434

# Storage
export MILVUS_URI=localhost:19530
export MILVUS_TOKEN=token

# Security
export JWT_SECRET=your-256-bit-secret
export SESSION_TIMEOUT=3600

# Git Integration
export GIT_REPOSITORIES_PATH=/var/lib/mcp/repos
export GIT_MAX_SIZE=1GB

# Monitoring
export METRICS_ENDPOINT=/metrics
export LOG_LEVEL=info
```

### Configuration Validation

```bash
# Validate configuration
cargo run --bin config-validator -- config.toml

# Check environment
cargo run --bin env-check

# Test connections
cargo run --bin connectivity-test
```

---

## 📊 Monitoring & Observability

### Metrics Endpoints

```bash
# Prometheus metrics
curl http://localhost:9090/metrics

# Health check
curl http://localhost:3000/health

# Readiness check
curl http://localhost:3000/ready
```

### Logging Configuration

```toml
[logging]
level = "info"
format = "json"
outputs = ["stdout", "file", "loki"]

[logging.file]
path = "/var/log/mcp-context-browser.log"
max_size = "100MB"
retention = "30d"

[logging.loki]
url = "http://loki:3100"
labels = { service = "mcp-context-browser", tenant = "${TENANT_ID}" }
```

### Distributed Tracing

```toml
[tracing]
enabled = true
service_name = "mcp-context-browser"
exporter = "jaeger"

[tracing.jaeger]
endpoint = "http://jaeger:14268/api/traces"
```

---

## 🔒 Security Configuration

### Authentication & Authorization

```toml
[security]
auth_provider = "jwt"
session_store = "redis"

[security.jwt]
algorithm = "HS256"
expiration_hours = 24

[security.oauth]
github_client_id = "${GITHUB_CLIENT_ID}"
github_client_secret = "${GITHUB_CLIENT_SECRET}"
google_client_id = "${GOOGLE_CLIENT_ID}"
google_client_secret = "${GOOGLE_CLIENT_SECRET}"
```

### Data Encryption

```toml
[encryption]
at_rest = true
in_transit = true
key_rotation_days = 90

[encryption.keys]
master_key = "${MASTER_ENCRYPTION_KEY}"
data_key_rotation = true
```

### Network Security

```toml
[network]
tls_enabled = true
certificate_path = "/etc/ssl/certs/mcp.crt"
private_key_path = "/etc/ssl/private/mcp.key"
ciphers = ["ECDHE-RSA-AES256-GCM-SHA384", "ECDHE-RSA-AES128-GCM-SHA256"]
```

---

## 🚀 Scaling & Performance

### Auto-Scaling Rules

```yaml
# k8s/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: mcp-context-browser-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: mcp-context-browser
  minReplicas: 3
  maxReplicas: 50
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: http_requests_per_second
      target:
        type: AverageValue
        averageValue: "100"
```

### Performance Tuning

```toml
[performance]
worker_threads = 4
max_connections = 1000
connection_timeout_seconds = 30
query_timeout_seconds = 60

[performance.caching]
enabled = true
ttl_seconds = 3600
max_size_mb = 512

[performance.database]
connection_pool_size = 20
statement_cache_size = 100
query_timeout_seconds = 30
```

This deployment guide provides comprehensive instructions for deploying MCP Context Browser in various environments, from local development to enterprise-scale distributed systems.

---

## Cross-References

-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
-   **Contributing**: [CONTRIBUTING.md](../developer/CONTRIBUTING.md)
-   **Changelog**: [CHANGELOG.md](./CHANGELOG.md)
-   **Roadmap**: [ROADMAP.md](../developer/ROADMAP.md)
-   **Module Documentation**: [docs/modules/](../modules/)
