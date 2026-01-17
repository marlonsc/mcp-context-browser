# daemon Module

**Note**: The daemon module from the legacy codebase has been simplified in v0.1.1.

**Previous Source**: `src/infrastructure/daemon/`
**Status**: Background service functionality integrated into mcb-server

## Overview

In v0.1.1, background daemon functionality is handled by:

-   **mcb-server**: Server lifecycle management
-   **Systemd**: External service management (see deployment docs)

## Migration Notes

The standalone daemon service has been replaced with:

1. Systemd service integration for production deployments
2. Built-in graceful shutdown in mcb-server
3. Health check endpoints for monitoring

## Related Documentation

-   **Server**: [server.md](./server.md) (server lifecycle)
-   **Deployment**: [DEPLOYMENT.md](../operations/DEPLOYMENT.md) (systemd setup)
-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

*Updated 2026-01-17 - Reflects modular crate architecture (v0.1.1)*
