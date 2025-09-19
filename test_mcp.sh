#!/bin/bash

# Test script for MCP server

SERVER_BINARY="/workspaces/ai-orchestrator-hub/backend/target/debug/mcp_server"

if [ ! -f "$SERVER_BINARY" ]; then
    echo "MCP server binary not found. Building..."
    cd backend
    . $HOME/.cargo/env
    cargo build
    cd ..
fi

echo "Testing MCP server..."

# Test initialize
echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"clientInfo": {"name": "test-client", "version": "1.0.0"}}}' | $SERVER_BINARY

# Test list tools
echo '{"jsonrpc": "2.0", "id": 2, "method": "tools/list"}' | $SERVER_BINARY

# Test system info tool
echo '{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "system_info", "arguments": {}}}' | $SERVER_BINARY

# Test create agent
echo '{"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "create_swarm_agent", "arguments": {"type": "worker"}}}' | $SERVER_BINARY

# Test get status
echo '{"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "get_swarm_status", "arguments": {}}}' | $SERVER_BINARY

echo "MCP testing completed."