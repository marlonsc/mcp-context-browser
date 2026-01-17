# Configuration Guide

**MCP Context Browser**uses environment variables and configuration files for security-first setup. This guide documents all configuration options for production deployment.

## Security Design Principles

1.**No Hardcoded Credentials**- All secrets must come from environment or config files
2.**Graceful Degradation**- Missing optional credentials disable features, never crash
3.**Environment Variables First**- Configuration follows 12-factor app principles
4.**Validation by Default**- All credentials validated at startup
5.**Production Warnings**- Security issues logged at startup with clear guidance

## Quick Start

### Local Development (Auth Disabled)

```bash

# Minimal setup - auth disabled, all services optional
cargo run
```

### Production with Auth

```bash

# Set required credentials
export ADMIN_USERNAME=admin
export ADMIN_PASSWORD="your-secure-password"
export JWT_SECRET="your-32-character-minimum-jwt-secret-key"

# Optional: enable database
export DATABASE_URL="postgresql://user:pass@localhost:5432/mcp"

# Run server
cargo run --release
```

## Configuration Priority

1.**Environment Variables**- Highest priority (production)
2.**Config File**- Secondary (optional `~/.config/mcp-context-browser/config.toml`)
3.**Defaults**- Lowest priority (local development)

## Environment Variables Reference

### Admin Interface Configuration

**Required for Production**(if enabling admin dashboard)

| Variable | Type | Min Length | Default | Purpose |
|----------|------|-----------|---------|---------|
| `ADMIN_USERNAME` | String | 1 char | None | Admin login username |
| `ADMIN_PASSWORD` | String | 8 chars | None | Admin login password |
| `JWT_SECRET` | String | 32 chars | None | JWT signing secret |
| `JWT_EXPIRATION` | u64 | 1+ | 3600 | Token expiration in seconds |

**Behavior:**
\1-   If ALL THREE (`ADMIN_USERNAME`, `ADMIN_PASSWORD`, `JWT_SECRET`) are set and valid:
\1-   ✅ Admin interface ENABLED
\1-   Admin user created automatically
\1-   Credentials stored securely
\1-   If ANY are missing or invalid:
\1-   ✅ Server starts normally
\1-   ❌ Admin interface DISABLED
\1-  **No error or crash**- graceful degradation

**Example:**

```bash
export ADMIN_USERNAME="admin"
export ADMIN_PASSWORD="secure-8-char-minimum-password"
export JWT_SECRET="your-32-character-minimum-jwt-secret-key"
export JWT_EXPIRATION="86400"  # 24 hours
```

### Authentication Configuration

**Optional**(for API JWT authentication)

| Variable | Type | Min Length | Default | Purpose |
|----------|------|-----------|---------|---------|
| `JWT_SECRET` | String | 32 chars | Empty | JWT signing secret (also used for admin) |
| `ADMIN_PASSWORD` | String | 8 chars | Empty | Admin password (also used for admin) |

**Behavior:**
\1-   If BOTH `JWT_SECRET` AND `ADMIN_PASSWORD` are set and valid:
\1-   ✅ Authentication ENABLED
\1-   Admin user created with hashed password
\1-   JWT tokens required for API access
\1-   If either is missing:
\1-   ✅ Authentication DISABLED
\1-   API endpoints accessible without auth
\1-  **No error or crash**- graceful degradation

**Example:**

```bash
export JWT_SECRET="minimum-32-char-jwt-secret-key"
export ADMIN_PASSWORD="min-8-char-password"
```

### Database Configuration

**Optional**(PostgreSQL required only if database enabled)

| Variable | Type | Min | Default | Purpose |
|----------|------|-----|---------|---------|
| `DATABASE_URL` | String | 1 | Empty | PostgreSQL connection String (`postgresql://user:pass@host:5432/db`) |
| `DATABASE_MAX_CONNECTIONS` | u32 | 1 | 20 | Connection pool size |
| `DATABASE_MIN_IDLE` | u32 | 0 | 5 | Minimum idle connections |
| `DATABASE_MAX_LIFETIME_SECS` | u64 | 0 | 1800 | Max connection lifetime (seconds) |
| `DATABASE_IDLE_TIMEOUT_SECS` | u64 | 0 | 600 | Idle timeout (seconds) |
| `DATABASE_CONNECTION_TIMEOUT_SECS` | u64 | 0 | 30 | Connection timeout (seconds) |

**Behavior:**
\1-   If `DATABASE_URL` is set and valid:
\1-   ✅ Database ENABLED
\1-   Connection pool created with specified parameters
\1-   Health checks performed at startup
\1-   If `DATABASE_URL` is empty or missing:
\1-   ✅ Database DISABLED
\1-   All database operations use in-memory fallbacks
\1-  **No error or crash**- graceful degradation

**Example:**

```bash
export DATABASE_URL="postgresql://user:password@localhost:5432/mcp_context"
export DATABASE_MAX_CONNECTIONS="30"
export DATABASE_MIN_IDLE="10"
export DATABASE_MAX_LIFETIME_SECS="1800"
export DATABASE_IDLE_TIMEOUT_SECS="600"
export DATABASE_CONNECTION_TIMEOUT_SECS="30"
```

### Vector Store Configuration

**Optional**(defaults to in-memory if not configured)

| Variable | Type | Default | Purpose |
|----------|------|---------|---------|
| `VECTOR_STORE_PROVIDER` | String | `in-memory` | Vector store backend: `milvus`, `edgevec`, `in-memory`, `filesystem` |
| `MILVUS_ADDRESS` | String | Empty | Milvus server address (required for Milvus provider) |
| `MILVUS_TOKEN` | String | Empty | Milvus authentication token (optional) |

**Example:**

```bash
export VECTOR_STORE_PROVIDER="milvus"
export MILVUS_ADDRESS="localhost:19530"
export MILVUS_TOKEN="your-milvus-token"
```

### Embedding Provider Configuration

**Optional**(defaults to Ollama if not configured)

| Variable | Type | Default | Purpose |
|----------|------|---------|---------|
| `EMBEDDING_PROVIDER` | String | `ollama` | Embedding provider: `ollama`, `openai`, `voyageai`, `gemini` |
| `OLLAMA_BASE_URL` | String | `http://localhost:11434` | Ollama server URL |
| `OLLAMA_MODEL` | String | `nomic-embed-text` | Model name for Ollama |
| `OPENAI_API_KEY` | String | Empty | OpenAI API key (required for OpenAI provider) |
| `VOYAGEAI_API_KEY` | String | Empty | VoyageAI API key (required for voyageai provider) |
| `GEMINI_API_KEY` | String | Empty | Google Gemini API key (required for gemini provider) |

**Example:**

```bash
export EMBEDDING_PROVIDER="ollama"
export OLLAMA_BASE_URL="http://localhost:11434"
export OLLAMA_MODEL="nomic-embed-text"
```

### Server Configuration

**Optional**(defaults provided)

| Variable | Type | Default | Purpose |
|----------|------|---------|---------|
| `SERVER_HOST` | String | `0.0.0.0` | Server bind address |
| `SERVER_PORT` | u16 | `3000` | Server port |
| `METRICS_PORT` | u16 | `3001` | Metrics/admin HTTP port |
| `MCP__TRANSPORT__MODE` | String | `Hybrid` | Transport mode: `Stdio`, `Http`, `Hybrid` |

**Example:**

```bash
export SERVER_HOST="127.0.0.1"
export SERVER_PORT="3000"
export METRICS_PORT="3001"
export MCP__TRANSPORT__MODE="Hybrid"
```

## Configuration Files

### Default Location

```
~/.config/mcp-context-browser/config.toml
```

### Example Configuration File

```toml
[server]
host = "0.0.0.0"
port = 3000

[metrics]
enabled = true
port = 3001

[providers]
embedding_provider = "ollama"
vector_store_provider = "milvus"

[database]
enabled = false
max_connections = 20

[cache]
enabled = true
ttl_seconds = 3600
```

## Startup Security Checks

At server startup, the following security validations occur:

### Admin Configuration

```
✅ If all credentials present and valid:
\1-   Admin username: validated (non-empty)
\1-   Admin password: validated (8+ chars)
\1-   JWT secret: validated (32+ chars)
\1-   Password: hashed with Argon2id
   → Admin interface ENABLED

❌ If any credential invalid:
\1-   Error logged and ADMIN DISABLED
\1-   Server continues normally
```

### Authentication Configuration

```
✅ If JWT secret and admin password valid:
\1-   JWT secret length: validated (32+ chars)
\1-   Auth ENABLED for API endpoints

❌ If invalid:
\1-   Auth DISABLED
\1-   API endpoints accessible without auth
```

### Database Configuration

```
✅ If DATABASE_URL is set:
\1-   Connection string validated
\1-   Pool created with configured parameters
\1-   Health check performed
\1-   Database ENABLED

❌ If DATABASE_URL is empty:
\1-   Database DISABLED
\1-   In-memory fallback used
```

## Security Warnings

The server logs security issues at startup. Common warnings:

### Critical (Application blocks)

```
[SECURITY] ADMIN_INTERFACE_DISABLED: Admin credentials required
[SECURITY] INSECURE_JWT_SECRET: JWT secret < 32 characters
```

### High (Should fix)

```
[SECURITY] WEAK_PASSWORD: Admin password < 8 characters
[SECURITY] MISSING_DATABASE_URL: Database not configured
```

### Info (Monitor)

```
[SECURITY] AUTH_DISABLED: Authentication not configured
[SECURITY] ADMIN_INTERFACE_DISABLED: Admin not configured
```

## Production Deployment Checklist

\1-   [ ]**Admin Credentials**
\1-   [ ] Set `ADMIN_USERNAME` (non-empty)
\1-   [ ] Set `ADMIN_PASSWORD` (8+ chars, complex)
\1-   [ ] Set `JWT_SECRET` (32+ chars, random)
\1-   [ ] Verify admin interface starts with `✅ Admin interface enabled`

\1-   [ ]**Authentication**
\1-   [ ] JWT tokens working for API access
\1-   [ ] Token expiration set appropriately
\1-   [ ] Revocation working (if needed)

\1-   [ ]**Database**(if using PostgreSQL)
\1-   [ ] Set `DATABASE_URL` with production credentials
\1-   [ ] Connection pool parameters tuned
\1-   [ ] Health checks passing
\1-   [ ] Backups configured

\1-   [ ]**Embedding & Vector Stores**
\1-   [ ] Embedding provider configured
\1-   [ ] Vector store accessible
\1-   [ ] Indexing working

\1-   [ ]**Monitoring**
\1-   [ ] Metrics endpoint accessible
\1-   [ ] Logs configured
\1-   [ ] Alerts set up

\1-   [ ]**Security**
\1-   [ ] No hardcoded credentials in code
\1-   [ ] Environment variables validated
\1-   [ ] Secrets stored securely
\1-   [ ] HTTPS enabled (if exposed publicly)

## Troubleshooting

### Admin Interface Not Appearing

**Cause:**Missing or invalid credentials

```bash

# Check what's configured
echo "ADMIN_USERNAME: $ADMIN_USERNAME"
echo "ADMIN_PASSWORD: ${ADMIN_PASSWORD:0:5}***"
echo "JWT_SECRET: ${JWT_SECRET:0:5}***"

# JWT_SECRET must be 32+ chars
if [ ${#JWT_SECRET} -lt 32 ]; then
  echo "ERROR: JWT_SECRET too short (${#JWT_SECRET} < 32)"
fi

# Password must be 8+ chars
if [ ${#ADMIN_PASSWORD} -lt 8 ]; then
  echo "ERROR: ADMIN_PASSWORD too short (${#ADMIN_PASSWORD} < 8)"
fi
```

### Database Connection Failed

**Cause:**Invalid `DATABASE_URL`

```bash

# Verify connection string format

# postgresql://username:password@hostname:5432/database

# Test connection manually
psql "$DATABASE_URL" -c "SELECT version();"
```

### Authentication Not Working

**Cause:**JWT_SECRET not set or too short

```bash

# Generate secure JWT secret (32+ chars)
openssl rand -base64 32

# Set it
export JWT_SECRET="$(openssl rand -base64 32)"
```

### High Memory Usage

**Cause:**Connection pool or cache too large

```bash

# Reduce connection pool
export DATABASE_MAX_CONNECTIONS="10"

# Reduce cache TTL
export CACHE_TTL_SECONDS="1800"
```

## Related Documentation

\1-   [Architecture Overview](./architecture/ARCHITECTURE.md)
\1-   [Security Guide](./security/SECURITY.md)
\1-   [Admin API Reference](./api/admin-api.md)
\1-   [Deployment Guide](./deployment/DEPLOYMENT.md)

---

**Last Updated:** 2026-01-17
**Version:** 0.1.1
