import { Page, test as baseTest } from '@playwright/test'
/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable react-hooks/rules-of-hooks */
import { MockWebSocketServer } from './websocket-mock'

/**
 * Playwright test utilities for WebSocket mocking
 */

// Global mock server instance
let mockServer: MockWebSocketServer | null = null

/**
 * Setup WebSocket mock server before tests
 */
export async function setupWebSocketMock(port = 3001): Promise<MockWebSocketServer> {
  if (mockServer) {
    await mockServer.stop()
  }

  mockServer = new MockWebSocketServer(port)
  await mockServer.start()
  return mockServer
}

/**
 * Teardown WebSocket mock server after tests
 */
export async function teardownWebSocketMock(): Promise<void> {
  if (mockServer) {
    await mockServer.stop()
    mockServer = null
  }
}

/**
 * Get the current mock server instance
 */
export function getMockServer(): MockWebSocketServer | null {
  return mockServer
}

/**
 * Wait for WebSocket connection to be established
 */
export async function waitForWebSocketConnection(page: Page, timeout = 5000): Promise<void> {
  await page.waitForFunction(
    () => {
      // Check if the WebSocket connection is established by looking at the store state
      const store = (window as any).__ZUSTAND_STORE__
      return store?.isConnected === true
    },
    { timeout },
  )
}

/**
 * Wait for a specific WebSocket message type
 */
export async function waitForWebSocketMessage(
  page: Page,
  messageType: string,
  timeout = 5000,
): Promise<any> {
  return page.waitForFunction(
    type => {
      // This would need to be implemented based on how messages are stored in the app
      // For now, we'll use a simple approach by checking the store state
      const store = (window as any).__ZUSTAND_STORE__
      if (!store) {
        return null
      }

      // Check different message types
      switch (type) {
        case 'hive_status':
          return store.hiveStatus
        case 'agents_update':
          return store.agents?.length > 0 ? store.agents : null
        case 'tasks_update':
          return store.tasks?.length > 0 ? store.tasks : null
        case 'metrics_update':
          return store.hiveStatus?.metrics ? store.hiveStatus.metrics : null
        default:
          return null
      }
    },
    messageType,
    { timeout },
  )
}

/**
 * Send a WebSocket message from the mock server
 */
export async function sendMockWebSocketMessage(messageType: string, data: any): Promise<void> {
  if (!mockServer) {
    throw new Error('Mock WebSocket server not initialized')
  }

  mockServer.broadcast({
    message_type: messageType,
    data,
    timestamp: new Date().toISOString(),
  })
}

/**
 * Simulate WebSocket connection error
 */
export async function simulateWebSocketError(errorMessage = 'Connection failed'): Promise<void> {
  if (!mockServer) {
    throw new Error('Mock WebSocket server not initialized')
  }

  mockServer.simulateError(errorMessage)
}

/**
 * Get current mock data for assertions
 */
export function getMockData() {
  if (!mockServer) {
    throw new Error('Mock WebSocket server not initialized')
  }

  return mockServer.getMockData()
}

/**
 * Trigger a manual update from the mock server
 */
export function triggerMockUpdate(): void {
  if (!mockServer) {
    throw new Error('Mock WebSocket server not initialized')
  }

  mockServer.triggerUpdate()
}

/**
 * Wait for the hive store to have specific data
 */
export async function waitForHiveData(
  page: Page,
  condition: (data: any) => boolean,
  timeout = 5000,
): Promise<any> {
  return page.waitForFunction(
    cond => {
      const store = (window as any).__ZUSTAND_STORE__
      if (!store) {
        return null
      }

      const data = {
        isConnected: store.isConnected,
        hiveStatus: store.hiveStatus,
        agents: store.agents,
        tasks: store.tasks,
      }

      return cond(data) ? data : null
    },
    condition,
    { timeout },
  )
}

/**
 * Custom test fixture that sets up WebSocket mocking
 */
export const test = baseTest.extend<{
  mockWebSocket: MockWebSocketServer
}>({
  mockWebSocket: async ({}, use) => {
    const server = await setupWebSocketMock()
    await use(server)
    await teardownWebSocketMock()
  },
})

/**
 * Test utilities for common WebSocket testing patterns
 */
export const WebSocketTestUtils = {
  /**
   * Wait for initial connection and data load
   */
  async waitForInitialLoad(page: Page): Promise<void> {
    // Wait for WebSocket connection
    await waitForWebSocketConnection(page)

    // Wait for initial hive status
    await waitForWebSocketMessage(page, 'hive_status')

    // Wait for initial agents data
    await waitForWebSocketMessage(page, 'agents_update')

    // Wait for initial tasks data
    await waitForWebSocketMessage(page, 'tasks_update')
  },

  /**
   * Test agent creation workflow
   */
  async testAgentCreation(page: Page, _agentConfig: any): Promise<string> {
    const initialAgentCount = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.agents?.length || 0
    })

    // Trigger agent creation (this would be done through the UI)
    // The actual implementation depends on your UI components

    // Wait for agent creation response
    await waitForWebSocketMessage(page, 'agent_created')

    // Wait for agents update
    await waitForWebSocketMessage(page, 'agents_update')

    // Verify agent count increased
    await page.waitForFunction(expectedCount => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.agents?.length === expectedCount
    }, initialAgentCount + 1)

    // Get the created agent ID
    const agentId = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      const agents = store?.agents || []
      return agents[agents.length - 1]?.id
    })

    return agentId
  },

  /**
   * Test task creation workflow
   */
  async testTaskCreation(page: Page, _taskConfig: any): Promise<string> {
    const initialTaskCount = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.tasks?.length || 0
    })

    // Trigger task creation (this would be done through the UI)
    // The actual implementation depends on your UI components

    // Wait for task creation response
    await waitForWebSocketMessage(page, 'task_created')

    // Wait for tasks update
    await waitForWebSocketMessage(page, 'tasks_update')

    // Verify task count increased
    await page.waitForFunction(expectedCount => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.tasks?.length === expectedCount
    }, initialTaskCount + 1)

    // Get the created task ID
    const taskId = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      const tasks = store?.tasks || []
      return tasks[tasks.length - 1]?.id
    })

    return taskId
  },

  /**
   * Test WebSocket reconnection
   */
  async testReconnection(page: Page): Promise<void> {
    // Wait for initial connection
    await waitForWebSocketConnection(page)

    // Simulate disconnection by stopping the mock server
    await teardownWebSocketMock()

    // Wait for disconnection to be detected
    await page.waitForFunction(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.isConnected === false
    })

    // Restart the mock server
    await setupWebSocketMock()

    // Wait for reconnection
    await waitForWebSocketConnection(page)
  },

  /**
   * Test error handling
   */
  async testErrorHandling(page: Page, errorMessage = 'Test error'): Promise<void> {
    // Simulate an error
    await simulateWebSocketError(errorMessage)

    // Wait for error to be handled (this depends on your error handling implementation)
    // You might need to check for error UI elements or console messages
  },
}

/**
 * Environment variable helper for WebSocket URL
 */
export function getWebSocketUrl(port = 3001): string {
  return `ws://localhost:${port}/ws`
}

/**
 * Setup function for test files
 */
export async function setupTestEnvironment(port = 3001): Promise<{
  mockServer: MockWebSocketServer
  wsUrl: string
}> {
  const mockServer = await setupWebSocketMock(port)
  const wsUrl = getWebSocketUrl(port)

  return { mockServer, wsUrl }
}
