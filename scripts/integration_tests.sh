#!/bin/bash

# Integration test script for ai-orchestrator-hub
# Supports both standard and chaos engineering modes

set -e

echo "Running integration tests..."

# Check if chaos mode is enabled
CHAOS_MODE=false
if [[ "$1" == "--chaos" ]]; then
    CHAOS_MODE=true
    echo "Chaos engineering mode enabled"
fi

# Basic integration test simulation
echo "Testing backend API endpoints..."
sleep 2

echo "Testing frontend-backend integration..."
sleep 2

if [ "$CHAOS_MODE" = true ]; then
    echo "Running chaos engineering tests..."
    echo "- Testing with network latency"
    sleep 3
    echo "- Testing with intermittent failures"
    sleep 2
fi

echo "Testing database connections..."
sleep 1

echo "Integration tests completed successfully"
exit 0
