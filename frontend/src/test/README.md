# WebSocket Mocking for Playwright E2E Tests

This directory contains utilities for mocking WebSocket connections in Playwright E2E tests, enabling testing of real-time features without requiring a live backend server.

## Overview

The WebSocket mocking system provides:

- **Mock WebSocket Server**: Simulates the backend WebSocket server behavior
- **Real-time Data Simulation**: Generates realistic agent, task, and metrics data
- **Test Utilities**: Helper functions for common testing patterns
- **Connection Management**: Automatic setup and teardown of mock servers

## Quick Start

### 1. Install Dependencies

```bash
npm install
```

### 2. Run WebSocket Mock Tests

```bash
# Run all WebSocket mock tests
npm run test:e2e:websocket

# Run with UI mode for debugging
npm run test:e2e:websocket:ui

# Run specific test file
npx playwright test websocket-mock.spec.ts
```

### 3. Using in Your Tests

```typescript
import { test, WebSocketTestUtils } from '../src/test/playwright-websocket-utils'

test('my websocket test', async ({ page, mockWebSocket }) => {
  // Navigate to your app
  await page.goto('/')

  // Wait for WebSocket connection and initial data
  await WebSocketTestUtils.waitForInitialLoad(page)

  // Your test logic here
  // The mockWebSocket instance is available for direct manipulation
})
```

## Architecture

### MockWebSocketServer

The core mock server that simulates backend WebSocket behavior:

```typescript
import { MockWebSocketServer } from './websocket-mock'

const server = new MockWebSocketServer(3001)
await server.start()

// Server automatically:
// - Sends initial hive status on connection
// - Sends periodic updates (agents, tasks, metrics)
// - Handles client messages (create_agent, create_task, etc.)
// - Simulates realistic data changes

await server.stop()
```

### Message Types

The mock server handles these WebSocket message types:

#### Incoming Messages (Client → Server)

- `create_agent` - Create a new agent
- `create_task` - Create a new task
- `get_status` - Request hive status
- `ping` - Heartbeat/ping message

#### Outgoing Messages (Server → Client)

- `hive_status` - Initial and requested hive status
- `agents_update` - Agent list updates
- `tasks_update` - Task list updates
- `metrics_update` - Real-time metrics updates
- `agent_created` - Response to agent creation
- `task_created` - Response to task creation
- `error` - Error messages

### Mock Data Structure

```typescript
interface MockHiveData {
  hive_id: string
  created_at: string
  last_update: string
  metrics: {
    total_agents: number
    active_agents: number
    completed_tasks: number
    failed_tasks: number
    average_performance: number
    swarm_cohesion: number
    learning_progress: number
  }
  swarm_center: [number, number]
  total_energy: number
}

interface MockAgent {
  id: string
  name: string
  type: string
  state: string
  capabilities: Array<{
    name: string
    proficiency: number
    learning_rate: number
  }>
  position: [number, number]
  energy: number
  experience_count: number
  social_connections: number
}

interface MockTask {
  id: string
  description: string
  type: string
  priority: number
  status: string
  assigned_agent?: string
  created_at: string
  completed_at?: string
}
```

## Test Utilities

### WebSocketTestUtils

Pre-built utilities for common testing patterns:

```typescript
import { WebSocketTestUtils } from './playwright-websocket-utils'

// Wait for initial WebSocket connection and data load
await WebSocketTestUtils.waitForInitialLoad(page)

// Test agent creation workflow
const agentId = await WebSocketTestUtils.testAgentCreation(page, {
  type: 'Worker',
  name: 'Test Agent',
})

// Test task creation workflow
const taskId = await WebSocketTestUtils.testTaskCreation(page, {
  description: 'Test task',
  priority: 3,
})

// Test reconnection scenarios
await WebSocketTestUtils.testReconnection(page)

// Test error handling
await WebSocketTestUtils.testErrorHandling(page, 'Custom error message')
```

### Manual Control Functions

```typescript
import {
  waitForWebSocketConnection,
  waitForWebSocketMessage,
  sendMockWebSocketMessage,
  simulateWebSocketError,
  getMockData,
} from './playwright-websocket-utils'

// Wait for connection
await waitForWebSocketConnection(page)

// Wait for specific message type
await waitForWebSocketMessage(page, 'agents_update')

// Send custom message from mock server
await sendMockWebSocketMessage('custom_event', { data: 'test' })

// Simulate error
await simulateWebSocketError('Connection failed')

// Get current mock data for assertions
const mockData = getMockData()
expect(mockData.agents.length).toBeGreaterThan(0)
```

## Configuration

### Environment Variables

Set these in your test environment:

```bash
# WebSocket URL for mock server
NEXT_PUBLIC_WS_URL=ws://localhost:3001/ws

# Enable mock WebSocket mode
USE_MOCK_WEBSOCKET=true
```

### Playwright Configuration

The `playwright.config.ts` is already configured to:

- Start the mock WebSocket server before tests
- Stop the server after tests complete
- Set environment variables for the frontend
- Pass WebSocket URL to all browser contexts

## Writing Tests

### Basic Test Structure

```typescript
import { test } from '../src/test/playwright-websocket-utils'
import { expect } from '@playwright/test'

test.describe('My WebSocket Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    // Wait for WebSocket connection
    await waitForWebSocketConnection(page)
  })

  test('should handle real-time updates', async ({ page, mockWebSocket }) => {
    // Test logic here
    const initialCount = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.agents?.length || 0
    })

    // Trigger mock update
    mockWebSocket.triggerUpdate()

    // Verify UI updates
    await page.waitForFunction(count => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.agents?.length > count
    }, initialCount)
  })
})
```

### Testing Real-time Features

```typescript
test('should display live agent metrics', async ({ page }) => {
  // Wait for initial data
  await waitForWebSocketMessage(page, 'agents_update')

  // Get initial metrics
  const initialMetrics = await page.evaluate(() => {
    const store = (window as any).__ZUSTAND_STORE__
    return store?.hiveStatus?.metrics
  })

  // Wait for periodic update
  await page.waitForTimeout(6000) // Mock updates every 5 seconds

  // Verify metrics changed
  const updatedMetrics = await page.evaluate(() => {
    const store = (window as any).__ZUSTAND_STORE__
    return store?.hiveStatus?.metrics
  })

  expect(updatedMetrics.average_performance).not.toBe(initialMetrics.average_performance)
})
```

### Testing Error Scenarios

```typescript
test('should handle connection errors', async ({ page, mockWebSocket }) => {
  // Wait for stable connection
  await waitForWebSocketConnection(page)

  // Simulate server error
  mockWebSocket.simulateError('Server maintenance')

  // Verify error handling
  await page.waitForFunction(() => {
    const store = (window as any).__ZUSTAND_STORE__
    return store?.lastError !== null
  })

  // Verify UI shows error state
  await expect(page.locator('.error-message')).toBeVisible()
})
```

## Advanced Usage

### Custom Mock Data

```typescript
// Create server with custom initial data
const customServer = new MockWebSocketServer(3001)

// Modify mock data directly
const mockData = customServer.getMockData()
mockData.agents.push({
  id: 'custom-agent',
  name: 'Custom Agent',
  type: 'Specialist',
  state: 'active',
  // ... other properties
})

// Trigger update to broadcast changes
customServer.triggerUpdate()
```

### Testing Specific Scenarios

```typescript
test('should handle high-frequency updates', async ({ page, mockWebSocket }) => {
  // Send rapid updates
  for (let i = 0; i < 10; i++) {
    mockWebSocket.triggerUpdate()
    await page.waitForTimeout(100)
  }

  // Verify app remains stable
  const isConnected = await page.evaluate(() => {
    const store = (window as any).__ZUSTAND_STORE__
    return store?.isConnected
  })

  expect(isConnected).toBe(true)
})
```

### Integration with CI/CD

```yaml
# .github/workflows/e2e.yml
name: E2E Tests
on: [push, pull_request]

jobs:
  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
      - run: npm ci
      - run: npm run test:e2e:websocket
        env:
          NEXT_PUBLIC_WS_URL: ws://localhost:3001/ws
          USE_MOCK_WEBSOCKET: true
```

## Troubleshooting

### Common Issues

1. **WebSocket connection fails**
   - Check that port 3001 is available
   - Verify firewall settings
   - Ensure no other WebSocket server is running on the same port

2. **Mock data not updating**
   - Check that `triggerUpdate()` is called after data modifications
   - Verify WebSocket connection is established
   - Check browser console for WebSocket errors

3. **Tests timing out**
   - Increase timeout values for WebSocket operations
   - Ensure mock server is started before tests
   - Check that the frontend is configured to use the mock WebSocket URL

### Debug Mode

Enable debug logging:

```typescript
// In your test
page.on('console', msg => console.log('PAGE LOG:', msg.text()))

// Mock server logs are automatically shown in console
```

### Manual Testing

Test the mock server independently:

```bash
# Start mock server manually
node -e "
const { MockWebSocketServer } = require('./src/test/websocket-mock.ts');
const server = new MockWebSocketServer(3001);
server.start().then(() => console.log('Mock server running'));
"

# Connect with a WebSocket client
# Use browser dev tools or a tool like wscat
wscat -c ws://localhost:3001/ws
```

## Best Practices

1. **Use descriptive test names** that indicate what's being tested
2. **Wait for WebSocket connections** before performing actions
3. **Clean up resources** in afterEach hooks if needed
4. **Test both success and error scenarios**
5. **Use realistic mock data** that matches your application's expectations
6. **Test reconnection scenarios** to ensure robustness
7. **Verify UI updates** after WebSocket messages
8. **Test performance** with high-frequency updates

## API Reference

### MockWebSocketServer

- `start(): Promise<void>` - Start the mock server
- `stop(): Promise<void>` - Stop the mock server
- `getMockData(): MockData` - Get current mock data
- `triggerUpdate(): void` - Trigger a manual update
- `simulateError(message: string): void` - Simulate an error
- `getConnectedClientsCount(): number` - Get number of connected clients

### Test Utilities

- `waitForWebSocketConnection(page, timeout?)` - Wait for WebSocket connection
- `waitForWebSocketMessage(page, messageType, timeout?)` - Wait for specific message
- `sendMockWebSocketMessage(messageType, data)` - Send message from mock server
- `simulateWebSocketError(errorMessage)` - Simulate WebSocket error
- `getMockData()` - Get current mock data
- `WebSocketTestUtils` - Collection of high-level test utilities

This WebSocket mocking system enables comprehensive testing of real-time features without external dependencies, ensuring reliable and fast E2E tests for your multiagent hive system.
