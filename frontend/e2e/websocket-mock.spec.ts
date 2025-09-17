import { test } from '../src/test/playwright-websocket-utils'
import { expect } from '@playwright/test'

test.describe('WebSocket Mock E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the application
    await page.goto('/')

    // Wait for the main dashboard to load
    await page.waitForFunction(
      () => {
        const mainHeading = document.querySelector('h1:has-text("🐝 Multiagent Hive System")')
        return mainHeading !== null
      },
      { timeout: 10000 },
    )
  })

  test('should establish WebSocket connection with mock server', async ({
    page,
    mockWebSocket,
  }) => {
    // Wait for WebSocket connection to be established
    await page.waitForFunction(
      () => {
        // Check if the WebSocket connection is established
        const store = (window as any).__ZUSTAND_STORE__
        return store?.isConnected === true
      },
      { timeout: 5000 },
    )

    // Verify connection status is displayed
    const connectionStatus = page.locator('[class*="bg-green-100"]')
    await expect(connectionStatus).toBeVisible()
    await expect(connectionStatus).toContainText('Connected')

    // Verify mock server has at least one client connected
    expect(mockWebSocket.getConnectedClientsCount()).toBeGreaterThan(0)
  })

  test('should receive initial hive status from mock server', async ({ page }) => {
    // Wait for WebSocket connection and initial data
    await page.waitForFunction(
      () => {
        const store = (window as any).__ZUSTAND_STORE__
        return store?.hiveStatus !== null && store?.hiveStatus?.hive_id !== undefined
      },
      { timeout: 5000 },
    )

    // Verify hive status is displayed
    const hiveStatus = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.hiveStatus
    })

    expect(hiveStatus).toBeTruthy()
    expect(hiveStatus.hive_id).toBe('mock-hive-001')
    expect(hiveStatus.metrics.total_agents).toBeGreaterThan(0)
  })

  test('should receive periodic updates from mock server', async ({ page }) => {
    // Wait for initial data load
    await page.waitForFunction(
      () => {
        const store = (window as any).__ZUSTAND_STORE__
        return store?.agents?.length > 0
      },
      { timeout: 5000 },
    )

    const initialMetrics = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.hiveStatus?.metrics
    })

    // Wait for a periodic update (mock server updates every 5 seconds)
    await page.waitForTimeout(6000)

    // Check that metrics have been updated
    const updatedMetrics = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.hiveStatus?.metrics
    })

    // Metrics should be different (due to mock randomization)
    expect(updatedMetrics.average_performance).not.toBe(initialMetrics.average_performance)
  })

  test('should handle agent creation via WebSocket', async ({ page, mockWebSocket }) => {
    // Get initial agent count
    const initialAgentCount = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.agents?.length || 0
    })

    // Simulate agent creation by sending a WebSocket message
    // In a real test, this would be triggered by UI interaction
    await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      if (store?.createAgent) {
        store.createAgent({
          type: 'Worker',
          name: 'Test Agent',
          capabilities: ['processing'],
        })
      }
    })

    // Wait for agent creation response
    await page.waitForFunction(
      initialCount => {
        const store = (window as any).__ZUSTAND_STORE__
        return store?.agents?.length > initialCount
      },
      initialAgentCount,
      { timeout: 5000 },
    )

    // Verify agent count increased
    const finalAgentCount = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.agents?.length || 0
    })

    expect(finalAgentCount).toBe(initialAgentCount + 1)

    // Verify mock server data is updated
    const mockData = mockWebSocket.getMockData()
    expect(mockData.agents.length).toBe(finalAgentCount)
  })

  test('should handle task creation via WebSocket', async ({ page, mockWebSocket }) => {
    // Get initial task count
    const initialTaskCount = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.tasks?.length || 0
    })

    // Simulate task creation
    await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      if (store?.createTask) {
        store.createTask({
          description: 'Test task from WebSocket mock',
          priority: 3,
          type: 'processing',
        })
      }
    })

    // Wait for task creation response
    await page.waitForFunction(
      initialCount => {
        const store = (window as any).__ZUSTAND_STORE__
        return store?.tasks?.length > initialCount
      },
      initialTaskCount,
      { timeout: 5000 },
    )

    // Verify task count increased
    const finalTaskCount = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.tasks?.length || 0
    })

    expect(finalTaskCount).toBe(initialTaskCount + 1)

    // Verify mock server data is updated
    const mockData = mockWebSocket.getMockData()
    expect(mockData.tasks.length).toBe(finalTaskCount)
  })

  test('should handle WebSocket reconnection', async ({ page, mockWebSocket }) => {
    // Wait for initial connection
    await page.waitForFunction(
      () => {
        const store = (window as any).__ZUSTAND_STORE__
        return store?.isConnected === true
      },
      { timeout: 5000 },
    )

    // Simulate disconnection by stopping mock server
    await mockWebSocket.stop()

    // Wait for disconnection to be detected
    await page.waitForFunction(
      () => {
        const store = (window as any).__ZUSTAND_STORE__
        return store?.isConnected === false
      },
      { timeout: 10000 },
    )

    // Verify disconnected status is shown
    const connectionStatus = page.locator('[class*="bg-red-100"]')
    await expect(connectionStatus).toBeVisible()
    await expect(connectionStatus).toContainText(/(Disconnected|Reconnecting)/)

    // Restart mock server
    await mockWebSocket.start()

    // Wait for reconnection
    await page.waitForFunction(
      () => {
        const store = (window as any).__ZUSTAND_STORE__
        return store?.isConnected === true
      },
      { timeout: 10000 },
    )

    // Verify reconnected status
    const reconnectedStatus = page.locator('[class*="bg-green-100"]')
    await expect(reconnectedStatus).toBeVisible()
    await expect(reconnectedStatus).toContainText('Connected')
  })

  test('should handle WebSocket errors gracefully', async ({ page, mockWebSocket }) => {
    // Wait for initial connection
    await page.waitForFunction(
      () => {
        const store = (window as any).__ZUSTAND_STORE__
        return store?.isConnected === true
      },
      { timeout: 5000 },
    )

    // Simulate an error from the mock server
    mockWebSocket.simulateError('Test error from mock server')

    // The application should handle the error gracefully
    // (This depends on your error handling implementation)
    // For now, we'll just verify the connection remains stable
    await page.waitForTimeout(2000)

    const isStillConnected = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.isConnected === true
    })

    expect(isStillConnected).toBe(true)
  })

  test('should display real-time agent data from mock server', async ({ page }) => {
    // Wait for agents data to load
    await page.waitForFunction(
      () => {
        const store = (window as any).__ZUSTAND_STORE__
        return store?.agents?.length > 0
      },
      { timeout: 5000 },
    )

    // Verify agents are displayed in the UI
    const agentsData = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.agents
    })

    expect(agentsData).toBeTruthy()
    expect(Array.isArray(agentsData)).toBe(true)
    expect(agentsData.length).toBeGreaterThan(0)

    // Verify agent properties
    const firstAgent = agentsData[0]
    expect(firstAgent).toHaveProperty('id')
    expect(firstAgent).toHaveProperty('name')
    expect(firstAgent).toHaveProperty('type')
    expect(firstAgent).toHaveProperty('state')
    expect(firstAgent).toHaveProperty('capabilities')
  })

  test('should display real-time task data from mock server', async ({ page }) => {
    // Wait for tasks data to load
    await page.waitForFunction(
      () => {
        const store = (window as any).__ZUSTAND_STORE__
        return store?.tasks?.length > 0
      },
      { timeout: 5000 },
    )

    // Verify tasks are displayed in the UI
    const tasksData = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.tasks
    })

    expect(tasksData).toBeTruthy()
    expect(Array.isArray(tasksData)).toBe(true)
    expect(tasksData.length).toBeGreaterThan(0)

    // Verify task properties
    const firstTask = tasksData[0]
    expect(firstTask).toHaveProperty('id')
    expect(firstTask).toHaveProperty('description')
    expect(firstTask).toHaveProperty('status')
    expect(firstTask).toHaveProperty('priority')
  })

  test('should handle rapid WebSocket message bursts', async ({ page, mockWebSocket }) => {
    // Wait for initial connection
    await page.waitForFunction(
      () => {
        const store = (window as any).__ZUSTAND_STORE__
        return store?.isConnected === true
      },
      { timeout: 5000 },
    )

    // Send multiple rapid updates
    for (let i = 0; i < 5; i++) {
      mockWebSocket.triggerUpdate()
      await page.waitForTimeout(100) // Small delay between updates
    }

    // Verify the application handles the rapid updates without crashing
    await page.waitForTimeout(1000)

    const isStillConnected = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.isConnected === true
    })

    expect(isStillConnected).toBe(true)

    // Verify data is still being received
    const hasData = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.hiveStatus !== null && store?.agents?.length > 0
    })

    expect(hasData).toBe(true)
  })

  test('should maintain connection quality metrics', async ({ page }) => {
    // Wait for connection to be established and stable
    await page.waitForFunction(
      () => {
        const store = (window as any).__ZUSTAND_STORE__
        return store?.isConnected === true && store?.connectionQuality !== 'disconnected'
      },
      { timeout: 5000 },
    )

    // Wait for some time to let connection quality be calculated
    await page.waitForTimeout(3000)

    // Verify connection quality is being tracked
    const connectionQuality = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.connectionQuality
    })

    expect(['excellent', 'good', 'poor']).toContain(connectionQuality)
  })
})
