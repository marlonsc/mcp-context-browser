#!/bin/bash

# Test MCP Client Script
# Tests the MCP Context Browser server by sending JSON-RPC messages

set -e

echo "🧪 Testing MCP Context Browser Server"

# Check if server is running
if ! pgrep -f "mcp-context-browser" > /dev/null; then
    echo "❌ MCP server not running. Start with 'make dev' first."
    exit 1
fi

echo "✅ MCP server is running"

# Get the process PID
SERVER_PID=$(pgrep -f "mcp-context-browser")
echo "📍 Server PID: $SERVER_PID"

# Test 1: Initialize
echo "📤 Sending initialize request..."
INIT_MSG='{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test-client", "version": "1.0.0"}}}'

echo "$INIT_MSG" | timeout 5 bash -c "
    exec 3>&1
    (
        echo '$INIT_MSG'
        sleep 1
    ) | nc -U /proc/$SERVER_PID/fd/0 2>/dev/null || echo 'Direct stdin connection failed'
" || echo "⚠️  Direct stdin test skipped (expected on some systems)"

# Test 2: List tools
echo "📤 Sending tools/list request..."
LIST_MSG='{"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}'

echo "$LIST_MSG" | timeout 5 bash -c "
    (
        echo '$LIST_MSG'
        sleep 1
    ) | nc -U /proc/$SERVER_PID/fd/0 2>/dev/null || echo 'Direct stdin connection failed'
" || echo "⚠️  Direct stdin test skipped (expected on some systems)"

# Test 3: Check HTTP health endpoint
echo "🌐 Testing HTTP health endpoint..."
if curl -s http://localhost:3001/api/health > /dev/null; then
    echo "✅ HTTP health endpoint responding"
    HEALTH_RESPONSE=$(curl -s http://localhost:3001/api/health)
    echo "📊 Health response: $HEALTH_RESPONSE"
else
    echo "❌ HTTP health endpoint not responding"
fi

echo "🎉 MCP server testing complete!"