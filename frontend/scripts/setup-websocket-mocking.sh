#!/bin/bash

# Setup script for WebSocket mocking in Playwright E2E tests
# This script installs dependencies and verifies the setup

set -e

echo "🚀 Setting up WebSocket mocking for Playwright E2E tests..."

# Check if we're in the frontend directory
if [ ! -f "package.json" ]; then
    echo "❌ Error: Please run this script from the frontend directory"
    exit 1
fi

# Install WebSocket dependency
echo "📦 Installing WebSocket dependency..."
npm install --save-dev ws

# Verify installation
if ! npm list ws > /dev/null 2>&1; then
    echo "❌ Error: Failed to install ws package"
    exit 1
fi

echo "✅ WebSocket dependency installed successfully"

# Check if test files exist
if [ ! -f "src/test/websocket-mock.ts" ]; then
    echo "❌ Error: WebSocket mock files not found. Please ensure all mock files are in place."
    exit 1
fi

if [ ! -f "src/test/playwright-websocket-utils.ts" ]; then
    echo "❌ Error: WebSocket utilities not found. Please ensure all utility files are in place."
    exit 1
fi

if [ ! -f "src/test/global-setup.ts" ]; then
    echo "❌ Error: Global setup file not found. Please ensure all setup files are in place."
    exit 1
fi

if [ ! -f "src/test/global-teardown.ts" ]; then
    echo "❌ Error: Global teardown file not found. Please ensure all setup files are in place."
    exit 1
fi

echo "✅ All WebSocket mock files are present"

# Check if Playwright config is updated
if ! grep -q "globalSetup" playwright.config.ts; then
    echo "⚠️  Warning: Playwright config may not be updated with WebSocket mocking setup"
    echo "   Please ensure playwright.config.ts includes globalSetup and globalTeardown"
fi

# Check if environment variables are set
echo "🔧 Checking environment configuration..."
if [ -z "$NEXT_PUBLIC_WS_URL" ]; then
    echo "ℹ️  NEXT_PUBLIC_WS_URL not set, will use default: ws://localhost:3001/ws"
fi

if [ -z "$USE_MOCK_WEBSOCKET" ]; then
    echo "ℹ️  USE_MOCK_WEBSOCKET not set, will use default: true for tests"
fi

# Test the setup by running a quick syntax check
echo "🔍 Running syntax check on mock files..."
npx tsc --noEmit src/test/websocket-mock.ts src/test/playwright-websocket-utils.ts src/test/global-setup.ts src/test/global-teardown.ts

if [ $? -eq 0 ]; then
    echo "✅ All mock files pass TypeScript compilation"
else
    echo "❌ Error: TypeScript compilation failed for mock files"
    exit 1
fi

echo ""
echo "🎉 WebSocket mocking setup complete!"
echo ""
echo "📋 Next steps:"
echo "  1. Run WebSocket mock tests: npm run test:e2e:websocket"
echo "  2. Run with UI mode: npm run test:e2e:websocket:ui"
echo "  3. View test results: npx playwright show-report"
echo ""
echo "📚 Documentation: src/test/README.md"
echo "🔧 Configuration: playwright.config.ts"
echo ""
echo "💡 Tips:"
echo "  - Mock server runs on port 3001 by default"
echo "  - WebSocket URL: ws://localhost:3001/ws"
echo "  - Mock data updates every 5 seconds automatically"
echo "  - Use the mockWebSocket fixture in your tests for direct server control"