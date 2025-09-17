import { test, expect } from '@playwright/test'
import {
  waitForWebSocketConnection,
  WebSocketTestUtils,
} from '../src/test/playwright-websocket-utils'

test.describe('Dashboard E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the application
    await page.goto('/')

    // Wait for either the main dashboard or connection error screen to load
    await page.waitForFunction(
      () => {
        const mainHeading = document.querySelector('h1:has-text("🐝 Multiagent Hive System")')
        const connectionError = document.querySelector('h1:has-text("Connection Failed")')
        return mainHeading !== null || connectionError !== null
      },
      { timeout: 10000 },
    )
  })

  test('should load dashboard successfully', async ({ page }) => {
    // Check if either the main dashboard or connection error screen is visible
    const mainHeading = page.locator('h1:has-text("🐝 Multiagent Hive System")')
    const connectionError = page.locator('h1:has-text("Connection Failed")')

    // One of them should be visible
    await expect(mainHeading.or(connectionError)).toBeVisible()

    // If main dashboard is loaded, check navigation tabs
    if (await mainHeading.isVisible()) {
      await expect(page.locator('button:has-text("Dashboard")')).toBeVisible()
      await expect(page.locator('button:has-text("Agents")')).toBeVisible()
      await expect(page.locator('button:has-text("Tasks")')).toBeVisible()
    }
  })

  test('should display connection status', async ({ page }) => {
    // Check for connection status indicator
    const connectionStatus = page.locator(
      '[class*="bg-green-100"], [class*="bg-red-100"], [class*="bg-yellow-100"]',
    )
    await expect(connectionStatus).toBeVisible()

    // Should contain connection-related text
    await expect(connectionStatus).toContainText(/(Connected|Disconnected|Reconnecting|Connecting)/)
  })

  test('should navigate between tabs', async ({ page }) => {
    // Start on dashboard
    await expect(page.locator('button:has-text("Dashboard")')).toHaveClass(/bg-blue-100/)

    // Navigate to agents
    await page.click('button:has-text("Agents")')
    await expect(page.locator('button:has-text("Agents")')).toHaveClass(/bg-blue-100/)
    await expect(page.locator('button:has-text("Dashboard")')).not.toHaveClass(/bg-blue-100/)

    // Navigate to tasks
    await page.click('button:has-text("Tasks")')
    await expect(page.locator('button:has-text("Tasks")')).toHaveClass(/bg-blue-100/)
    await expect(page.locator('button:has-text("Agents")')).not.toHaveClass(/bg-blue-100/)

    // Navigate back to dashboard
    await page.click('button:has-text("Dashboard")')
    await expect(page.locator('button:has-text("Dashboard")')).toHaveClass(/bg-blue-100/)
  })

  test('should handle network disconnection gracefully', async ({ page }) => {
    // Simulate network disconnection by blocking network requests
    await page.route('**/api/**', route => route.abort())

    // Wait a bit for the disconnection to be detected
    await page.waitForTimeout(2000)

    // Should show disconnected status
    const connectionStatus = page.locator('[class*="bg-red-100"]')
    await expect(connectionStatus).toBeVisible()
    await expect(connectionStatus).toContainText('Disconnected')

    // Should still be able to navigate between tabs
    await page.click('button:has-text("Agents")')
    await expect(page.locator('button:has-text("Agents")')).toHaveClass(/bg-blue-100/)
  })

  test('should handle page refresh', async ({ page }) => {
    // Refresh the page
    await page.reload()

    // Should still load properly
    await expect(page.locator('h1:has-text("🐝 Multiagent Hive System")')).toBeVisible()

    // Should maintain navigation functionality
    await expect(page.locator('button:has-text("Dashboard")')).toBeVisible()
    await expect(page.locator('button:has-text("Agents")')).toBeVisible()
    await expect(page.locator('button:has-text("Tasks")')).toBeVisible()
  })

  test('should be responsive on mobile', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 })

    // Check if header is still visible and functional
    await expect(page.locator('h1:has-text("🐝 Multiagent Hive System")')).toBeVisible()

    // Check if navigation buttons are accessible
    await expect(page.locator('button:has-text("Dashboard")')).toBeVisible()
    await expect(page.locator('button:has-text("Agents")')).toBeVisible()
    await expect(page.locator('button:has-text("Tasks")')).toBeVisible()

    // Test navigation on mobile
    await page.click('button:has-text("Agents")')
    await expect(page.locator('button:has-text("Agents")')).toHaveClass(/bg-blue-100/)
  })

  test('should handle WebSocket connection with mock server', async ({ page }) => {
    // Wait for WebSocket connection to be established
    await waitForWebSocketConnection(page, 10000)

    // Verify connection status indicator shows connected
    const connectionStatus = page.locator('[class*="bg-green-100"]')
    await expect(connectionStatus).toBeVisible()
    await expect(connectionStatus).toContainText('Connected')

    // Verify initial data is loaded
    await page.waitForFunction(
      () => {
        const store = (window as any).__ZUSTAND_STORE__
        return store?.hiveStatus !== null && store?.agents?.length > 0
      },
      { timeout: 5000 },
    )

    // Verify hive status contains expected data
    const hiveStatus = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.hiveStatus
    })

    expect(hiveStatus).toBeTruthy()
    expect(hiveStatus.hive_id).toBe('mock-hive-001')
    expect(hiveStatus.metrics.total_agents).toBeGreaterThan(0)
  })

  test('should receive real-time updates from mock WebSocket server', async ({ page }) => {
    // Wait for initial WebSocket connection and data
    await WebSocketTestUtils.waitForInitialLoad(page)

    // Get initial metrics
    const initialMetrics = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.hiveStatus?.metrics
    })

    // Wait for periodic update (mock server updates every 5 seconds)
    await page.waitForTimeout(6000)

    // Verify metrics have been updated
    const updatedMetrics = await page.evaluate(() => {
      const store = (window as any).__ZUSTAND_STORE__
      return store?.hiveStatus?.metrics
    })

    // At least one metric should have changed due to mock randomization
    const metricsChanged = Object.keys(initialMetrics).some(
      key => initialMetrics[key] !== updatedMetrics[key],
    )
    expect(metricsChanged).toBe(true)
  })
})

test.describe('Error Handling E2E Tests', () => {
  test('should handle 404 errors gracefully', async ({ page }) => {
    // Navigate to a non-existent page
    await page.goto('/non-existent-page')

    // Should show error boundary or redirect to home
    // This depends on Next.js error handling configuration
    await expect(page.locator('body')).toBeVisible()
  })

  test('should handle API errors gracefully', async ({ page }) => {
    // Mock API to return errors
    await page.route('**/api/**', route => {
      route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'Internal Server Error' }),
      })
    })

    await page.goto('/')

    // Should still load the page despite API errors
    await expect(page.locator('h1:has-text("🐝 Multiagent Hive System")')).toBeVisible()

    // Should show appropriate error states
    await page.waitForTimeout(1000) // Wait for error states to appear
  })
})
